extern crate config;
extern crate serde;

extern crate serde_derive;
extern crate lazy_static;

mod settings;

use std::sync::{RwLock, RwLockReadGuard};
use settings::Settings;
use lazy_static::lazy_static;

use flexi_logger::{Age, Cleanup, Criterion, Duplicate, LogTarget, Logger, Naming};
use log::{error, info};
use serde_json::to_string_pretty;
use std::convert::Infallible;
use std::result::Result;
use warp::{http::{HeaderMap, StatusCode, Method}, reply, Filter, Rejection, Reply};
mod outgoing;
mod incoming;
use outgoing::{BotType, DealAction, RequestBody};
use incoming::{IncomingSignal, SignalAction};

fn create_actions(action: SignalAction) -> [(DealAction, BotType); 2] {
    match action {
        SignalAction::Buy => [(DealAction::Start, BotType::Long), (DealAction::Close, BotType::Short)],
        SignalAction::Sell => [(DealAction::Close, BotType::Long), (DealAction::Start, BotType::Short)],
    }
}

lazy_static! {
	pub static ref SETTINGS: RwLock<Settings> = match Settings::new() {
        Ok(s) => RwLock::new(s),
        Err(e) => panic!("Error loading config: {:?}", e)
    };
}

pub fn get_settings() -> RwLockReadGuard<'static, Settings> {
    SETTINGS.read().unwrap()
}

fn request_for_action(deal_action_pair: &(DealAction, BotType)) -> String {
    to_string_pretty(&RequestBody::new(deal_action_pair)).unwrap()
}

fn log_request() -> impl Filter<Extract = (), Error = warp::Rejection> + Copy {
    warp::path!("trade")
        .and(warp::method())
        .and(warp::header::headers_cloned())
        .map(|method: Method, headers: HeaderMap| {
            info!("Oho, a {:?} request: {:?}", method, headers);
        })
        .untuple_one()
        .and(warp::body::bytes())
        .map(|bytes: bytes::Bytes| {
            info!("Body: {:?}", bytes);
        })
        .untuple_one()
}

// fn get_json() -> impl Filter<Extract = (), Error = warp::Rejection> + Copy {
//     warp::path!("trade")
//         .and(warp::post())
//         .and(warp::body::json())
//         .map(|signal: IncomingSignal| {
//             info!("Got signal: {:?}", signal);
//             for action in create_actions(signal.action).iter() {
//                 info!("Generating {:?} request: {}", action, request_for_action(action));
//             }
//         })
//         .untuple_one()
// }

fn get_json() -> impl Filter<Extract = (IncomingSignal,), Error = warp::Rejection> + Copy {
    warp::path!("trade")
        .and(warp::post())
        .and(warp::body::json())
        .map(|signal: IncomingSignal| signal)
}

fn log_json(signal: IncomingSignal) {
    info!("Got signal: {:?}", signal);
    for action in create_actions(signal.action).iter() {
        info!("Generating {:?} request: {}", action, request_for_action(action));
    }
}

fn ok_result() -> impl Reply {
    reply::with_status("Success!", StatusCode::OK)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let logger = Logger::with_str("info")
        .log_target(LogTarget::File)
        .rotate(
            Criterion::AgeOrSize(Age::Day, 1000000),
            Naming::Timestamps,
            Cleanup::KeepLogFiles(8),
        )
        .duplicate_to_stderr(Duplicate::Info)
        .start()?;

    info!("Tradeproxy starting up!");

    let api = log_request()
        .and(get_json())
        .map(log_json).untuple_one()
        .map(ok_result)
        .recover(handle_error);

    warp::serve(api).run(([0, 0, 0, 0], 3137)).await;

    logger.shutdown();
    Ok(())
}

async fn handle_error(err: Rejection) -> Result<impl Reply, Infallible> {
    let err_text = format!("Whoa, bad JSON: {:?}", err);

    error!("{}", err_text);

    Ok(reply::with_status(err_text, StatusCode::BAD_REQUEST))
}

#[cfg(test)]
mod tests {
    use super::*;
    use warp::test::{request, RequestBuilder};

    fn mock_request() -> RequestBuilder {
        request().path("/trade").method("POST")
    }

    #[tokio::test]
    async fn it_accepts_valid_json() {
        assert!(
            mock_request()
                .body(r#"{"action": "buy", "contracts": "1"}"#)
                .matches(&get_json())
                .await
        );
    }

    #[tokio::test]
    async fn it_rejects_bad_json() {
        let filter = get_json();

        assert!(!mock_request().body("blah blah blah").matches(&filter).await);

        assert!(
            !mock_request()
                .body(r#"{"wrong": "json"}"#)
                .matches(&filter)
                .await
        );
    }

    #[tokio::test]
    async fn it_accepts_unnecesary_fields_in_json() {
        assert!(mock_request().body(r#"{"action": "buy", "contracts": "1"}"#).matches(&get_json()).await);
    }

    #[tokio::test]
    async fn it_returns_correct_status() {
        assert_eq!(
            mock_request()
                .body("blah blah blah")
                .filter(&get_json().map(|_| ()).untuple_one().map(ok_result).recover(handle_error))
                .await
                .unwrap()
                .into_response()
                .status(),
            StatusCode::BAD_REQUEST
        );

        assert_eq!(
            mock_request()
                .body(r#"{"action": "sell", "contracts": "1"}"#)
                .filter(&get_json().map(|_| ()).untuple_one().map(ok_result).recover(handle_error))
                .await
                .unwrap()
                .into_response()
                .status(),
            StatusCode::OK
        );
    }
}
