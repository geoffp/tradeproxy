use serde::{Deserialize, Serialize};
use serde_json::ser::to_string;

// Start json
//{  "message_type": "bot",  "bot_id": ***REMOVED***,  "email_token": "***REMOVED***",  "delay_seconds": 0}

// Close json
//{  "message_type": "bot",  "bot_id": ***REMOVED***,  "email_token": "***REMOVED***",  "delay_seconds": 0,  "action": "close_at_market_price"}

const EMAIL_TOKEN: &str = "***REMOVED***";
const BOT_ID: u64 = ***REMOVED***;

#[derive(Deserialize, Serialize, Debug)]
pub struct Json {
    pub message_type: &'static str,
    pub bot_id: u64,
    pub email_token: &'static str,
    pub delay_seconds: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<&'static str>,
}

impl Json {
    fn new(request_type: Type) -> Json {
        Json {
            message_type: "bot",
            bot_id: BOT_ID,
            email_token: EMAIL_TOKEN,
            delay_seconds: 0,
            action: match request_type {
                Type::Start => None,
                Type::Close => Some("close_at_market_price"),
            },
        }
    }
}

pub enum Type {
    Start,
    Close,
}

#[cfg(test)]
mod tests {
    use super::*;

    const CORRECT_START_JSON: &str = r#"{"message_type":"bot","bot_id":***REMOVED***,"email_token":"***REMOVED***","delay_seconds":0}"#;
    const CORRECT_CLOSE_JSON: &str = r#"{"message_type":"bot","bot_id":***REMOVED***,"email_token":"***REMOVED***","delay_seconds":0,"action":"close_at_market_price"}"#;

    #[test]
    fn start_json_is_correct() {
        assert_eq!(
            to_string(&Json::new(Type::Start)).unwrap(),
            CORRECT_START_JSON
        );
    }

    #[test]
    fn close_json_is_correct() {
        assert_eq!(
            to_string(&Json::new(Type::Close)).unwrap(),
            CORRECT_CLOSE_JSON
        );
    }
}
