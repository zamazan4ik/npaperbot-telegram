use crate::fetch_database::update_database_thread;
use crate::storage::Paper;
use std::{
    sync::{Arc, Mutex},
    thread,
};
use teloxide::{prelude::*, utils::command::BotCommand};

mod commands;
mod fetch_database;
mod logging;
mod parameters;
mod storage;
mod utils;
mod webhook;

#[tokio::main]
async fn main() {
    run().await;
}

async fn run() {
    logging::init_logger();

    log::info!("Starting npaperbot-telegram");

    let parameters = std::sync::Arc::new(parameters::Parameters::new());
    let bot_parameters = parameters.clone();

    let bot = Bot::from_env();

    let papers = Arc::new(Mutex::new(storage::PaperDatabase::new_empty()));

    let update_papers = papers.clone();
    let papers_database_uri = parameters.papers_database_uri.clone();
    let database_update_periodicity = parameters.database_update_periodicity.clone();
    let h = thread::spawn(move || {
        update_database_thread(
            update_papers,
            papers_database_uri,
            database_update_periodicity,
        )
    });

    let mut bot_dispatcher = Dispatcher::new(bot.clone()).messages_handler(
        move |rx: DispatcherHandlerRx<Bot, Message>| {
            let rx = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);
            rx.for_each(move |message| {
                let parameters = bot_parameters.clone();
                let papers = papers.clone();
                async move {
                    let message_text = match message.update.text() {
                        Some(x) => x,
                        None => return,
                    };

                    // Handle commands
                    match commands::Command::parse(&message_text, &parameters.bot_name) {
                        Ok(command) => {
                            commands::command_answer(
                                &message,
                                command,
                                papers.clone(),
                                parameters.max_results_per_request,
                            )
                            .await
                            .log_on_error()
                            .await;
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

                            let title_pattern = mat.name("title");
                            let revision_pattern = mat.name("revision");

                            if title_pattern.is_none() {
                                log::warn!("Title pattern is empty");
                                break;
                            }

                            let mut pattern = title_pattern.unwrap().as_str().to_lowercase();

                            if let Some(revision_pattern) = revision_pattern {
                                pattern.push_str(
                                    format!("r{}", revision_pattern.as_str().to_lowercase())
                                        .as_str(),
                                );
                            }

                            let (is_result_truncated_t, found_papers) = paper_database
                                .search_by_number(&pattern, parameters.max_results_per_request);

                            is_result_truncated = is_result_truncated_t || is_result_truncated;

                            for paper in found_papers {
                                result_papers.push(paper);

                                if result_papers.len()
                                    == parameters.max_results_per_request as usize
                                {
                                    is_result_truncated = true;
                                    break;
                                }
                            }

                            if is_result_truncated {
                                break;
                            }
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
                            log::info!("Result is truncated");

                            message
                                .reply_to(crate::utils::markdown_v2_escape(
                                    format!(
                                        "Показаны только первые {} результатов. \
                          Если нужного среди них нет - используйте более точный запрос. Спасибо!",
                                        parameters.max_results_per_request
                                    )
                                    .as_str(),
                                ))
                                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                                .send()
                                .await
                                .log_on_error()
                                .await;
                        }
                    }
                }
            })
        },
    );

    if parameters.is_webhook_mode_enabled {
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
