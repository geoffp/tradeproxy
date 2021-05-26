use serde::Deserialize;
use super::outgoing::{OutgoingRequest, DealAction, BotType};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum SignalAction {
    Buy,
    Sell,
}

#[derive(Deserialize, Debug)]
pub struct IncomingSignal {
    pub action: SignalAction,
    pub contracts: f64,
}

impl IncomingSignal {
    pub fn to_requests(&self) -> Vec<OutgoingRequest> {
        self.create_actions().iter()
            .map(|action_pair| OutgoingRequest::new(action_pair))
            .collect()
    }

    fn create_actions(&self) -> Vec<(DealAction, BotType)> {
        match self.action {
            SignalAction::Buy => vec![
                (DealAction::Start, BotType::Long),
                (DealAction::Close, BotType::Short),
            ],
            SignalAction::Sell => vec![
                (DealAction::Close, BotType::Long),
                (DealAction::Start, BotType::Short),
            ],
        }
    }
}
