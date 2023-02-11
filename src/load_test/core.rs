use crate::request::definition::RequestDefinition;
use crate::request::interface::{Response, SendRequest};

pub struct LoadTest<'a, R>
where
    R: SendRequest,
{
    connection: R,
    to_call: Vec<RequestDefinition<'a>>,
}

impl<'a, R> LoadTest<'a, R>
where
    R: SendRequest,
{
    pub fn new(connection: R, to_call: Vec<RequestDefinition<'a>>) -> Self {
        Self {
            connection,
            to_call,
        }
    }
    pub fn run(&self) -> Vec<Response> {
        self.to_call
            .iter()
            .map(|post_request_data| match post_request_data {
                RequestDefinition::POST { endpoint, to_json } => {
                    self.connection.post(endpoint, to_json)
                }
                RequestDefinition::GET { endpoint } => self.connection.get(endpoint),
            })
            .filter_map(|response_result| response_result.ok())
            .collect()
    }
}
