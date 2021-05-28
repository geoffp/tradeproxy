use log::{info};
use reqwest::{Client, Response};
use super::{get_settings, incoming::Action};
use serde::Serialize;

pub const REQUEST_URL: &str = "https://3commas.io/trade_signal/trading_view";

pub fn request_url() -> String {
    if cfg!(test) {
        mockito::server_url()
    } else {
        String::from(REQUEST_URL)
    }
}

#[derive(Debug)]
pub enum DealAction {
    Start,
    Close,
}

#[derive(Debug)]
pub enum BotType {
    Long,
    Short,
}

#[derive(Serialize, Debug)]
pub struct OutgoingRequest {
    pub message_type: &'static str,
    pub bot_id: u64,
    pub email_token: String,
    pub delay_seconds: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<&'static str>,
}

pub type ReqwestResult = Result<Response, reqwest::Error>;

pub struct ExecutionResult {
    request: OutgoingRequest,
    result: ReqwestResult,
}

impl ExecutionResult {

    pub fn new(result: ReqwestResult, request: OutgoingRequest) -> ExecutionResult {
        ExecutionResult {
            request,
            result,
        }
    }

    pub fn is_success(&self) -> bool {
        let result = &self.result;
        result.is_ok() && result.as_ref().unwrap().status().is_success()
    }

    pub fn log(&self) {
        if self.is_success() {
            info!("{:?} request successful!", self.request.action);
        }
    }
}

impl OutgoingRequest {
    pub fn new(action: Action) -> OutgoingRequest {
        let settings = get_settings();
        info!(
            "Generating {:?} request.",
            &action,
        );

        let (action, bot_type) = action;
        OutgoingRequest {
            message_type: "bot",
            bot_id: match bot_type {
                BotType::Long => settings.long_bot_id,
                BotType::Short => settings.short_bot_id,
            },
            email_token: settings.email_token.to_string(),
            delay_seconds: 0,
            action: match action {
                DealAction::Start => None,
                DealAction::Close => Some("close_at_market_price"),
            },
        }
    }

    /// Executes the action http request!
    pub async fn execute(self) -> ExecutionResult {
        info!(
            "Executing {:?} Request with: {:?}",
            self.action,
            self
        );

        #[cfg(test)]
        let _mock = OutgoingRequest::mock_request();

        let url: &str = &request_url();
        let client: Client = Client::new();
        let result: ReqwestResult = client.post(url).json(&self).send().await;
        ExecutionResult::new(result, self)
    }

    #[cfg(test)]
    fn mock_request() -> mockito::Mock {
        mockito::mock("POST", "/")
            .with_status(200)
            .create()
    }
}


#[cfg(test)]
mod data_tests {
    use super::*;
    use serde_json::ser::to_string;

    // These just test long bots
    pub const CORRECT_LONG_START_JSON: &str = r#"{"message_type":"bot","bot_id":1234567,"email_token":"89abcdef-789a-bcde-f012-456789abcdef","delay_seconds":0}"#;
    pub const CORRECT_LONG_CLOSE_JSON: &str = r#"{"message_type":"bot","bot_id":1234567,"email_token":"89abcdef-789a-bcde-f012-456789abcdef","delay_seconds":0,"action":"close_at_market_price"}"#;

    #[test]
    fn start_json_is_correct() {
        assert_eq!(
            to_string(&OutgoingRequest::new(&(
                DealAction::Start,
                BotType::Long
            )))
            .unwrap(),
            CORRECT_LONG_START_JSON
        );
    }

    #[test]
    fn close_json_is_correct() {
        assert_eq!(
            to_string(&OutgoingRequest::new(&(
                DealAction::Close,
                BotType::Long
            )))
            .unwrap(),
            CORRECT_LONG_CLOSE_JSON
        );
    }
}

#[cfg(test)]
mod request_tests {
    use super::*;
    use data_tests::CORRECT_LONG_START_JSON;
    use mockito::{mock, Matcher};
    use reqwest::{get, Client, Response};
    use serde_json::json;

    #[tokio::test]
    async fn hello_world() {
        let _m = mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_body("hello world")
            .create();

        async fn req() -> Result<String, reqwest::Error> {
            let url: &str = &request_url();
            let result = get(url).await?.text().await?;
            Ok(result)
        }

        let result = req().await;

        assert!(result.is_ok());
        _m.assert();
        assert_eq!(result.unwrap().as_str(), "hello world");
    }

    #[tokio::test]
    async fn correct_post() {
        let good_json = OutgoingRequest::new(&(DealAction::Start, BotType::Long));
        let good_json_str = String::from(CORRECT_LONG_START_JSON);

        let _m = mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .match_body(Matcher::JsonString(good_json_str))
            .create();

        let result_good = good_json.execute().await;
        _m.assert();
        assert!(result_good.is_ok());
        assert_eq!(result_good.unwrap().status(), reqwest::StatusCode::OK)
    }

    #[tokio::test]
    async fn bad_post() {
        let bad_json = json!({"like": "whatever"});

        let _m = mock("POST", "/")
            .with_header("content-type", "application/json")
            .match_body(Matcher::JsonString(String::from(CORRECT_LONG_START_JSON)))
            .create();

        async fn req_bad(bad_json: &serde_json::Value) -> Result<Response, reqwest::Error> {
            let url: &str = &request_url();
            let client: Client = Client::new();
            let result = client.post(url).json(bad_json).send().await?;
            Ok(result)
        }

        let result_bad: Result<Response, reqwest::Error> = req_bad(&bad_json).await;
        assert_eq!(result_bad.unwrap().status(), reqwest::StatusCode::NOT_IMPLEMENTED);
    }
}
