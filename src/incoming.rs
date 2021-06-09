use serde::Deserialize;
use super::outgoing::{OutgoingRequest, deal_and_bot_types::{ActionType, BotType}};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum SignalAction {
    Buy,
    Sell,
}

#[derive(Deserialize, Debug)]
pub struct IncomingSignal {
    // pub action: SignalAction,
    // pub contracts: f64,
    pub position_size: f64,
    pub order: IncomingSignalOrder,
    pub market_position: String,
    pub market_position_size: f64,
    pub prev_market_position: String,
    pub prev_market_position_size: f64,
}

#[derive(Deserialize, Debug)]
pub struct IncomingSignalOrder {
    pub action: SignalAction,
    pub contracts: f64,
    pub price: f64,
    pub id: String,
    pub comment: String,
    pub alert_message: String,
}

pub type Action = (ActionType, BotType);
pub type ActionPair = (Action, Action);

impl IncomingSignal {
    pub fn to_requests(&self) -> Vec<OutgoingRequest> {
        let (action1, action2) = self.create_actions();

        vec![
            OutgoingRequest::new(action1),
            OutgoingRequest::new(action2),
        ]
    }

    fn create_actions(&self) -> ActionPair {
        use BotType::*;
        use ActionType::*;
        use SignalAction::*;

        // The order of these is important! Have to close the open deal before we try to open one.
        match self.order.action {
            Buy => (
                (CloseDeal, Short),
                (StartDeal, Long),
            ),
            Sell => (
                (CloseDeal, Long),
                (StartDeal, Short),
            ),
        }
    }
}
