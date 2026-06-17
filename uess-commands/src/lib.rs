pub mod aur_check;

use serenity::all::{CommandInteraction, Context, CreateCommand};

pub fn register_commands() -> Vec<CreateCommand> {
    vec![aur_check::register()]
}

pub async fn dispatch(ctx: &Context, command: &CommandInteraction) {
    match command.data.name.as_str() {
        "aur-check" => aur_check::run(ctx, command).await,
        _ => {}
    }
}
