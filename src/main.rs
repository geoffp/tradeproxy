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
use outgoing::{OutgoingRequest, deal_and_bot_types::BotType};
pub use settings::{get_settings, Settings, SETTINGS};
use tokio::time::{Duration, sleep};
use std::{collections::HashSet, convert::Infallible, result::Result};
use warp::{Filter, Rejection, Reply, filters::BoxedFilter, http::{HeaderMap, Method, StatusCode}, reply};
use clap::{AppSettings, Clap};

const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

/// Tradeproxy listens for signals to trade and passes them on to 3commas bots.
#[derive(Clap)]
#[clap(version = VERSION, author = AUTHORS)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// Starts both bots and waits for signals. WIthout this option, will just wait for signals.
    #[clap(long)]
    start_bots: bool,

    /// Stops both bots and exits.
    #[clap(long)]
    stop_bots: bool,
}

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
            info!("Got signal {:?}...", signal);
            signal
        })
        .boxed()
}

async fn handle_signal(signal: IncomingSignal, server: String) -> Result<impl Reply, Infallible> {
    tokio::spawn(async move {
        info!("[{:?}] Handling signal: {:?}", Local::now(), signal);
        let requests = signal.to_requests();
        info!("[{:?}] Signal results in requests: {:?}", Local::now(), requests);
        for request in requests {
            info!("[{:?}] Executing request {:?}...", Local::now(), &request);
            let er = request.execute_with_server(server.clone()).await;
            er.log();
            info!("[{:?}] Sleeping for 5s...", Local::now());
            sleep(Duration::from_secs(5)).await;
            info!("Done sleeping!");
        }
    });
    info!("Generating OK result...");
    Ok(StatusCode::OK)
}

fn entire_api(server: String) -> BoxedFilter<(impl Reply,)> {
    get_json()
        .and_then(move |signal| {
            handle_signal(signal, server.clone())
        })
        .recover(handle_error)
        .boxed()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();
    let log_path = &get_settings().log_path;

    // Start the logger!
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

    info!("Tradeproxy {} starting up! Logging to {}", VERSION, log_path);

    // Should we start or stop the 3commas bots?
    if opts.stop_bots {
        stop_bots().await;
        return Ok(());
    }
    if opts.start_bots {
        start_bots().await
    }

    // Start the server!
    let server = get_settings().request_server.clone();
    warp::serve(entire_api(server))
        .run(([0, 0, 0, 0], get_settings().listen_port))
        .await;

    logger.shutdown();
    Ok(())
}

fn both_bots() -> Vec<BotType> {
    vec![BotType::Long, BotType::Short]
}

async fn start_bots() {
    for bot_type in both_bots() {
        let er = OutgoingRequest::start(bot_type).execute().await;
        er.log();
    };
}

async fn stop_bots() {
    for bot_type in both_bots() {
        let er = OutgoingRequest::stop(bot_type).execute().await;
        er.log();
    };
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
    const GOOD_SIGNAL_JSON: &str = r#"{
    "strategy": "fancy v1",
    "position_size": 0.005,
    "order": {
        "action": "buy",
        "contracts": 1,
        "price": 0.3,
        "id": "BUY",
        "comment": "whatev",
        "alert_message": "hwatev"
    },
    "market_position": "long",
    "market_position_size": 0.003,
    "prev_market_position": "short",
    "prev_market_position_size": 0.003
}"#;

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
        const GOOD_BUT_WITH_EXTRA_FIELD: &str = r#"{
    "strategy": "fancy v1",
    "position_size": 0.005,
    "order": {
        "action": "buy",
        "contracts": 1,
        "price": 0.3,
        "id": "BUY",
        "comment": "whatev",
        "alert_message": "hwatev"
    },
    "market_position": "long",
    "market_position_size": 0.003,
    "prev_market_position": "short",
    "prev_market_position_size": 0.003,
    "extra": "whoopee!"
}"#;
        assert!(
            mock_request()
                .body(GOOD_BUT_WITH_EXTRA_FIELD)
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

        sleep(Duration::from_secs(12)).await;

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
