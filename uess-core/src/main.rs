mod honeypot;

use serenity::all::{
    ChannelId, Context, EventHandler, GatewayIntents, GuildId, Interaction, Message, Ready,
};
use serenity::async_trait;
use serenity::Client;

struct Handler {
    honeypot_channel: Option<ChannelId>,
}

// It's a server surrounding something arch-linux based, if you didn't expect there to be cat stuff in here then that's your fault
fn cat_reply(content: &str) -> Option<&'static str> {
    content
        .split_whitespace()
        .find_map(|word| match word.to_lowercase().as_str() {
            "nya" => Some("Nya!"),
            "meow" => Some("mrrrow"),
            _ => None,
        })
}

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
        honeypot::handle(&ctx, &msg, self.honeypot_channel).await;

        if !msg.author.bot {
            if let Some(reply) = cat_reply(&msg.content) {
                let _ = msg.channel_id.say(&ctx.http, reply).await;
            }
        }
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
    let honeypot_channel = honeypot::honeypot_channel_from_env();

    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler { honeypot_channel })
        .await
        .expect("failed to build client");

    if let Err(why) = client.start().await {
        eprintln!("client error: {why:?}");
    }
}
