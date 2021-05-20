use serde::Deserialize;

// Start json
//{  "message_type": "bot",  "bot_id": ***REMOVED***,  "email_token": "***REMOVED***",  "delay_seconds": 0}

// Close json
//{  "message_type": "bot",  "bot_id": ***REMOVED***,  "email_token": "***REMOVED***",  "delay_seconds": 0,  "action": "close_at_market_price"}

const EMAIL_TOKEN: &str = "***REMOVED***";
const BOT_ID: u64 = ***REMOVED***;

#[derive(Deserialize, Debug)]
pub struct OutgoingBotRequest {
    pub message_type: &'static str,
    pub bot_id: u64,
    pub email_token: &'static str,
    pub delay_seconds: u64,
    pub action: Option<String>
}

pub fn start_request() -> OutgoingBotRequest {
    OutgoingBotRequest {
        message_type: "bot",
        bot_id: BOT_ID,
        email_token: EMAIL_TOKEN,
        delay_seconds: 0,
        action: None
    }
}
