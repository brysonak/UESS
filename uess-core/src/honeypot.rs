use serenity::all::{ChannelId, Context, Message};

// Remove any message that contains a discord invite link or has 4 or more attachments
// These are common patterns used by those pesky scam bots
pub fn is_violation(attachment_count: usize, content: &str) -> bool {
    attachment_count >= 4
        || content.contains("discord.gg/")
        || content.contains("discord.com/invite/")
        || content.contains("discordapp.com/invite/")
}

// This channel exists only to catch the bots who just spam every channel
// Any message sent here, regardless of whether they are a bot or not, will ban the sender

// If users can't read the big 'DON'T TYPE HERE' message and get banned, it's natural selection at that point
pub fn honeypot_channel_from_env() -> Option<ChannelId> {
    std::env::var("HONEYPOT_CHANNEL")
        .ok()?
        .parse::<u64>()
        .ok()
        .map(ChannelId::new)
}

pub async fn handle(ctx: &Context, msg: &Message, honeypot_channel: Option<ChannelId>) {
    if honeypot_channel == Some(msg.channel_id) {
        // You pesky shits will try and get bots banned, not happening
        if !msg.author.bot {
            if let Some(guild_id) = msg.guild_id {
                let _ = guild_id
                    .ban_with_reason(&ctx.http, msg.author.id, 0, "honeypot channel trigger")
                    .await;
            }
        }
        let _ = msg.delete(&ctx.http).await;
        return;
    }

    if msg.author.bot {
        return;
    }

    if is_violation(msg.attachments.len(), &msg.content) {
        let _ = msg.delete(&ctx.http).await;
    }
}
