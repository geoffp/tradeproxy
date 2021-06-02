use serde::{Serialize, Deserialize};
use crate::settings::get_settings;

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
        matches!(d, DealAction::Start)
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
