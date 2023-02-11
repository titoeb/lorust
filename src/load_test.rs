use crate::request::{Response, SendRequest};
use erased_serde::Serialize;

pub enum RequestData<'a> {
    POST {
        endpoint: &'a str,
        to_json: &'a dyn Serialize,
    },
    GET {
        endpoint: &'a str,
    },
}

pub struct LoadTest<'a, R>
where
    R: SendRequest,
{
    connection: R,
    to_call: Vec<RequestData<'a>>,
}

impl<'a, R> LoadTest<'a, R>
where
    R: SendRequest,
{
    pub fn new(connection: R, to_call: Vec<RequestData<'a>>) -> Self {
        Self {
            connection,
            to_call,
        }
    }
    pub fn run(&self) -> Vec<Response> {
        self.to_call
            .iter()
            .map(|post_request_data| match post_request_data {
                RequestData::POST { endpoint, to_json } => self.connection.post(endpoint, to_json),
                RequestData::GET { endpoint } => self.connection.get(endpoint),
            })
            .filter_map(|response_result| response_result.ok())
            .collect()
    }
}
