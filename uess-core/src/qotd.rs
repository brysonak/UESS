use chrono::{Duration as ChronoDuration, NaiveTime, TimeZone, Utc};
use chrono_tz::America::New_York;
use serenity::all::{ChannelId, Colour, Context, CreateEmbed, CreateMessage};
use serde::Deserialize;

const QOTD_URL: &str = "https://api.harys.is-a.dev/v1/qotd";
const POST_HOUR_EST: u32 = 10;

#[derive(Deserialize)]
struct QotdResponse {
    questions: Vec<String>,
}

pub fn general_chat_from_env() -> Option<ChannelId> {
    std::env::var("GENERAL_CHAT")
        .ok()?
        .parse::<u64>()
        .ok()
        .map(ChannelId::new)
}

async fn fetch_question() -> Result<String, String> {
    let resp: QotdResponse = reqwest::get(QOTD_URL)
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    resp.questions.into_iter().next().ok_or_else(|| "empty questions list".into())
}

async fn post(ctx: &Context, channel: ChannelId) {
    let question = match fetch_question().await {
        Ok(q) => q,
        Err(e) => {
            eprintln!("qotd fetch failed: {e}");
            return;
        }
    };

    let embed = CreateEmbed::new()
        .title("Question of the Day")
        .description(question)
        .colour(Colour::BLITZ_BLUE);

    let _ = channel.send_message(&ctx.http, CreateMessage::new().embed(embed)).await;
}


fn next_post_at() -> chrono::DateTime<Utc> {
    let now_est = Utc::now().with_timezone(&New_York);
    let today_target = now_est.date_naive().and_time(NaiveTime::from_hms_opt(POST_HOUR_EST, 0, 0).unwrap());
    let target_est = New_York
        .from_local_datetime(&today_target)
        .single()
        .unwrap_or_else(|| New_York.from_utc_datetime(&today_target.and_utc().naive_utc()));

    let target_est = if target_est <= now_est {
        target_est + ChronoDuration::days(1)
    } else {
        target_est
    };

    target_est.with_timezone(&Utc)
}

pub async fn run(ctx: Context, channel: ChannelId) {
    loop {
        let wait = next_post_at() - Utc::now();
        tokio::time::sleep(wait.to_std().unwrap_or(std::time::Duration::from_secs(60))).await;
        post(&ctx, channel).await;
    }
}

