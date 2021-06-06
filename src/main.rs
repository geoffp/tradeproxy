extern crate config;
extern crate serde;

extern crate chrono;
extern crate lazy_static;
extern crate serde_derive;

pub mod incoming;
mod outgoing;
pub mod settings;

use chrono::prelude::Local;
use flexi_logger::{Age, Cleanup, Criterion, Duplicate, LogTarget, Logger, Naming};
use incoming::IncomingSignal;
use log::{error, info};
pub use settings::{get_settings, Settings, SETTINGS};
use tokio::time::{Duration, sleep};
use std::{collections::HashSet, convert::Infallible,  result::Result};
use warp::{Filter, Rejection, Reply, filters::BoxedFilter, http::{HeaderMap, Method, StatusCode}, reply};

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

fn get_json() -> BoxedFilter<(IncomingSignal,)> {
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
            info!("Handling signal {:?}...", signal);
            signal
        })
        .boxed()
}

async fn handle_signal(signal: IncomingSignal, server: String) -> Result<impl Reply, Infallible>{
    info!("Got signal: {:?}", signal);
    let requests = signal.to_requests();
    info!("Signal results in requests: {:?}", requests);
    for request in requests {
        info!("Executing request {:?}...", &request);
        let er = request.execute_with_server(server.clone()).await;
        er.log();
        // info!("Sleeping for 5s...");
        // sleep(Duration::from_secs(5)).await;
    }
    info!("Generating OK result...");
    Ok(StatusCode::OK)
}

fn entire_api(server: String) -> BoxedFilter<(impl Reply,)>{
    get_json()
        .and_then(move |signal| {
            handle_signal(signal, server.clone())
        })
        .recover(handle_error)
        .boxed()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log_path = &get_settings().log_path;

    let logger = Logger::with_str("info")
        .log_target(LogTarget::File)
        .directory(log_path)
        .rotate(
            Criterion::AgeOrSize(Age::Day, 1000000),
            Naming::Timestamps,
            Cleanup::KeepLogFiles(8),
        )
        .duplicate_to_stderr(Duplicate::Info)
        .start()?;

    info!("Tradeproxy starting up! Logging to {}", log_path);

    let server = get_settings().request_server.clone();

    warp::serve(entire_api(server))
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
    use httpmock::MockServer;
    use warp::test::{request, RequestBuilder};
    const GOOD_SIGNAL_JSON: &str = r#"{"action": "buy", "contracts": 1}"#;

    fn mock_request() -> RequestBuilder {
        request().path("/trade").method("POST")
    }

    #[tokio::test]
    async fn it_accepts_valid_json() {
        assert!(mock_request()
                .body(GOOD_SIGNAL_JSON)
                .matches(&get_json())
                .await
        );
    }

    #[tokio::test]
    async fn it_rejects_bad_json() {
        let filter = get_json();

        assert!(!mock_request()
                .body("blah blah blah")
                .matches(&filter)
                .await);

        assert!(
            !mock_request()
                .body(r#"{"wrong": "json"}"#)
                .matches(&get_json())
                .await
        );
    }

    #[tokio::test]
    async fn it_accepts_unnecesary_fields_in_json() {
        assert!(
            mock_request()
                .body(r#"{"action": "buy", "contracts": 1, "extra": 42}"#)
                .matches(&get_json())
                .await
        );
    }

    #[tokio::test]
    async fn it_returns_bad_request_for_malformed_json() {
        let server = MockServer::start();
        let _mock = mock_remote_server(&server);
        assert_eq!(
            mock_request()
                .body("blah blah blah")
                .filter(&entire_api(server.base_url()))
                .await
                .unwrap()
                .into_response()
                .status(),
            StatusCode::BAD_REQUEST
        );
    }

    #[tokio::test]
    async fn it_returns_ok_for_good_json() {
        let server = MockServer::start();
        let _mock = mock_remote_server(&server);
        assert_eq!(
            mock_request()
                .body(&GOOD_SIGNAL_JSON)
                .filter(&entire_api(server.base_url()))
                .await
                .unwrap()
                .into_response()
                .status(),
            StatusCode::OK
        );
    }

    #[tokio::test]
    async fn it_returns_bad_request_for_get_request() {
        let server = MockServer::start();
        let _mock = mock_remote_server(&server);
        assert_eq!(
            request()
                .path("/trade")
                .method("GET")
                .filter(&entire_api(server.base_url()))
                .await
                .unwrap()
                .into_response()
                .status(),
            StatusCode::BAD_REQUEST
        );
    }

    #[tokio::test]
    async fn it_rejects_a_put() {
        let server = MockServer::start();
        let _mock = mock_remote_server(&server);
        assert_eq!(
            request()
                .path("/trade")
                .method("PUT")
                .body(&GOOD_SIGNAL_JSON)
                .filter(&entire_api(server.base_url()))
                .await
                .unwrap()
                .into_response()
                .status(),
            StatusCode::BAD_REQUEST
        );
    }

    #[tokio::test]
    async fn makes_both_requests() {
        // simulate the remote API
        let server = MockServer::start();
        let mock = mock_remote_server(&server);

        // Simulate the incoming signal
        let incoming_signal_request = mock_request()
            .body(&GOOD_SIGNAL_JSON)
            .filter(&entire_api(server.base_url()))
            .await;

        assert!(incoming_signal_request.is_ok());
        mock.assert_hits(2);
    }

    fn mock_remote_server<'a>(server: &'a MockServer) -> httpmock::MockRef<'a> {
        server.mock(move |when, then| {
            when.method("POST")
                .path("/trade_signal/trading_view")
                .header("Content-Type", "application/json");
            then.status(200);
        })
    }
}
