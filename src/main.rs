use lazy_static::lazy_static;
use regex::Regex;
use serde_json;
use std::ops::AddAssign;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
};
use teloxide::{prelude::*, utils::command::BotCommand};
use tokio::runtime::Runtime;

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "show generic information about the bot.")]
    About,
}

#[tokio::main]
async fn main() {
    run().await;
}

async fn run() {
    teloxide::enable_logging!();
    log::info!("Starting npaperbot!");

    let bot = Bot::from_env();

    let papers = Arc::new(Mutex::new(HashMap::<String, serde_json::Value>::new()));

    let update_papers = papers.clone();
    let h = thread::spawn(move || update_database_thread(update_papers));

    Dispatcher::new(bot)
        .messages_handler(|rx: DispatcherHandlerRx<Message>| {
            let rx = rx;
            rx.for_each(move |message| {
                let papers = papers.clone();
                async move {
                    let message_text = match message.update.text() {
                        Some(x) => x,
                        None => return,
                    };

                    // Handle commands
                    match Command::parse(&message_text, "cppaperbot") {
                        Ok(command) => {
                            command_answer(&message, command).await.log_on_error().await;
                            return;
                        },
                        Err(_) => ()
                    };

                    let mut result = Vec::<String>::new();
                    {
                        let matches = find_search_request_in_message(&message_text);

                        for mat in matches {
                            let values = papers.lock().unwrap();
                            for (key, value) in values.iter() {
                                if key
                                    .to_lowercase()
                                    .find(mat["title"].to_lowercase().as_str())
                                    != None
                                {
                                    // Format answer here
                                    let mut link_title: String = key.clone();
                                    if let Some(x) = value.get("title") {
                                        link_title.add_assign(": ");
                                        link_title.add_assign(x.as_str().unwrap());
                                    }

                                    let mut one_result = format!(
                                        "[{}]({})",
                                        markdown_v2_escape(link_title.as_str()),
                                        markdown_v2_escape_inline_uri(
                                            value.get("link").unwrap().as_str().unwrap()
                                        )
                                    );

                                    if let Some(x) = value.get("author") {
                                        one_result.add_assign(
                                            format!(
                                                r#" \(by {}\)"#,
                                                markdown_v2_escape(x.as_str().unwrap())
                                            )
                                            .as_str(),
                                        );
                                    }

                                    if let Some(x) = value.get("date") {
                                        one_result.add_assign(
                                            format!(
                                                r#" \({}\)"#,
                                                markdown_v2_escape(x.as_str().unwrap())
                                            )
                                            .as_str(),
                                        );
                                    }

                                    if let Some(x) = value.get("github_url") {
                                        one_result.add_assign(
                                            format!(
                                                r#" \(Related: [GitHub issue]({})\)"#,
                                                markdown_v2_escape(x.as_str().unwrap())
                                            )
                                            .as_str(),
                                        );
                                    }

                                    result.push(one_result);

                                    if result.len() > 20 {
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    if !result.is_empty() {
                        message
                            .reply_to(result.join("\n\n"))
                            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                            .send()
                            .await
                            .log_on_error()
                            .await;
                    }
                }
            })
        })
        .dispatch()
        .await;

    h.join().unwrap();
}

async fn command_answer(cx: &UpdateWithCx<Message>, command: Command) -> ResponseResult<()> {
    static HELP_TEXT: &str =
        "Команды:
        (инлайн-режим) - Просто напишите \
        {Nxxxx|Pxxxx|PxxxxRx|Dxxxx|DxxxxRx|CWGxxx|EWGxxx|LWGxxx|LEWGxxx|FSxxx} в любом сообщении
        /about - информация о боте
        /help - показать это сообщение";
    static ABOUT_TEXT: &str =
        "Репозиторий бота: https://github.com/ZaMaZaN4iK/npaperbot-telegram .\
        Там вы можете получить более подробную справку, оставить отчёт о проблеме или внести \
        какое-либо предложение.";

    match command {
        Command::Help => cx.reply_to(HELP_TEXT).send().await?,
        Command::About => cx.reply_to(ABOUT_TEXT).send().await?,
    };

    Ok(())
}

fn update_database_thread(papers: Arc<Mutex<HashMap<String, serde_json::Value>>>) {
    loop {
        let new_papers = Runtime::new()
            .unwrap()
            .block_on(update_paper_database())
            .unwrap();

        *papers.lock().unwrap() = new_papers;

        println!(
            "Update executed successfully! {}",
            papers.lock().unwrap().len()
        );
        std::thread::sleep(chrono::Duration::hours(1).to_std().unwrap());
    }
}

async fn update_paper_database() -> reqwest::Result<HashMap<String, serde_json::Value>> {
    let resp = reqwest::get("https://raw.githubusercontent.com/wg21link/db/master/index.json")
        .await?
        .json::<HashMap<String, serde_json::Value>>()
        .await?;

    Ok(resp)
}

fn find_search_request_in_message(text: &str) -> regex::CaptureMatches {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"(?i)[\{|\[|<](?P<title>(?:N|P|D|CWG|EWG|LWG|LEWG|FS|EDIT|SD)\d{1,5})(?:R(?P<revision>\d{1,2}))?[\}|\]|>]"#).unwrap();
    }

    RE.captures_iter(text)
}

fn markdown_v2_escape(text: &str) -> String {
    lazy_static! {
        static ref TELEGRAM_ESCAPE_REGEX_ESCAPE: [&'static str; 11] =
            ["{", "}", "[", "]", "(", ")", "+", "*", "|", ".", "-"];
        static ref TELEGRAM_ESCAPE_REGEX_NOT_ESCAPE: [&'static str; 7] =
            ["_", "~", "`", ">", "#", "=", "!"];
        static ref RE: Regex = Regex::new(
            format!(
                r#"(?P<symbol>([\{}{}]))"#,
                &TELEGRAM_ESCAPE_REGEX_ESCAPE.join(r#"\"#),
                &TELEGRAM_ESCAPE_REGEX_NOT_ESCAPE.join("")
            )
            .as_str()
        )
        .unwrap();
    }

    RE.replace_all(text, r#"\$symbol"#).to_string()
}

fn markdown_v2_escape_inline_uri(text: &str) -> String {
    lazy_static! {
        static ref SYMBOLS_FOR_ESCAPING: [&'static str; 2] = [")", "\\"];
        static ref RE: Regex = Regex::new(
            format!(r#"(?P<symbol>([\{}]))"#, &SYMBOLS_FOR_ESCAPING.join(r#"\"#)).as_str()
        )
        .unwrap();
    }

    RE.replace_all(text, r#"\$symbol"#).to_string()
}
