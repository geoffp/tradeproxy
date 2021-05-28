extern crate config;
extern crate serde;

extern crate chrono;
extern crate lazy_static;
extern crate serde_derive;

mod incoming;
mod outgoing;
mod settings;

use chrono::prelude::Local;
use flexi_logger::{Age, Cleanup, Criterion, Duplicate, LogTarget, Logger, Naming};
use incoming::IncomingSignal;
use log::{error, info};
pub use settings::{get_settings, Settings, SETTINGS};
use std::{collections::HashSet, convert::Infallible,  result::Result};
use warp::{
    http::{HeaderMap, Method, StatusCode},
    reply, Filter, Rejection, Reply,
};

fn get_real_remote_ip(headers: &'_ HeaderMap) -> &str {
    let error_message = "[Remote address unknown]";

    let real_ip_header = headers.get("x-real-ip");
    if let Some(h) = real_ip_header {
        match h.to_str() {
            Ok(s) => &s,
            Err(_) => error_message,
        }
    } else {
        error_message
    }
}

fn is_tradingview_ip(remote_ip: &str) -> bool {
    let settings = get_settings();
    let tradingview_apt_ips: &HashSet<String> = &settings.tradingview_api_ips;
    tradingview_apt_ips.contains(&String::from(remote_ip))
}

fn log_remote_source(remote_ip: &str) {
    if is_tradingview_ip(remote_ip) {
        info!("REQUEST FROM TRADINGVIEW, FOR REAL!");
    }
}

fn get_json() -> impl Filter<Extract = (IncomingSignal,), Error = warp::Rejection> + Copy {
    warp::path!("trade")
        .and(warp::path::full())
        .and(warp::method())
        .and(warp::header::headers_cloned())
        .map(
            |path: warp::path::FullPath, method: Method, headers: HeaderMap| {
                let remote_ip = get_real_remote_ip(&headers);
                info!(
                    "[{:?}] Oho, a {:?} request from {} to {:?}: {:?}",
                    Local::now(),
                    method,
                    get_real_remote_ip(&headers),
                    path,
                    headers
                );
                log_remote_source(remote_ip);
            },
        )
        .untuple_one()
        .and(warp::post())
        .and(warp::body::json())
        .map(|signal: IncomingSignal| {
            info!("Handling signal...");
            signal
        })
}

async fn handle_signal(signal: IncomingSignal) {
    info!("Got signal: {:?}", signal);
    let requests = signal.to_requests();
    let iter = requests.into_iter();
    for request in iter {
        let er = request.execute().await;
        er.log();
    }
}

fn ok_result() -> warp::reply::WithStatus<&'static str> {
    info!("Generating OK result...");
    reply::with_status("Success!", StatusCode::OK)
}

fn entire_api() -> impl Filter<Extract = (impl Reply,), Error = Infallible> + Copy + Send {
    get_json()
        .map(handle_signal)
        .map(|_|{}) // consume the Future
        .untuple_one()
        .map(ok_result)
        .recover(handle_error)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log_path = &get_settings().log_path;
    let default_log_path = String::from(".");
    let log_path_str = log_path.as_ref().unwrap_or(&default_log_path);

    let logger = Logger::with_str("info")
        .log_target(LogTarget::File)
        .directory(log_path_str)
        .rotate(
            Criterion::AgeOrSize(Age::Day, 1000000),
            Naming::Timestamps,
            Cleanup::KeepLogFiles(8),
        )
        .duplicate_to_stderr(Duplicate::Info)
        .start()?;

    info!("Tradeproxy starting up! Logging to {}", log_path_str);

    warp::serve(entire_api())
        .run(([0, 0, 0, 0], get_settings().listen_port))
        .await;

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
        assert!(request()
                .path("/trade")
                .method("POST")
                .body(GOOD_JSON)
                .matches(&get_json())
                .await
        );
    }

    #[tokio::test]
    async fn it_rejects_bad_json() {
        let filter = get_json();

        assert!(!request()
                .path("/trade")
                .method("POST")
                .body("blah blah blah")
                .matches(&filter)
                .await);

        assert!(
            !request()
                .path("/trade")
                .method("POST")
                .body(r#"{"wrong": "json"}"#)
                .matches(&get_json())
                .await
        );
    }

    #[tokio::test]
    async fn it_accepts_unnecesary_fields_in_json() {
        assert!(
            request()
                .path("/trade")
                .method("POST")
                .body(r#"{"action": "buy", "contracts": 1, "extra": 42}"#)
                .matches(&get_json())
                .await
        );
    }

    #[tokio::test]
    async fn it_returns_bad_request_for_malformed_json() {
        assert_eq!(
            request()
                .path("/trade")
                .method("POST")
                .body("blah blah blah")
                .filter(&entire_api())
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
            request()
                .path("/trade")
                .method("POST")
                .body(&GOOD_JSON)
                .filter(&entire_api())
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
            request()
                .path("/trade")
                .method("GET")
                .filter(&entire_api())
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
            request()
                .path("/trade")
                .method("PUT")
                .body(&GOOD_JSON)
                .filter(&entire_api())
                .await
                .unwrap()
                .into_response()
                .status(),
            StatusCode::BAD_REQUEST
        );
    }
}
