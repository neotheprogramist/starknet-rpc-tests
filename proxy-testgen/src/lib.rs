pub mod state_machines;

pub trait StateMachine {
    fn run(
        &mut self,
        request_body: String,
        response_body: String,
        path: String,
    ) -> StateMachineResult;
    fn filter(&self, path: String) -> bool;
    fn step(&mut self, request_body: String, response_body: String) -> StateMachineResult;
}

pub enum StateMachineResult {
    Ok(String),
    Invalid(String),
    Skipped,
}
