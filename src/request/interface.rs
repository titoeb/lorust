use core::fmt;
use erased_serde::Serialize;
use std::time::Duration;
pub trait SendRequest {
    fn get(&self, endpoint: &'_ str) -> Result<Response, RequestError>;
    fn post<'a>(
        &self,
        endpoint: &'a str,
        body: &'a dyn Serialize,
    ) -> Result<Response, RequestError>;
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
#[derive(Debug, Clone)]
pub struct Response {
    text: String,
    response_time: Duration,
}
impl Response {
    pub fn new(text: String, response_time: Duration) -> Self {
        Self {
            text,
            response_time,
        }
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Reponse: '{}'\n Response_time: '{}'",
            self.text,
            to_millisecond(self.response_time)
        )
    }
}

fn to_millisecond(duration: Duration) -> f64 {
    duration.as_nanos() as f64 / 1_000_000.0
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::Duration;

    #[test]
    fn ten_milliseconds_to_milliseconds() {
        let duration = Duration::from_millis(1000);

        assert_eq!(to_millisecond(duration), 1000.0)
    }

    #[test]
    fn from_nanos_to_milliseconds() {
        let duration = Duration::from_nanos(1_000_000);

        assert_eq!(to_millisecond(duration), 1.0)
    }
    #[test]
    fn one_and_a_half_milliseconds() {
        let duration = Duration::from_nanos(1_500_000);

        assert_eq!(to_millisecond(duration), 1.5)
    }
}
