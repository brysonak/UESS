use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateInteractionResponse, CreateInteractionResponseMessage,
};
use std::time::Duration;
use tokio::sync::OnceCell;

const NPM_MALWARE: &[&str] = &["atomic-lockfile", "js-digest", "lockfile-js", "nextfile-js"];

const RAT_PACKAGES: &[&str] = &[
    "librewolf-fix-bin",
    "firefox-patch-bin",
    "zen-browser-patched-bin",
    "vesktop-bin-patched",
    "minecraft-cracked",
    "ttf-ms-fonts-all",
    "ttf-all-ms-fonts",
];

const SPAM_LIST: &str = include_str!("spam.txt");

const AUR_MALWARE_URL: &str =
    "https://raw.githubusercontent.com/lenucksi/aur-malware-check/refs/heads/master/data/campaigns/aur-infected/packages.txt";

static AUR_REMOTE: OnceCell<Vec<String>> = OnceCell::const_new();

fn is_known_static(pkg: &str) -> Option<&'static str> {
    if NPM_MALWARE.contains(&pkg) {
        return Some("NPM Package");
    }
    if RAT_PACKAGES.contains(&pkg) {
        return Some("Rat Package");
    }
    if SPAM_LIST.lines().any(|l| l.trim() == pkg) {
        return Some("Russian Spam Package");
    }
    None
}

async fn aur_remote_list() -> Result<&'static Vec<String>, String> {
    AUR_REMOTE
        .get_or_try_init(|| async {
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .map_err(|e| e.to_string())?;

            let body = client
                .get(AUR_MALWARE_URL)
                .send()
                .await
                .map_err(|e| e.to_string())?
                .text()
                .await
                .map_err(|e| e.to_string())?;

            Ok(body
                .lines()
                .map(|l| l.trim().to_lowercase())
                .filter(|l| !l.is_empty())
                .collect())
        })
        .await
}

async fn check(pkg: &str) -> Result<Option<&'static str>, String> {
    if let Some(kind) = is_known_static(pkg) {
        return Ok(Some(kind));
    }
    let remote = aur_remote_list().await?;
    Ok(remote.iter().any(|l| l == pkg).then_some("AUR Malware"))
}

pub fn register() -> CreateCommand {
    CreateCommand::new("aur-check")
        .description("Check a package name against known malware/spam lists")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "package", "Package name")
                .required(true),
        )
}

pub async fn run(ctx: &Context, command: &CommandInteraction) {
    let pkg = command
        .data
        .options
        .first()
        .and_then(|o| o.value.as_str())
        .unwrap_or("")
        .trim()
        .to_lowercase();

    let content = match check(&pkg).await {
        Ok(Some(kind)) => format!(
            "`{pkg}` is malicious. Type: {kind}\nIf you have this package installed on your system, please refer to [this](https://discord.com/channels/868690424506773604/868692029616582666/1517282537527971860) immediately"
        ),
        Ok(None) => format!("`{pkg}` is not in any known malware list."),
        Err(e) => format!("Could not reach the remote malware list, only checked local lists, error: {e}"),
    };

    let _ = command
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content(content)),
        )
        .await;
}
