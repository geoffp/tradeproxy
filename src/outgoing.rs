use super::get_settings;
use serde::Serialize;
use reqwest::{Response, Client};

pub const REQUEST_URL: &str = "https://3commas.io/trade_signal/trading_view";

pub fn request_url() -> String {
    if cfg!(test) {
        String::from(mockito::server_url())
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
pub struct OutgoingRequestBody {
    pub message_type: &'static str,
    pub bot_id: u64,
    pub email_token: String,
    pub delay_seconds: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<&'static str>,
}

impl OutgoingRequestBody {
    pub fn new((action, bot_type): &(DealAction, BotType)) -> OutgoingRequestBody {
        OutgoingRequestBody {
            message_type: "bot",
            bot_id: match bot_type {
                BotType::Long => get_settings().long_bot_id,
                BotType::Short => get_settings().short_bot_id,
            },
            email_token: get_settings().email_token.to_string(),
            delay_seconds: 0,
            action: match action {
                DealAction::Start => None,
                DealAction::Close => Some("close_at_market_price"),
            },
        }
    }
}

pub async fn execute_request(request: OutgoingRequestBody) -> Result<Response, reqwest::Error> {
    let url: &str = &request_url();
    let client: Client = Client::new();
    let result = client.post(url).json(&request).send().await?;
    Ok(result)
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
            to_string(&OutgoingRequestBody::new(&(
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
            to_string(&OutgoingRequestBody::new(&(
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
        let good_json = OutgoingRequestBody::new(&(DealAction::Start, BotType::Long));
        let good_json_str = String::from(CORRECT_LONG_START_JSON);

        let _m = mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .match_body(Matcher::JsonString(good_json_str))
            .create();

        let result_good: Result<Response, reqwest::Error> = execute_request(good_json).await;
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
