use base64::{engine::general_purpose::STANDARD, Engine};
use serenity::all::{Context, Message};

const GODBOLT_API: &str = "https://godbolt.org/api/compiler";

// (tag, godbolt language id, compiler id)
const LANGUAGES: &[(&str, &str, &str)] = &[
    ("c",        "c",        "cg141"),
    ("c++",      "c++",      "g141"),
    ("cpp",      "c++",      "g141"),
    ("rust",     "rust",     "r1820"),
    ("go",       "go",       "gl1221"),
    ("python",   "python",   "python312"),
    ("java",     "java",     "java2100"),
    ("haskell",  "haskell",  "ghc961"),
    ("d",        "d",        "ldc1_37_0"),
    ("pascal",   "pascal",   "fpc331"),
    ("fortran",  "fortran",  "gfortran141"),
    ("ada",      "ada",      "gnat141"),
    ("nim",      "nim",      "nim2020"),
    ("zig",      "zig",      "z0130"),
    ("assembly", "assembly", "nasm2_16_01"),
    ("asm",      "assembly", "nasm2_16_01"),
    ("swift",    "swift",    "swift60"),
    ("kotlin",   "kotlin",   "kotlinc2020"),
    ("csharp",   "csharp",   "dotnet9csc"),
    ("c#",       "csharp",   "dotnet9csc"),
    ("fsharp",   "fsharp",   "dotnet9fsharpc"),
    ("f#",       "fsharp",   "dotnet9fsharpc"),
];

fn lookup(tag: &str) -> Option<(&'static str, &'static str)> {
    LANGUAGES
        .iter()
        .find(|(k, _, _)| *k == tag)
        .map(|(_, lang, compiler)| (*lang, *compiler))
}

fn godbolt_url(lang_id: &str, compiler_id: &str, source: &str) -> String {
    let state = format!(
        r#"{{"sessions":[{{"id":1,"language":"{lang_id}","source":{src},"compilers":[],"executors":[{{"compiler":{{"id":"{compiler_id}","libs":[],"options":""}},"stdin":"","args":""}}]}}]}}"#,
        src = serde_json::to_string(source).unwrap_or_default(),
    );
    let encoded = STANDARD.encode(state).replace('/', "%2F");
    format!("https://godbolt.org/clientstate/{encoded}")
}

fn extract_text(arr: &[serde_json::Value]) -> String {
    arr.iter()
        .filter_map(|v| v["text"].as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

pub async fn run(ctx: &Context, msg: &Message, content: &str) {
    let Some(fence_start) = content.find("```") else {
        let _ = msg.reply(&ctx.http, "Wrap your code in a fenced code block with the language.\nExample:\n\\`\\`\\`rust\nfn main() {}\n\\`\\`\\`").await;
        return;
    };

    let after_fence = &content[fence_start + 3..];
    let Some(close) = after_fence.find("```") else {
        let _ = msg.reply(&ctx.http, "Couldn't find closing ` ``` `.").await;
        return;
    };

    let block = &after_fence[..close];
    let (first_line, rest) = block.split_once('\n').unwrap_or((block, ""));
    let lang_tag = first_line.trim().to_lowercase();
    let source = rest.trim();

    if lang_tag.is_empty() {
        let _ = msg.reply(&ctx.http, "Specify a language after the opening backticks, e.g. ` ```rust `.").await;
        return;
    }
    if source.is_empty() {
        let _ = msg.reply(&ctx.http, "Code block is empty.").await;
        return;
    }

    let Some((lang_id, compiler_id)) = lookup(&lang_tag) else {
        let mut keys: Vec<&str> = LANGUAGES.iter().map(|(k, _, _)| *k).collect();
        keys.dedup();
        let _ = msg.reply(&ctx.http, format!("Unsupported language `{lang_tag}`.\nSupported: {}", keys.join(", "))).await;
        return;
    };

    let url = godbolt_url(lang_id, compiler_id, source);

    let payload = serde_json::json!({
        "source": source,
        "compiler": compiler_id,
        "options": {
            "userArguments": "",
            "executeParameters": { "args": "", "stdin": "" },
            "compilerOptions": { "executorRequest": true },
            "filters": { "execute": true },
            "tools": [],
            "libraries": []
        },
        "lang": serde_json::Value::Null,
        "allowStoreCodeDebug": true
    });

    let result: serde_json::Value = match reqwest::Client::new()
        .post(format!("{GODBOLT_API}/{compiler_id}/compile"))
        .header("Accept", "application/json")
        .json(&payload)
        .send()
        .await
    {
        Ok(resp) => match resp.json().await {
            Ok(v) => v,
            Err(e) => {
                let _ = msg.reply(&ctx.http, format!("Failed to parse Godbolt response: {e}")).await;
                return;
            }
        },
        Err(e) => {
            let _ = msg.reply(&ctx.http, format!("Godbolt request failed: {e}")).await;
            return;
        }
    };

    let mut parts: Vec<String> = Vec::new();

    let build_stderr = extract_text(result["buildResult"]["stderr"].as_array().unwrap_or(&vec![]));
    if !build_stderr.is_empty() {
        parts.push(format!("[build errors]\n{build_stderr}"));
    }

    let stdout = extract_text(result["stdout"].as_array().unwrap_or(&vec![]));
    if !stdout.is_empty() {
        parts.push(stdout);
    }

    let stderr = extract_text(result["stderr"].as_array().unwrap_or(&vec![]));
    if !stderr.is_empty() {
        parts.push(format!("[stderr]\n{stderr}"));
    }

    if parts.is_empty() {
        let code = result["code"]
            .as_i64()
            .or_else(|| result["exitCode"].as_i64())
            .map(|c| c.to_string())
            .unwrap_or_else(|| "?".into());
        parts.push(format!("(no output, exit code {code})"));
    }

    let output = parts.join("\n\n");
    let link = format!("\n[view on godbolt](<{url}>)");

    if output.len() + link.len() + 8 > 1990 {
        let _ = msg
            .channel_id
            .send_files(
                &ctx.http,
                [serenity::all::CreateAttachment::bytes(output.into_bytes(), "output.txt")],
                serenity::all::CreateMessage::new().content(link.trim()),
            )
            .await;
    } else {
        let _ = msg.reply(&ctx.http, format!("```\n{output}\n```{link}")).await;
    }
}
