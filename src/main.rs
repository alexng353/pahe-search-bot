use poise::serenity_prelude::{self as serenity};

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

use dotenv::dotenv;
use uuid::Uuid;

use reqwest;
use serde_json::json;
// make the compilre ignore the unused variables in the structs
#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct Anime {
    episodes: u32,
    id: u32,
    #[serde(rename = "type")]
    kind: String,
    poster: String,
    score: f32,
    season: String,
    session: String,
    status: String,
    title: String,
    year: u32,
}

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
        .json::<AnimeSearch>()
        .await?;

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
        })
        .ephemeral(true)
    })
    .await?;

    Ok(())
}

#[poise::command(slash_command)]
async fn anilist(
    ctx: Context<'_>,
    #[description = "Search query"] query: String,
) -> Result<(), Error> {
    let a_query: &str = "query ($page: Int, $perPage: Int, $search: String) {
        Page(page: $page, perPage: $perPage) {
          pageInfo {
            total
            perPage
          }
          media(search: $search, type: ANIME, sort: FAVOURITES_DESC) {
            id
            title {
              romaji
              english
              native
            }
            type
            description
          }
        }
      }
";
    structstruck::strike! {
        #[strikethrough[derive(Debug, serde::Deserialize)]]
        struct AnilistQ {
            data: struct {
                #[serde(rename = "Page")]
                page: struct {
                    media: Vec<struct {
                        id: u32,
                        title: struct {
                            romaji: String,
                            english: String,
                            native: String,
                        },
                        #[serde(rename = "type")]
                        kind: String,
                        description: String,
                    }>,
                },
            },
        }
    }

    let client = reqwest::Client::new();
    let json = json!(
        {
            "query": a_query,
            "variables": {
                "search": query,
                "page": 1,
                "perPage": 1,
            }
        }
    );
    let res = client
        .post("https://graphql.anilist.co")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&json)
        .send()
        .await?
        .json::<AnilistQ>()
        .await?;

    // stringify and send in a code block in an ephemeral embed
    ctx.send(|r| {
        r.embed(|e| {
            e.title(format!(
                "{}",
                // res["data"]["Page"]["media"][0]["title"]["romaji"]
                res.data.page.media[0]
                    .title
                    .romaji
                    .to_string()
                    .replace('"', "")
            ))
            .description(format!(
                "{}",
                // res["data"]["Page"]["media"][0]["description"]
                res.data.page.media[0]
                    .description
                    .to_string()
                    .replace("<br>", "\n")
                    .replace("\\n", "\n")
                    .replace('"', "")
            ))
            // .description(format!("{}", res.to_string()))
        })
        .ephemeral(true)
    })
    .await?;

    // ctx.say(format!("```json\n{}```", res.to_string())).await?;

    Ok(())
}

#[poise::command(slash_command)]
async fn animepahe(
    ctx: Context<'_>,
    #[description = "Search query"] query: String,
) -> Result<(), Error> {
    let client = reqwest::Client::new();

    let res = client
        .get("https://animepahe.ru/api")
        .query(&[("m", "search"), ("q", &query)])
        .send()
        .await?
        .json::<AnimeSearch>()
        .await?;

    // get all the sessions

    let sessions = res.data.iter().enumerate().map(|(i, anime)| {
        (
            format!("{} ({})", anime.title, anime.year),
            format!(
                "Watch: [animepahe](https://animepahe.com/a/{})\n{}",
                anime.id, i
            ),
            false,
        )
    });

    ctx.send(|r| {
        r.embed(|e| e.title("Anime Search").fields(sessions))
            .ephemeral(true);
        r.components(|f| {
            f.create_action_row(|f| {
                for i in 0..res.data.len() - 1 {
                    f.create_button(|b| {
                        b.label(format!("{}", i))
                            .style(serenity::ButtonStyle::Primary)
                            .custom_id(format!("{}", Uuid::new_v4()))
                    });
                }
                f.create_button(|b| {
                    b.label(format!("{}", res.data.len()))
                        .style(serenity::ButtonStyle::Primary)
                        .custom_id(format!("{}", Uuid::new_v4()))
                })
            })
        })
    })
    .await?;

    // ctx.send(|r| {
    //     r.components(|f| {
    //         f.create_action_row(|f| {
    //             f.create_button(|b| {
    //                 b.label("1")
    //                     .style(serenity::ButtonStyle::Primary)
    //                     .custom_id("1")
    //             })
    //         })
    //     })
    // })
    // .await?;

    // attach it to the original message

    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![search(), anilist(), animepahe()],
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
