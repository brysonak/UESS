use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateInteractionResponse, CreateInteractionResponseMessage,
};

const PURGE_DAYS: u8 = 7;

fn is_owner(invoker: u64) -> bool {
    std::env::var("OWNER_ID")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        == Some(invoker)
}

pub fn register() -> CreateCommand {
    CreateCommand::new("cleanse")
        .description("Only Bryson can do this command :-)")
        .add_option(
            CreateCommandOption::new(CommandOptionType::User, "user", "User to cleanse")
                .required(true),
        )
}

pub async fn run(ctx: &Context, command: &CommandInteraction) {
    let content = if !is_owner(command.user.id.get()) {
        "Only the bot owner can run this command.".to_string()
    } else {
        match cleanse(ctx, command).await {
            Ok(target) => format!("Deleted the last {PURGE_DAYS} days of messages from <@{target}> and banned them."),
            Err(e) => e,
        }
    };

    let _ = command
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content(content)),
        )
        .await;
}

async fn cleanse(ctx: &Context, command: &CommandInteraction) -> Result<u64, String> {
    let Some(guild_id) = command.guild_id else {
        return Err("This command only works in a server.".to_string());
    };

    let target = command
        .data
        .options
        .first()
        .and_then(|o| o.value.as_user_id())
        .ok_or("Missing or invalid `user` option.")?;

    guild_id
        .ban_with_reason(&ctx.http, target, PURGE_DAYS, "User was a confirmed spam bot")
        .await
        .map_err(|e| format!("Ban failed: {e}"))?;

    Ok(target.get())
}
