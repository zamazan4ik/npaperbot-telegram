use std::env;

use actix_web::{App, HttpResponse, HttpServer, Responder};
use actix_web::middleware;
use actix_web::web;
use teloxide::prelude::*;
use tokio::sync::mpsc;

async fn telegram_request(tx: web::Data<mpsc::UnboundedSender<Result<Update, String>>>, input: String) -> impl Responder {
    let try_parse = match serde_json::from_str(&input) {
        Ok(update) => Ok(update),
        Err(error) => {
            log::error!(
                "Cannot parse an update.\nError: {:?}\nValue: {}\n\
                       This is a bug in teloxide, please open an issue here: \
                       https://github.com/teloxide/teloxide/issues.",
                error,
                input
            );
            Err(error)
        }
    };
    if let Ok(update) = try_parse {
        tx.send(Ok(update)).expect("Cannot send an incoming update from the webhook")
    }

    HttpResponse::Ok()
}

pub async fn webhook(bot: Bot) -> mpsc::UnboundedReceiver<Result<Update, String>> {
    let bind_address = Result::unwrap_or(env::var("BIND_ADDRESS"), "0.0.0.0".to_string());
    let bind_port: u16 = env::var("BIND_PORT")
        .unwrap_or("8080".to_string())
        .parse()
        .expect("BIND_PORT value has to be an integer");

    let teloxide_token = env::var("TELOXIDE_TOKEN")
        .expect("TELOXIDE_TOKEN env variable missing");
    let host = env::var("HOST")
        .expect("HOST env variable missing");
    let path = format!("/{}/api/v1/message", teloxide_token);
    let url = format!("https://{}/{}", host, path);

    bot.set_webhook(url).send().await.expect("Cannot setup a webhook");

    let (tx, rx) = mpsc::unbounded_channel();

    let sender_channel_data: web::Data<mpsc::UnboundedSender<Result<Update, String>>> = web::Data::new(tx);

    let local = tokio::task::LocalSet::new();
    let sys = actix_rt::System::run_in_tokio("server", &local);
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(sender_channel_data.clone())
            .route(path.as_str(), web::post()
                .to(telegram_request))
    })
        .bind(format!("{}:{}", bind_address, bind_port)).unwrap()
        .run();
    tokio::spawn(sys);

    return rx;
}
