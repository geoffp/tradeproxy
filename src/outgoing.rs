use serde::{Deserialize, Serialize};
use serde_json::ser::to_string;

// Start json
//{  "message_type": "bot",  "bot_id": ***REMOVED***,  "email_token": "***REMOVED***",  "delay_seconds": 0}

// Close json
//{  "message_type": "bot",  "bot_id": ***REMOVED***,  "email_token": "***REMOVED***",  "delay_seconds": 0,  "action": "close_at_market_price"}

const EMAIL_TOKEN: &str = "***REMOVED***";
const BOT_ID: u64 = ***REMOVED***;

#[derive(Deserialize, Serialize, Debug)]
pub struct RequestBody {
    pub message_type: &'static str,
    pub bot_id: u64,
    pub email_token: &'static str,
    pub delay_seconds: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<&'static str>,
}

impl RequestBody {
    fn new(action: DealAction) -> RequestBody {
        RequestBody {
            message_type: "bot",
            bot_id: BOT_ID,
            email_token: EMAIL_TOKEN,
            delay_seconds: 0,
            action: match action {
                DealAction::Start => None,
                DealAction::Close => Some("close_at_market_price"),
            },
        }
    }

    fn start() -> RequestBody {
        RequestBody::new(DealAction::Start)
    }

    fn close() -> RequestBody {
        RequestBody::new(DealAction::Close)
    }
}

pub enum DealAction {
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
            to_string(&RequestBody::start()).unwrap(),
            CORRECT_START_JSON
        );
    }

    #[test]
    fn close_json_is_correct() {
        assert_eq!(
            to_string(&RequestBody::close()).unwrap(),
            CORRECT_CLOSE_JSON
        );
    }
}
