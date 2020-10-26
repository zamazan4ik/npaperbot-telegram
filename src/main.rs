use lazy_static::lazy_static;
use regex::Regex;
use serde_json;
use tokio::runtime::Runtime;

use chrono::Duration;
use std::ops::AddAssign;
use std::{
    collections::HashMap,
    env,
    sync::{Arc, Mutex},
    thread,
};
use teloxide::{prelude::*, utils::command::BotCommand};

mod logging;
mod search;
mod utils;
mod webhook;

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "show generic information about the bot.")]
    About,
    #[command(description = "search C++ proposal with a title part or an author name.")]
    Search,
}

#[tokio::main]
async fn main() {
    run().await;
}

async fn run() {
    logging::init_logger();

    log::info!("Starting npaperbot-telegram");

    let bot = Bot::from_env();

    let papers = Arc::new(Mutex::new(HashMap::<String, serde_json::Value>::new()));

    let papers_database_uri =
        env::var("PAPERS_DATABASE_URI").unwrap_or("https://wg21.link/index.json".to_string());

    let max_results_per_request = env::var("MAX_RESULTS_PER_REQUEST")
        .unwrap_or("20".to_string())
        .parse::<u8>()
        .expect("Cannot convert MAX_RESULTS_PER_REQUEST to u8");

    let database_update_periodicity = Duration::hours(
        env::var("DATABASE_UPDATE_PERIODICITY_IN_HOURS")
            .unwrap_or("1".to_string())
            .parse::<i64>()
            .expect("Cannot convert DATABASE_UPDATE_PERIODICITY_IN_HOURS to hours"),
    );

    let update_papers = papers.clone();
    let h = thread::spawn(move || {
        update_database_thread(
            update_papers,
            papers_database_uri,
            database_update_periodicity,
        )
    });

    let is_webhook_mode_enabled = env::var("WEBHOOK_MODE")
        .unwrap_or("false".to_string())
        .parse::<bool>()
        .expect(
            "Cannot convert WEBHOOK_MODE to bool. Applicable values are only \"true\" or \"false\"",
        );

    let bot_dispatcher =
        Dispatcher::new(bot.clone()).messages_handler(move |rx: DispatcherHandlerRx<Message>| {
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
                        }
                        Err(_) => (),
                    };

                    let mut result = Vec::<String>::new();
                    let mut is_result_truncated = false;
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
                                        utils::markdown_v2_escape(link_title.as_str()),
                                        utils::markdown_v2_escape_inline_uri(
                                            value.get("link").unwrap().as_str().unwrap()
                                        )
                                    );

                                    if let Some(x) = value.get("author") {
                                        one_result.add_assign(
                                            format!(
                                                r#" \(by {}\)"#,
                                                utils::markdown_v2_escape(x.as_str().unwrap())
                                            )
                                            .as_str(),
                                        );
                                    }

                                    if let Some(x) = value.get("date") {
                                        one_result.add_assign(
                                            format!(
                                                r#" \({}\)"#,
                                                utils::markdown_v2_escape(x.as_str().unwrap())
                                            )
                                            .as_str(),
                                        );
                                    }

                                    if let Some(x) = value.get("github_url") {
                                        one_result.add_assign(
                                            format!(
                                                r#" \(Related: [GitHub issue]({})\)"#,
                                                utils::markdown_v2_escape(x.as_str().unwrap())
                                            )
                                            .as_str(),
                                        );
                                    }

                                    result.push(one_result);

                                    if result.len() > max_results_per_request as usize {
                                        is_result_truncated = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    result.sort_unstable();

                    if !result.is_empty() {
                        message
                            .reply_to(result.join("\n\n"))
                            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                            .send()
                            .await
                            .log_on_error()
                            .await;

                        if is_result_truncated {
                            message
                                .reply_to(format!("Показаны только первые {} результатов. \
                                Если нужного среди них нет - используйте более точный запрос. Спасибо!",
                                                  max_results_per_request))
                                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                                .send()
                                .await
                                .log_on_error()
                                .await;
                        }
                    }
                }
            })
        });

    if is_webhook_mode_enabled {
        log::info!("Webhook mode activated");
        let rx = webhook::webhook(bot);
        bot_dispatcher
            .dispatch_with_listener(
                rx.await,
                LoggingErrorHandler::with_custom_text("An error from the update listener"),
            )
            .await;
    } else {
        log::info!("Long polling mode activated");
        bot.delete_webhook()
            .send()
            .await
            .expect("Cannot delete a webhook");
        bot_dispatcher.dispatch().await;
    }

    h.join().unwrap();
}

async fn command_answer(cx: &UpdateWithCx<Message>, command: Command) -> ResponseResult<()> {
    static HELP_TEXT: &str = "Команды:
        (инлайн-режим) - Просто напишите \
        {Nxxxx|Pxxxx|PxxxxRx|Dxxxx|DxxxxRx|CWGxxx|EWGxxx|LWGxxx|LEWGxxx|FSxxx} в любом сообщении
        /about - информация о боте
        /search - поиск бумаги по её номеру, части названия или автору
        /help - показать это сообщение";
    static ABOUT_TEXT: &str =
        "Репозиторий бота: https://github.com/ZaMaZaN4iK/npaperbot-telegram .\
        Там вы можете получить более подробную справку, оставить отчёт о проблеме или внести \
        какое-либо предложение.";

    match command {
        Command::Help => cx.reply_to(HELP_TEXT).send().await?,
        Command::About => cx.reply_to(ABOUT_TEXT).send().await?,
        Command::Search => {
            cx.reply_to("Not implemented yet. Stay tuned :)")
                .send()
                .await?
        }
    };

    Ok(())
}

fn update_database_thread(
    papers: Arc<Mutex<HashMap<String, serde_json::Value>>>,
    uri: String,
    update_periodicity: Duration,
) {
    loop {
        let new_papers = Runtime::new()
            .unwrap()
            .block_on(update_paper_database(&uri))
            .unwrap();

        *papers.lock().unwrap() = new_papers;

        log::info!(
            "Update executed successfully! Papers database size: {}",
            papers.lock().unwrap().len()
        );

        std::thread::sleep(
            update_periodicity
                .to_std()
                .expect("Cannot convert to std time"),
        );
    }
}

async fn update_paper_database(
    uri: &String,
) -> reqwest::Result<HashMap<String, serde_json::Value>> {
    let resp = reqwest::get(uri)
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
