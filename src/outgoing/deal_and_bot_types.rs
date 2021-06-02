use serde::{Serialize, Deserialize};

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
