use log::info;
use serde::{Deserialize, Serialize};
use reqwest::{Client, Response};
use super::{get_settings, incoming::Action};

pub fn request_server() -> String {
    "https://3commas.io".into()
}

pub fn request_path() -> String {
    "/trade_signal/trading_view".into()
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DealAction {
    Start,
    #[serde(rename = "close_at_market_price")]
    Close,
}

impl Default for DealAction {
    fn default() -> Self {
        DealAction::Start
    }
}

impl DealAction {
    pub fn is_start(d: &DealAction) -> bool {
        match d {
            DealAction::Start => true,
            _ => false
        }
    }
}

#[derive(Debug)]
pub enum BotType {
    Long,
    Short,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OutgoingRequest {
    pub message_type: String,
    pub bot_id: u64,
    pub email_token: String,
    pub delay_seconds: u64,
    #[serde(default)]
    #[serde(skip_serializing_if = "DealAction::is_start")]
    pub action: DealAction,
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

    pub fn status(&self) -> reqwest::StatusCode {
        self.result.as_ref().unwrap().status()
    }

    pub fn is_success(&self) -> bool {
        self.status().is_success()
    }

    pub fn log(&self) {
        if self.is_success() {
            info!("{:?} request successful!", self.request.action);
        } else {
            info!("{:?} request failed. :(", self.request.action);
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
            message_type: "bot".into(),
            bot_id: match bot_type {
                BotType::Long => settings.long_bot_id,
                BotType::Short => settings.short_bot_id,
            },
            email_token: settings.email_token.to_string(),
            delay_seconds: 0,
            action,
        }
    }

    /// Executes the action http request!
    pub async fn execute(self) -> ExecutionResult {
        self.execute_with_server(request_server()).await
    }

    pub async fn execute_with_server(self, server: String) -> ExecutionResult {
        info!(
            "Executing {:?} Request with: {:?}",
            self.action,
            self
        );

        let url = format!("{}{}", server, request_path());
        let client: Client = Client::new();
        let result: ReqwestResult = client.post(url).json(&self).send().await;
        ExecutionResult::new(result, self)
    }
}


#[cfg(test)]
mod data_tests {
    use super::*;

    // These just test long bots
    pub const CORRECT_LONG_START_JSON: &str = r#"{"message_type":"bot","bot_id":1234567,"email_token":"89abcdef-789a-bcde-f012-456789abcdef","delay_seconds":0}"#;
    pub const CORRECT_LONG_CLOSE_JSON: &str = r#"{"message_type":"bot","bot_id":1234567,"email_token":"89abcdef-789a-bcde-f012-456789abcdef","delay_seconds":0,"action":"close_at_market_price"}"#;

    #[test]
    fn start_json_is_correct() {
        let request = OutgoingRequest::new((
            DealAction::Start,
            BotType::Long
        ));
        assert_eq!(
            serde_json::to_string(&request)
            .unwrap(),
            CORRECT_LONG_START_JSON
        );
    }

    #[test]
    fn close_json_is_correct() {
        let request = OutgoingRequest::new((
            DealAction::Close,
            BotType::Long
        ));
        assert_eq!(
            serde_json::to_string(&request)
            .unwrap(),
            CORRECT_LONG_CLOSE_JSON
        );
    }
}

#[cfg(test)]
mod request_tests {
    use super::*;
    use reqwest::Client;
    use serde_json::json;
    use httpmock::{MockServer, HttpMockRequest};

    fn request_is_valid_json(req: &HttpMockRequest) -> bool {
        if req.body.is_none() { return false; };
        let bytes: &Vec<u8> = req.body.as_ref().unwrap();
        let s: String = String::from_utf8_lossy(&bytes).into_owned();
        let or: Result<OutgoingRequest, serde_json::Error> = serde_json::from_str(&s);
        if or.is_err() { return false; }
        return true;
    }

    #[tokio::test]
    async fn correct_post() {
        let good_json = OutgoingRequest::new((DealAction::Start, BotType::Long));
        // let good_json_str = String::from(CORRECT_LONG_START_JSON);

        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method("POST")
                .path("/trade_signal/trading_view")
                .header("Content-Type", "application/json")
                .matches(|req| {
                    request_is_valid_json(&req)
                });
            then.status(200)
                .body("Success!");
        });

        let result_good = good_json.execute_with_server(server.base_url()).await;

        mock.assert();

        assert!(result_good.is_success());
    }

    #[tokio::test]
    async fn bad_post() {
        let bad_json = json!({"like": "whatever"});

        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method("POST")
                .path("/trade_signal/trading_view")
                .header("Content-Type", "application/json")
                .matches(|req| {
                    !request_is_valid_json(&req)
                });
            then.status(501)
                .body("Fail!");
        });

        async fn req_bad(bad_json: &serde_json::Value, base_url: String) -> ReqwestResult {
            let url: String = format!("{}{}", base_url, request_path());
            let client: Client = Client::new();
            let result = client.post(url).json(bad_json).send().await?;

            Ok(result)
        }

        let result_bad: ReqwestResult = req_bad(&bad_json, server.base_url()).await;
        mock.assert();
        assert!(result_bad.unwrap().status().is_server_error());
    }
}
