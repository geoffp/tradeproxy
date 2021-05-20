use serde::{Deserialize, Serialize};
use serde_json::ser::to_string;

// Start json
//{  "message_type": "bot",  "bot_id": ***REMOVED***,  "email_token": "***REMOVED***",  "delay_seconds": 0}

// Close json
//{  "message_type": "bot",  "bot_id": ***REMOVED***,  "email_token": "***REMOVED***",  "delay_seconds": 0,  "action": "close_at_market_price"}

const EMAIL_TOKEN: &str = "***REMOVED***";
const BOT_ID: u64 = ***REMOVED***;

fn is_none<T>(opt: &Option<T>) -> bool {
    match opt {
        Some(_) => false,
        None => true
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OutgoingBotRequest {
    pub message_type: &'static str,
    pub bot_id: u64,
    pub email_token: &'static str,
    pub delay_seconds: u64,
    #[serde(skip_serializing_if = "is_none")]
    pub action: Option<&'static str>
}

pub fn start_deal_request() -> OutgoingBotRequest {
    OutgoingBotRequest {
        message_type: "bot",
        bot_id: BOT_ID,
        email_token: EMAIL_TOKEN,
        delay_seconds: 0,
        action: None
    }
}

pub fn close_deal_request() -> OutgoingBotRequest {
    OutgoingBotRequest {
        message_type: "bot",
        bot_id: BOT_ID,
        email_token: EMAIL_TOKEN,
        delay_seconds: 0,
        action: Some("close_at_market_price")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_json_is_correct() {
        assert_eq!(
            to_string(&start_deal_request()).unwrap(),
            r#"{"message_type":"bot","bot_id":***REMOVED***,"email_token":"***REMOVED***","delay_seconds":0}"#
        );
    }
}
