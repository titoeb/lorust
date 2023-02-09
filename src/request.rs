use async_trait::async_trait;
use serde::Serialize;
#[async_trait]
pub trait SendRequest {
    async fn get<'a>(&self, endpoint: &'a str) -> Result<String, RequestError>;
    async fn post<'a>(
        &self,
        endpoint: &'a str,
        body: &'a (impl Serialize + Sync),
    ) -> Result<String, RequestError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestError {
    RequestUnsuccesful,
}

impl From<reqwest::Error> for RequestError {
    fn from(_: reqwest::Error) -> Self {
        RequestError::RequestUnsuccesful
    }
}
