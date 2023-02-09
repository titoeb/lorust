use async_trait::async_trait;
use serde::Serialize;

use crate::request::{RequestError, SendRequest};

#[derive(Debug, Clone)]
pub struct ReqwestConnection<'a> {
    client: reqwest::Client,
    host: &'a str,
}

impl<'a> ReqwestConnection<'a> {
    pub fn new(host: &'a str) -> Self {
        Self {
            client: reqwest::Client::new(),
            host,
        }
    }
}
#[async_trait]
impl SendRequest for ReqwestConnection<'_> {
    async fn get<'a>(&self, endpoint: &'a str) -> Result<String, RequestError> {
        let response_text = self
            .client
            .get(format!("{}/{}", self.host, endpoint))
            .send()
            .await?
            .text()
            .await?;
        Ok(response_text)
    }
    async fn post<'a>(
        &self,
        endpoint: &'a str,
        body: &'a (impl Serialize + Sync),
    ) -> Result<String, RequestError> {
        let response_text = self
            .client
            .post(format!("{}/{}", self.host, endpoint))
            .json(body)
            .send()
            .await?
            .text()
            .await?;
        Ok(response_text)
    }
}
