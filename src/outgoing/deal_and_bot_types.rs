use serde::{Serialize, Deserialize};
use crate::settings::get_settings;

#[derive(Serialize, Deserialize, Debug)]
pub enum ActionType {
    #[serde(rename = "start_bot")]
    StartBot,
    #[serde(rename = "stop_bot")]
    StopBot,
    StartDeal,
    #[serde(rename = "close_at_market_price")]
    CloseDeal,
}

impl Default for ActionType {
    fn default() -> Self {
        ActionType::StartDeal
    }
}

impl ActionType {
    pub fn is_start(d: &ActionType) -> bool {
        matches!(d, ActionType::StartDeal)
    }
}

#[derive(Debug)]
pub enum BotType {
    Long,
    Short,
}

impl BotType {
    pub fn get_bot_id(&self) -> u64 {
        let settings = get_settings();
        match self {
            BotType::Long => settings.long_bot_id,
            BotType::Short => settings.short_bot_id,
        }
    }
}
