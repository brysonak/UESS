use serenity::all::{
    CommandInteraction, Context, CreateCommand, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};

pub fn register() -> CreateCommand {
    CreateCommand::new("source-code").description("Get a link to the bot's source code")
}

pub async fn run(ctx: &Context, command: &CommandInteraction) {
    let content = "The source code for this bot can be found at: https://github.com/brysonak/UESS";

    let _ = command
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content(content)),
        )
        .await;
}