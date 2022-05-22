use teloxide::{prelude::*, utils::command::BotCommands};

#[derive(Clone, BotCommands)]
#[command(rename = "lowercase", description = "These commands are supported:")]
pub(crate) enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "show generic information about the bot.")]
    About,
    #[command(description = "search C++ proposal with a title part or an author name.")]
    Search(String),
}

#[allow(unused_assignments)]
pub(crate) async fn command_handler(
    msg: Message,
    bot: AutoSend<Bot>,
    command: Command,
    papers: crate::storage::PapersStorage,
    limit: u8,
) -> anyhow::Result<()> {
    static HELP_TEXT: &str = "Команды:
        (инлайн-режим) - Просто напишите \
        [Nxxxx|Pxxxx|PxxxxRx|Dxxxx|DxxxxRx|CWGxxx|EWGxxx|LWGxxx|LEWGxxx|FSxxx] в любом сообщении
        /about - информация о боте
        /search - поиск бумаги по её номеру, части названия или автору
        /help - показать это сообщение";
    static ABOUT_TEXT: &str =
        "Репозиторий бота: https://github.com/ZaMaZaN4iK/npaperbot-telegram .\
        Там вы можете получить более подробную справку, оставить отчёт о проблеме или внести \
        какое-либо предложение.";

    match command {
        Command::Help => {
            bot.send_message(msg.chat.id, HELP_TEXT)
                .reply_to_message_id(msg.id)
                .await?;
        }
        Command::About => {
            bot.send_message(msg.chat.id, ABOUT_TEXT)
                .reply_to_message_id(msg.id)
                .await?;
        }
        Command::Search(pattern) => {
            let mut is_limit_reached = false;
            let mut found_papers = Vec::<crate::storage::Paper>::new();

            {
                let paper_database = papers.lock().unwrap();
                let (is_limit_reached_t, found_papers_t) =
                    paper_database.search_any(&pattern, limit);
                is_limit_reached = is_limit_reached_t;
                found_papers = found_papers_t;
            }

            if !found_papers.is_empty() {
                bot.send_message(
                    msg.chat.id,
                    crate::utils::convert_papers_to_result(found_papers),
                )
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .reply_to_message_id(msg.id)
                .await?;

                if is_limit_reached {
                    bot.send_message(
                        msg.chat.id,
                        crate::utils::markdown_v2_escape(
                            format!(
                                "Показаны только первые {} результатов. \
                          Если нужного среди них нет - используйте более точный запрос. Спасибо!",
                                limit
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
                    )
                    )
                    .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                    .reply_to_message_id(msg.id)
                    .await?;
            }
        }
    };

    Ok(())
}
