use crate::request::{RequestError, Response, SendRequest};
use erased_serde::Serialize;
use std::time::Instant;
#[derive(Debug, Clone)]
pub struct ReqwestConnection<'a> {
    client: reqwest::blocking::Client,
    host: &'a str,
}

impl<'a> ReqwestConnection<'a> {
    pub fn new(host: &'a str) -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
            host,
        }
    }
}
impl SendRequest for ReqwestConnection<'_> {
    fn get(&self, endpoint: &'_ str) -> Result<Response, RequestError> {
        let request = self
            .client
            .get(format!("{}/{}", self.host, endpoint))
            .build()?;

        let request_send = Instant::now();
        let response = self.client.execute(request)?;
        let reponse_time = request_send.elapsed();

        let response_text = response.text()?;
        Ok(Response::new(response_text, reponse_time))
    }
    fn post<'a>(
        &self,
        endpoint: &'a str,
        body: &'a dyn Serialize,
    ) -> Result<Response, RequestError> {
        let request = self
            .client
            .post(format!("{}/{}", self.host, endpoint))
            .json(body)
            .build()?;

        let request_send = Instant::now();
        let response = self.client.execute(request)?;
        let reponse_time = request_send.elapsed();

        let response_text = response.text()?;
        Ok(Response::new(response_text, reponse_time))
    }
}
