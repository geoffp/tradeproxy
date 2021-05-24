extern crate config;
extern crate serde;

extern crate serde_derive;
extern crate lazy_static;
extern crate chrono;

mod settings;
mod outgoing;
mod incoming;

use std::{convert::Infallible,result::Result, collections::HashSet};
pub use settings::{Settings, SETTINGS, get_settings};
use flexi_logger::{Age, Cleanup, Criterion, Duplicate, LogTarget, Logger, Naming};
use log::{error, info};
use serde_json::to_string_pretty;
use warp::{http::{HeaderMap, StatusCode, Method}, reply, Filter, Rejection, Reply};
use outgoing::{BotType, DealAction, RequestBody};
use incoming::{IncomingSignal, SignalAction};
use chrono::prelude::Local;

fn create_actions(action: SignalAction) -> [(DealAction, BotType); 2] {
    match action {
        SignalAction::Buy => [(DealAction::Start, BotType::Long), (DealAction::Close, BotType::Short)],
        SignalAction::Sell => [(DealAction::Close, BotType::Long), (DealAction::Start, BotType::Short)],
    }
}

fn request_for_action(deal_action_pair: &(DealAction, BotType)) -> String {
    to_string_pretty(&RequestBody::new(deal_action_pair)).unwrap()
}

fn get_real_remote_ip<'a>(headers: &'a HeaderMap) -> &str {
    let error_message = "[Remote address unknown]";

    let real_ip_header = headers.get("x-real-ip");
    if let Some(h) = real_ip_header {
        match h.to_str() {
            Ok(s) => &s,
            Err(_) => error_message
        }
    } else {
        error_message
    }
}

// fn is_tradingview_ip() {

// }

fn log_remote_source(remote_ip: &str) {
    let settings = get_settings();
    let tradingview_apt_ips: &HashSet<String> = &settings.tradingview_api_ips;
    if tradingview_apt_ips.contains(&String::from(remote_ip)) {
        info!("REQUEST FROM TRADINGVIEW, FOR REAL!");
    }
}

fn get_json() -> impl Filter<Extract = (IncomingSignal,), Error = warp::Rejection> + Copy {
    warp::path!("trade")
        .and(warp::path::full())
        .and(warp::method())
        .and(warp::header::headers_cloned())
        .map(|path: warp::path::FullPath, method: Method, headers: HeaderMap| {
            let remote_ip = get_real_remote_ip(&headers);
            info!("[{:?}] Oho, a {:?} request from {} to {:?}: {:?}", Local::now(), method, get_real_remote_ip(&headers), path, headers);
            log_remote_source(remote_ip);
        })
        .untuple_one()
        .and(warp::post())
        .and(warp::body::json())
        .map(|signal: IncomingSignal| {
            info!("Handling signal...");
            signal
        })
}

fn log_json(signal: IncomingSignal) {
    info!("Got signal: {:?}", signal);
    for action in create_actions(signal.action).iter() {
        info!("Generating {:?} request: {}", action, request_for_action(action));
    }
}

fn ok_result() -> impl Reply {
    info!("Generating OK result...");
    reply::with_status("Success!", StatusCode::OK)
}

// fn entire_api() -> Recover {
//     get_json()
//         .map(log_json).untuple_one()
//         .map(ok_result)
//         .recover(handle_error)
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let logger = Logger::with_str("info")
        .log_target(LogTarget::File)
        .directory(&get_settings().log_dir)
        .rotate(
            Criterion::AgeOrSize(Age::Day, 1000000),
            Naming::Timestamps,
            Cleanup::KeepLogFiles(8),
        )
        .duplicate_to_stderr(Duplicate::Info)
        .start()?;

    info!("Tradeproxy starting up!");

    let api =
        get_json()
        .map(log_json).untuple_one()
        .map(ok_result)
        .recover(handle_error);

    warp::serve(api).run(([0, 0, 0, 0], get_settings().listen_port)).await;

    logger.shutdown();
    Ok(())
}

async fn handle_error(err: Rejection) -> Result<impl Reply, Infallible> {
    let err_text = format!("Rejected: {:?}", err);

    error!("{}", err_text);

    Ok(reply::with_status(err_text, StatusCode::BAD_REQUEST))
}

#[cfg(test)]
mod tests {
    use super::*;
    use warp::test::{request, RequestBuilder};
    const GOOD_JSON: &str = r#"{"action": "buy", "contracts": 1}"#;

    fn mock_request() -> RequestBuilder {
        request().path("/trade").method("POST")
    }

    #[tokio::test]
    async fn it_accepts_valid_json() {
        assert!(
            mock_request()
                .body(GOOD_JSON)
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
        assert!(mock_request().body(GOOD_JSON).matches(&get_json()).await);
    }

    #[tokio::test]
    async fn it_returns_bad_request_for_malformed_json() {
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
    }

    #[tokio::test]
    async fn it_returns_ok_for_good_json() {
        assert_eq!(
            mock_request()
                .method("POST")
                .body(&GOOD_JSON)
                .filter(&get_json().map(|_| ()).untuple_one().map(ok_result).recover(handle_error))
                .await
                .unwrap()
                .into_response()
                .status(),
            StatusCode::OK
        );
    }

    #[tokio::test]
    async fn it_returns_bad_request_for_get_request() {
        assert_eq!(
            mock_request()
                .method("GET")
                .filter(&get_json().map(|_| ()).untuple_one().map(ok_result).recover(handle_error))
                .await
                .unwrap()
                .into_response()
                .status(),
            StatusCode::BAD_REQUEST
        );
    }

    #[tokio::test]
    async fn it_rejects_a_put() {
        assert_eq!(
            mock_request()
                .method("PUT")
                .body(GOOD_JSON)
                .filter(&get_json().map(|_| ()).untuple_one().map(ok_result).recover(handle_error))
                .await
                .unwrap()
                .into_response()
                .status(),
            StatusCode::BAD_REQUEST
        );
    }
}
