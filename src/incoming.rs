use serde::Deserialize;
use super::outgoing::{OutgoingRequest, deal_and_bot_types::{DealAction, BotType}};

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

pub type Action = (DealAction, BotType);
pub type ActionPair = (Action, Action);

impl IncomingSignal {
    pub fn to_requests(&self) -> Vec<OutgoingRequest> {
        let action_pair = self.create_actions();
        let (action1, action2) = action_pair;

        vec![
            OutgoingRequest::new(action1),
            OutgoingRequest::new(action2),
        ]
    }

    fn create_actions(&self) -> ActionPair {
        use BotType::*;
        use DealAction::*;
        use SignalAction::*;

        match self.action {
            Buy => (
                (Start, Long),
                (Close, Short)
            ),
            Sell => (
                (Close, Long),
                (Start, Short),
            ),
        }
    }
}
