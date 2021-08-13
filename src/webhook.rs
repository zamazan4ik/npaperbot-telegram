use axum::extract::Extension;
use axum::http::StatusCode;
use axum::prelude::*;
use axum::response::IntoResponse;
use axum::routing::RoutingDsl;
use teloxide::dispatching::{stop_token::AsyncStopToken, update_listeners::StatefulListener};
use teloxide::prelude::Request;
use teloxide::prelude::*;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tower::ServiceBuilder;
use tower_http::{add_extension::AddExtensionLayer, trace::TraceLayer};

async fn telegram_request(
    input: String,
    tx: Extension<mpsc::UnboundedSender<Result<teloxide::types::Update, String>>>,
) -> impl IntoResponse {
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
        tx.send(Ok(update))
            .expect("Cannot send an incoming update from the webhook")
    }

    StatusCode::OK
}

pub async fn webhook(
    bot: Bot,
) -> impl teloxide::dispatching::update_listeners::UpdateListener<String> {
    let bind_address = Result::unwrap_or(std::env::var("BIND_ADDRESS"), "0.0.0.0".to_string());
    let bind_port: u16 = std::env::var("BIND_PORT")
        .unwrap_or("8080".to_string())
        .parse()
        .expect("BIND_PORT value has to be an integer");

    let teloxide_token =
        std::env::var("TELOXIDE_TOKEN").expect("TELOXIDE_TOKEN env variable missing");
    let host = std::env::var("HOST").expect("HOST env variable missing");
    let path = format!("/{}/api/v1/message", teloxide_token);
    let url = format!("https://{}{}", host, path);

    bot.set_webhook(url.parse().unwrap())
        .send()
        .await
        .expect("Cannot setup a webhook");

    let (tx, rx) = mpsc::unbounded_channel();

    let app = route(path.as_str(), post(telegram_request)).layer(
        ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(AddExtensionLayer::new(tx))
            .into_inner(),
    );

    let server_address: std::net::SocketAddr = format!("{}:{}", bind_address, bind_port)
        .parse()
        .expect("Unable to parse socket address");

    tokio::spawn(async move {
        axum::Server::bind(&server_address)
            .serve(app.into_make_service())
            .await
            .expect("Axum server error")
    });

    let stream = UnboundedReceiverStream::new(rx);

    fn streamf<S, T>(state: &mut (S, T)) -> &mut S {
        &mut state.0
    }

    let (stop_token, _) = AsyncStopToken::new_pair();
    StatefulListener::new(
        (stream, stop_token),
        streamf,
        |state: &mut (_, AsyncStopToken)| state.1.clone(),
    )
}
