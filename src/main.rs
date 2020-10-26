use crate::fetch_database::update_database_thread;
use crate::storage::Paper;
use chrono::Duration;
use std::{
    env,
    sync::{Arc, Mutex},
    thread,
};
use teloxide::{prelude::*, utils::command::BotCommand};

mod fetch_database;
mod logging;
mod storage;
mod utils;
mod webhook;

type PapersStorage = Arc<Mutex<storage::PaperDatabase>>;

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "show generic information about the bot.")]
    About,
    #[command(description = "search C++ proposal with a title part or an author name.")]
    Search(String),
}

#[tokio::main]
async fn main() {
    run().await;
}

async fn run() {
    logging::init_logger();

    log::info!("Starting npaperbot-telegram");

    let bot = Bot::from_env();

    let papers = Arc::new(Mutex::new(storage::PaperDatabase::new_empty()));

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
                            command_answer(&message, command, papers.clone(), max_results_per_request).await.log_on_error().await;
                            return;
                        }
                        Err(_) => (),
                    };

                    let mut result_papers = Vec::<Paper>::new();
                    let mut is_result_truncated = false;
                    {
                        let matches = utils::find_search_request_in_message(&message_text);

                        for mat in matches {
                            let paper_database = papers.lock().unwrap();

                            let (is_result_truncated_t, found_papers) =
                                paper_database.search_by_number(&mat["title"].to_lowercase(),
                                                                max_results_per_request);

                            is_result_truncated = is_result_truncated_t || is_result_truncated;
                            result_papers = found_papers;

                            // FIXME: For now we support only one implicit request per message
                            // Possibly will be extended later
                            break;
                        }
                    }

                    if !result_papers.is_empty() {
                        message
                            .reply_to(utils::convert_papers_to_result(result_papers))
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

#[allow(unused_assignments)]
async fn command_answer(
    cx: &UpdateWithCx<Message>,
    command: Command,
    papers: PapersStorage,
    limit: u8,
) -> ResponseResult<()> {
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
        Command::Help => {
            cx.reply_to(HELP_TEXT).send().await?;
        }
        Command::About => {
            cx.reply_to(ABOUT_TEXT).send().await?;
        }
        Command::Search(pattern) => {
            let mut is_limit_reached = false;
            let mut found_papers = Vec::<Paper>::new();

            {
                let paper_database = papers.lock().unwrap();
                let (is_limit_reached_t, found_papers_t) =
                    paper_database.search_any(&pattern, limit);
                is_limit_reached = is_limit_reached_t;
                found_papers = found_papers_t;
            }

            if !found_papers.is_empty() {
                cx.reply_to(utils::convert_papers_to_result(found_papers))
                    .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                    .send()
                    .await
                    .log_on_error()
                    .await;

                if is_limit_reached {
                    cx.reply_to(utils::markdown_v2_escape(
                        format!(
                            "Показаны только первые {} результатов. \
                          Если нужного среди них нет - используйте более точный запрос. Спасибо!",
                            limit
                        )
                        .as_str(),
                    ))
                    .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                    .send()
                    .await
                    .log_on_error()
                    .await;
                }
            } else {
                cx.reply_to(utils::markdown_v2_escape(
                    "К сожалению, по Вашему запросу ничего не найдено. Попробуйте другой запрос!",
                ))
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .send()
                .await
                .log_on_error()
                .await;
            }
        }
    };

    Ok(())
}
