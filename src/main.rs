use crate::fetch_database::update_database_thread;
use crate::storage::Paper;
use anyhow::anyhow;
use teloxide::prelude::*;

mod commands;
mod fetch_database;
mod implicit_search_request_parser;
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

    let bot = Bot::from_env().auto_send();

    let papers = std::sync::Arc::new(std::sync::Mutex::new(storage::PaperDatabase::new_empty()));

    let update_papers = papers.clone();
    let papers_database_uri = parameters.papers_database_uri.clone();
    let database_update_periodicity = parameters.database_update_periodicity.clone();

    let _ = tokio::spawn(async move {
        update_database_thread(
            update_papers,
            papers_database_uri,
            database_update_periodicity
                .to_std()
                .expect("Cannot convert Duration to std"),
        )
        .await;
    });

    let handler = Update::filter_message()
        .branch(
            dptree::entry()
                .filter_command::<commands::Command>()
                .endpoint(commands::command_handler),
        )
        .branch(
            dptree::filter(|msg: Message| msg.text().is_some()).endpoint(
                |msg: Message,
                 bot: AutoSend<Bot>,
                 papers: crate::storage::PapersStorage,
                 max_results_per_request: u8| async move {
                    process_message(msg, bot, papers, max_results_per_request).await?;
                    anyhow::Result::Ok(())
                },
            ),
        );

    if !parameters.is_webhook_mode_enabled {
        log::info!("Webhook deleted");
        bot.delete_webhook().await.expect("Cannot delete a webhook");
    }

    let mut bot_dispatcher = Dispatcher::builder(bot.clone(), handler)
        .dependencies(dptree::deps![papers, parameters.max_results_per_request])
        .default_handler(|_| async move {})
        .error_handler(LoggingErrorHandler::with_custom_text(
            "An error has occurred in the dispatcher",
        ))
        .build();

    if parameters.is_webhook_mode_enabled {
        log::info!("Webhook mode activated");
        let rx = webhook::webhook(bot);
        bot_dispatcher
            .setup_ctrlc_handler()
            .dispatch_with_listener(
                rx.await,
                LoggingErrorHandler::with_custom_text("An error from the update listener"),
            )
            .await;
    } else {
        log::info!("Long polling mode activated");
        bot_dispatcher.setup_ctrlc_handler().dispatch().await;
    }
}

async fn process_message(
    msg: Message,
    bot: AutoSend<Bot>,
    papers: crate::storage::PapersStorage,
    max_results_per_request: u8,
) -> anyhow::Result<()> {
    let mut at_least_one_valid_request = false;
    let mut result_papers = Vec::<Paper>::new();
    let mut is_result_truncated = false;
    {
        let paper_requests = utils::find_search_request_in_message(
            msg.text()
                .ok_or_else(|| anyhow!("Cannot find text in the message"))?,
        );

        match paper_requests {
            Ok(paper_requests) => {
                for paper_request in paper_requests {
                    at_least_one_valid_request = true;
                    let paper_type = paper_request.paper_type;
                    let paper_number = paper_request.paper_number;
                    let revision_number = paper_request.revision_number;

                    let mut pattern = format!("{}{}", paper_type, paper_number);

                    if let Some(revision_number) = revision_number {
                        pattern.push_str(format!("r{}", revision_number).as_str());
                    }

                    let paper_database = papers.lock().unwrap();
                    let (is_result_truncated_t, found_papers) =
                        paper_database.search_by_number(&pattern, max_results_per_request);

                    is_result_truncated = is_result_truncated_t || is_result_truncated;

                    for paper in found_papers {
                        result_papers.push(paper);

                        if result_papers.len() == max_results_per_request as usize {
                            is_result_truncated = true;
                            break;
                        }
                    }

                    if is_result_truncated {
                        break;
                    }
                }
            }
            Err(err) => {
                log::warn!("Implicit search request parse error: {:?}", err)
            }
        }
    }

    if at_least_one_valid_request {
        if !result_papers.is_empty() {
            bot.send_message(msg.chat.id, utils::convert_papers_to_result(result_papers))
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .reply_to_message_id(msg.id)
                .await?;

            if is_result_truncated {
                log::info!("Result is truncated");

                bot.send_message(
                    msg.chat.id,
                    crate::utils::markdown_v2_escape(
                        format!(
                            "Показаны только первые {} результатов. \
                          Если нужного среди них нет - используйте более точный запрос. Спасибо!",
                            max_results_per_request
                        )
                        .as_str(),
                    ),
                )
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .reply_to_message_id(msg.id)
                .await?;
            }
        } else {
            bot.send_message(
                msg.chat.id,
                crate::utils::markdown_v2_escape(
                    "К сожалению, по Вашему запросу ничего не найдено. Попробуйте другой запрос!",
                ),
            )
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .reply_to_message_id(msg.id)
            .await?;
        }
    }

    Ok(())
}
