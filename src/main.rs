use poise::serenity_prelude::{self as serenity};

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

use dotenv::dotenv;

use reqwest;
// make the compilre ignore the unused variables in the structs
#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct Anime {
    id: u32,
    title: String,
    poster: String,
    score: f32,
    episodes: u32,
    year: u32,
    season: String,
    status: String,
    #[serde(rename = "type")]
    kind: String,
}

// make a type for the above struct
#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct AnimeSearch {
    data: Vec<Anime>,
    from: u32,
    last_page: u32,
    per_page: u32,
    to: u32,
    total: u32,
}

#[poise::command(slash_command)]
async fn search(
    ctx: Context<'_>,
    #[description = "Search query"] query: String,
) -> Result<(), Error> {
    let client = reqwest::Client::new();

    let res = client
        .get("https://animepahe.ru/api")
        .query(&[("m", "search"), ("q", &query)])
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let res = serde_json::from_value::<AnimeSearch>(res)?;

    let data = res
        .data
        .iter()
        .map(|anime| {
            (
                format!("{} ({})", anime.title, anime.year),
                format!(
                    "Watch: [animepahe](https://animepahe.com/a/{})\nScore: **{}**\nEpisodes: **{}**\nStatus: **{}**\nType: **{}**",
                    anime.id,
                    anime.score,
                    anime.episodes, 
                    anime.status, 
                    anime.kind
                ),
                false,
            )
        })
        .collect::<Vec<_>>();

    let data = data.iter().take(4).cloned().collect::<Vec<_>>();

    ctx.send(|r| {
        r.embed(|e| {
            e.title("Anime Search")
                .description(format!("Found {} results", res.total))
                .fields(data)
        }).ephemeral(true)
    })
    .await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![search()],
            ..Default::default()
        })
        .token(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        });

    println!("Ready!");
    framework.run().await.unwrap();
}
