mod honeypot;

use serenity::all::{
    Context, EventHandler, GatewayIntents, GuildId, Interaction, Message, Ready,
};
use serenity::async_trait;
use serenity::Client;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} connected", ready.user.name);

        let commands = uess_commands::register_commands();

        if let Ok(guild_id) = std::env::var("GUILD_ID") {
            if let Ok(id) = guild_id.parse::<u64>() {
                let _ = GuildId::new(id)
                    .set_commands(&ctx.http, commands)
                    .await;
                return;
            }
        }

        for command in commands {
            let _ = serenity::all::Command::create_global_command(&ctx.http, command).await;
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        honeypot::handle(&ctx, &msg).await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            uess_commands::dispatch(&ctx, &command).await;
        }
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let token = std::env::var("TOKEN").expect("TOKEN not set in .env");

    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("failed to build client");

    if let Err(why) = client.start().await {
        eprintln!("client error: {why:?}");
    }
}
