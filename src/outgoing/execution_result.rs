use log::{debug, info};
use reqwest::Response;
use super::{OutgoingRequest, deal_and_bot_types::DealAction};

pub type ReqwestResult = Result<Response, reqwest::Error>;

pub struct ExecutionResult {
    request: OutgoingRequest,
    result: ReqwestResult,
}

impl ExecutionResult {
    pub fn new(result: ReqwestResult, request: OutgoingRequest) -> Self {
        ExecutionResult {
            request,
            result,
        }
    }

    pub fn status(&self) -> reqwest::StatusCode {
        self.result.as_ref().unwrap().status()
    }

    pub fn is_success(&self) -> bool {
        self.status().is_success()
    }

    pub fn log(&self) {
        let action: &DealAction = &self.request.action;
        let result: &ReqwestResult = &self.result;
        let result_bytes = &result.as_ref().unwrap();

        if self.is_success() {
            info!("{:?} request successful", action);
        } else {
            info!("{:?} request failed :(", action);
            debug!("Result content: {:?}", result_bytes);
        }
    }
}
