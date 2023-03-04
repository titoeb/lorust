use core::fmt;
use core::fmt::Debug;
use erased_serde::Serialize;
use std::time::Duration;

pub trait SerializableInThread: Serialize + Sync + Debug {}
impl<T> SerializableInThread for T where T: Serialize + Sync + Debug {}

pub trait HTTPClient {
    fn get(&self, endpoint: &'_ str) -> Result<TimedResponse, RequestError>;
    fn post<'a>(
        &self,
        endpoint: &'a str,
        body: &'a dyn SerializableInThread,
    ) -> Result<TimedResponse, RequestError>;
}

impl<'a> serde::Serialize for dyn SerializableInThread + 'a {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        erased_serde::serialize(self, serializer)
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusCodeGroup {
    Information,
    Success,
    Redirect,
    ClientError,
    ServerError,
    Unknown,
}
impl std::fmt::Display for StatusCodeGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                StatusCodeGroup::Information => "Information",
                StatusCodeGroup::Success => "Success",
                StatusCodeGroup::Redirect => "Redirect",
                StatusCodeGroup::ClientError => "Client Error",
                StatusCodeGroup::ServerError => "Server Error",
                Self::Unknown => "Unkown Server Error reported",
            },
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimedResponse {
    pub status_code_group: StatusCodeGroup,
    pub response_time: Duration,
}
impl TimedResponse {
    pub fn new(status_code_group: StatusCodeGroup, response_time: Duration) -> Self {
        Self {
            status_code_group,
            response_time,
        }
    }
}
impl fmt::Display for TimedResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Status: '{}'\n Response_time: '{}'",
            self.status_code_group,
            to_millisecond(self.response_time)
        )
    }
}

pub(crate) fn to_millisecond(duration: Duration) -> f64 {
    duration.as_nanos() as f64 / 1_000_000.0
}

pub fn to_seconds(duration: Duration) -> f64 {
    to_millisecond(duration) / 1000.0
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::Duration;

    #[test]
    fn thousand_milliseconds_to_milliseconds() {
        let duration = Duration::from_millis(1_000);

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

    #[test]
    fn thousand_milliseconds_to_seconds() {
        let duration = Duration::from_millis(1_000);

        assert_eq!(to_seconds(duration), 1.0)
    }

    #[test]
    fn from_nanos_to_second() {
        let duration = Duration::from_nanos(1_000_000);

        assert_eq!(to_seconds(duration), 0.001)
    }
    #[test]
    fn one_and_a_half_seconds() {
        let duration = Duration::from_nanos(1_500_000);

        assert_eq!(to_seconds(duration), 0.0015)
    }

    #[test]
    fn display_simple_response() {
        assert_eq!(
            format!(
                "{}",
                TimedResponse::new(StatusCodeGroup::Success, Duration::new(10, 2))
            ),
            String::from("Status: 'Success'\n Response_time: '10000.000002'")
        )
    }
}
