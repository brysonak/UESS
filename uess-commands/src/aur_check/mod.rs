use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateInteractionResponse, CreateInteractionResponseMessage,
};
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
    "https://raw.githubusercontent.com/lenucksi/aur-malware-check/refs/heads/master/package_list.txt";

static AUR_REMOTE: OnceCell<Vec<String>> = OnceCell::const_new();

fn is_known_static(pkg: &str) -> Option<&'static str> {
    if NPM_MALWARE.contains(&pkg) {
        return Some("npm package");
    }
    if RAT_PACKAGES.contains(&pkg) {
        return Some("rat pkg");
    }
    if SPAM_LIST.lines().any(|l| l.trim() == pkg) {
        return Some("russian spam pkg");
    }
    None
}

async fn aur_remote_list() -> &'static Vec<String> {
    AUR_REMOTE
        .get_or_init(|| async {
            let body = match reqwest::get(AUR_MALWARE_URL).await {
                Ok(resp) => resp.text().await.unwrap_or_default(),
                Err(_) => String::new(),
            };
            body.lines()
                .map(|l| l.trim().to_lowercase())
                .filter(|l| !l.is_empty())
                .collect()
        })
        .await
}

async fn check(pkg: &str) -> Option<&'static str> {
    if let Some(kind) = is_known_static(pkg) {
        return Some(kind);
    }
    if aur_remote_list().await.iter().any(|l| l == pkg) {
        return Some("aur malware");
    }
    None
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
        Some(kind) => format!(
            "Package is malicious. Type: {kind}\nIf you have this package installed on your system, please refer to [this](https://github.com/lenucksi/aur-malware-check#what-to-do-if-infected) immediately"
        ),
        None => format!("`{pkg}` is not in any known malware list."),
    };

    let _ = command
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content(content)),
        )
        .await;
}

#[cfg(test)]
mod tests {
    use super::is_known_static;

    #[test]
    fn catches_npm_malware() {
        assert_eq!(is_known_static("js-digest"), Some("npm package"));
    }

    #[test]
    fn catches_rat_packages() {
        assert_eq!(is_known_static("minecraft-cracked"), Some("rat pkg"));
    }

    #[test]
    fn catches_spam_list_entries() {
        assert_eq!(is_known_static("nikto-git"), Some("russian spam pkg"));
    }

    #[test]
    fn clean_package_is_none() {
        assert_eq!(is_known_static("firefox"), None);
    }
}
