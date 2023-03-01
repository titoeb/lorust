use crate::request::interface::StatusCodeGroup;
use crate::request::interface::{HTTPClient, RequestError, SerializableInThread, TimedResponse};
use mockall::automock;
use std::time::{Duration, Instant};
#[derive(Debug, Clone)]
pub struct ReqwestClient<'a> {
    client: reqwest::blocking::Client,
    host: &'a str,
}

#[automock]
trait Client {
    fn execute(
        &self,
        request: reqwest::blocking::Request,
    ) -> Result<reqwest::blocking::Response, reqwest::Error>;
}

impl Client for reqwest::blocking::Client {
    fn execute(
        &self,
        request: reqwest::blocking::Request,
    ) -> Result<reqwest::blocking::Response, reqwest::Error> {
        self.execute(request)
    }
}

impl<'a> ReqwestClient<'a> {
    pub fn new(host: &'a str) -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
            host,
        }
    }
}

impl HTTPClient for ReqwestClient<'_> {
    fn get(&self, endpoint: &'_ str) -> Result<TimedResponse, RequestError> {
        let request = build_get_request(&self.client, self.host, endpoint)?;
        let (response, response_time) = send_and_time_request(&self.client, request)?;

        Ok(TimedResponse::new(response.status().into(), response_time))
    }
    fn post<'a>(
        &self,
        endpoint: &'a str,
        body: &'a dyn SerializableInThread,
    ) -> Result<TimedResponse, RequestError> {
        let request = build_post_request(&self.client, self.host, endpoint, body)?;
        let (response, response_time) = send_and_time_request(&self.client, request)?;

        Ok(TimedResponse::new(response.status().into(), response_time))
    }
}
impl From<reqwest::StatusCode> for StatusCodeGroup {
    fn from(status_code: reqwest::StatusCode) -> Self {
        match status_code.as_u16() {
            100..=199 => Self::Information,
            200..=299 => Self::Success,
            300..=399 => Self::Redirect,
            400..=499 => Self::ClientError,
            500..=999 => Self::ServerError,
            _ => Self::Unknown,
        }
    }
}

fn build_post_request(
    client: &reqwest::blocking::Client,
    host: &'_ str,
    endpoint: &'_ str,
    body: &'_ dyn SerializableInThread,
) -> Result<reqwest::blocking::Request, reqwest::Error> {
    client
        .post(format!("{}/{}", host, endpoint))
        .json(body)
        .build()
}

fn build_get_request(
    client: &reqwest::blocking::Client,
    host: &'_ str,
    endpoint: &'_ str,
) -> Result<reqwest::blocking::Request, reqwest::Error> {
    client.get(format!("{}/{}", host, endpoint)).build()
}

fn send_and_time_request(
    client: &impl Client,
    request: reqwest::blocking::Request,
) -> Result<(reqwest::blocking::Response, Duration), reqwest::Error> {
    let request_send = Instant::now();
    let response = client.execute(request)?;
    let reponse_time = request_send.elapsed();

    Ok((response, reponse_time))
}

#[cfg(test)]
mod test {
    use serde::Serialize;

    use super::*;
    use crate::request::interface::to_millisecond;

    fn assert_request_same_method_url(
        request_1: &reqwest::blocking::Request,
        request_2: &reqwest::blocking::Request,
    ) {
        assert_eq!(request_1.method(), request_2.method());
        assert_eq!(request_1.url(), request_2.url(),);
    }

    #[test]
    fn test_build_get_request() {
        let request = build_get_request(
            &reqwest::blocking::Client::new(),
            "http://localhost",
            "test",
        )
        .unwrap();

        let expected_request = reqwest::blocking::Request::new(
            http::Method::GET,
            reqwest::Url::parse("http://localhost/test").unwrap(),
        );

        assert_request_same_method_url(&request, &expected_request)
    }

    #[derive(Serialize, Debug)]
    struct TestContent<'a> {
        message: &'a str,
    }

    #[test]
    fn test_build_post_request() {
        let request = build_post_request(
            &reqwest::blocking::Client::new(),
            "http://localhost",
            "test",
            &TestContent {
                message: "testing-message",
            },
        )
        .unwrap();

        let expected_request = reqwest::blocking::Request::new(
            http::Method::POST,
            reqwest::Url::parse("http://localhost/test").unwrap(),
        );

        assert_request_same_method_url(&request, &expected_request);
        assert_eq!(
            format!("{:?}", request.body().unwrap()),
            String::from("Body { kind: b\"{\\\"message\\\":\\\"testing-message\\\"}\" }")
        );
    }

    #[test]
    fn test_send_and_time_request() {
        let example_request = reqwest::blocking::Request::new(
            http::Method::POST,
            reqwest::Url::parse("http://localhost/test").unwrap(),
        );

        let mut mocked_client = MockClient::new();
        mocked_client.expect_execute().times(1).returning(|_| {
            std::thread::sleep(std::time::Duration::from_millis(100));
            Ok(reqwest::blocking::Response::from(http::Response::new(
                "body text",
            )))
        });

        let (actual_response, actual_response_time) =
            send_and_time_request(&mocked_client, example_request)
                .expect("Mock will return response.");

        assert!(to_millisecond(actual_response_time) > 100.0);
        assert_eq!(actual_response.text().unwrap(), String::from("body text"))
    }
}
