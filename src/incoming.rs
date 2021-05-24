use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum SignalAction {
    Buy,
    Sell
}

#[derive(Deserialize, Debug)]
pub struct IncomingSignal {
    pub action: SignalAction,
    pub contracts: f64,
}
