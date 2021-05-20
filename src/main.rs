use flexi_logger::{Age, Cleanup, Criterion, Duplicate, LogTarget, Logger, Naming};
use log::{error, info};
use serde::Deserialize;
use std::convert::Infallible;
use std::result::Result;
use warp::{http::StatusCode, reply, Filter, Rejection, Reply};
mod outgoing;

#[derive(Deserialize, Debug)]
pub struct IncomingSignal {
    pub action: String,
    pub contracts: String,
}

fn get_json() -> impl Filter<Extract = ((),), Error = warp::Rejection> + Copy {
    warp::path!("trade")
        .and(warp::post())
        .and(warp::body::json())
        .map(|req: IncomingSignal| {
            info!("Successful request: {:?}", req);
        })
}

fn ok_result(_: ()) -> impl Reply {
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

    let api = get_json().map(ok_result).recover(handle_error);

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
    async fn it_accepts_good_json() {
        let filter = get_json();
        assert!(
            mock_request()
                .body(r#"{"action": "foo", "contracts": "bar"}"#)
                .matches(&filter)
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
        let filter = get_json();
        assert!(mock_request().body(r#"{"action": "foo", "contracts": "bar"}"#).matches(&filter).await);
    }

    #[tokio::test]
    async fn it_returns_correct_status() {
        assert_eq!(
            mock_request()
                .body("blah blah blah")
                .filter(&get_json().map(ok_result).recover(handle_error))
                .await
                .unwrap()
                .into_response()
                .status(),
            StatusCode::BAD_REQUEST
        );

        assert_eq!(
            mock_request()
                .body(r#"{"action": "foo", "contracts": "bar"}"#)
                .filter(&get_json().map(ok_result).recover(handle_error))
                .await
                .unwrap()
                .into_response()
                .status(),
            StatusCode::OK
        );
    }
}
