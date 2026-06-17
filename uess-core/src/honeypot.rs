use serenity::all::{Context, Message};

// Remove any message that contains a discord invite link or has 4 or more attachments
// These are common patterns used by those pesky scam bots
pub fn is_violation(attachment_count: usize, content: &str) -> bool {
    attachment_count >= 4
        || content.contains("discord.gg/")
        || content.contains("discord.com/invite/")
        || content.contains("discordapp.com/invite/")
}

pub async fn handle(ctx: &Context, msg: &Message) {
    if msg.author.bot {
        return;
    }
    if is_violation(msg.attachments.len(), &msg.content) {
        let _ = msg.delete(&ctx.http).await;
    }
}

#[cfg(test)]
mod tests {
    use super::is_violation;

    #[test]
    fn flags_four_or_more_images() {
        assert!(!is_violation(3, "hello"));
        assert!(is_violation(4, "hello"));
    }

    #[test]
    fn flags_discord_invite_links() {
        assert!(is_violation(0, "join here discord.gg/abc123"));
        assert!(is_violation(0, "discord.com/invite/abc123"));
        assert!(!is_violation(0, "no links here"));
    }
}
