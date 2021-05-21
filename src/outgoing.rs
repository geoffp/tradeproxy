use serde::Serialize;

use super::{get_settings};

// Start json
//{  "message_type": "bot",  "bot_id": ***REMOVED***,  "email_token": "***REMOVED***",  "delay_seconds": 0}

// Close json
//{  "message_type": "bot",  "bot_id": ***REMOVED***,  "email_token": "***REMOVED***",  "delay_seconds": 0,  "action": "close_at_market_price"}

// const EMAIL_TOKEN: &str = "***REMOVED***";
// const BOT_ID: u64 = ***REMOVED***;

#[derive(Serialize, Debug)]
pub struct RequestBody {
    pub message_type: &'static str,
    pub bot_id: u64,
    pub email_token: String,
    pub delay_seconds: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<&'static str>,
}

impl RequestBody {
    pub fn new((action, bot_type): &(DealAction, BotType)) -> RequestBody {
        RequestBody {
            message_type: "bot",
            bot_id: match bot_type {
                BotType::Long => get_settings().long_bot_id,
                BotType::Short => get_settings().short_bot_id
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::ser::to_string;

    const CORRECT_START_JSON: &str = r#"{"message_type":"bot","bot_id":***REMOVED***,"email_token":"***REMOVED***","delay_seconds":0}"#;
    const CORRECT_CLOSE_JSON: &str = r#"{"message_type":"bot","bot_id":***REMOVED***,"email_token":"***REMOVED***","delay_seconds":0,"action":"close_at_market_price"}"#;

    #[test]
    fn start_json_is_correct() {
        assert_eq!(
            to_string(&RequestBody::new(&(DealAction::Start, BotType::Long))).unwrap(),
            CORRECT_START_JSON
        );
    }

    #[test]
    fn close_json_is_correct() {
        assert_eq!(
            to_string(&RequestBody::new(&(DealAction::Close, BotType::Long))).unwrap(),
            CORRECT_CLOSE_JSON
        );
    }
}
