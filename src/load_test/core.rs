use crate::request::definition::RequestDefinition;
use crate::request::interface::{HTTPClient, TimedResponse};

pub struct LoadTest<'a, R>
where
    R: HTTPClient,
{
    connection: &'a R,
    to_call: Vec<RequestDefinition<'a>>,
}

impl<'a, R> LoadTest<'a, R>
where
    R: HTTPClient,
{
    pub fn new(connection: &'a R, to_call: Vec<RequestDefinition<'a>>) -> Self {
        Self {
            connection,
            to_call,
        }
    }
    pub fn run(&self) -> Vec<TimedResponse> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::interface::HTTPClient;
    use crate::request::interface::TimedResponse;
    use erased_serde::Serialize;
    use serde_json::json;
    use std::cell::RefCell;
    use std::time::Duration;

    struct TestHTTPClient {
        post_request_endpoints: RefCell<Vec<(String, String)>>,
        get_request_endpoints: RefCell<Vec<String>>,
    }

    impl TestHTTPClient {
        fn emtpy() -> Self {
            Self {
                post_request_endpoints: RefCell::new(vec![]),
                get_request_endpoints: RefCell::new(vec![]),
            }
        }
    }
    impl HTTPClient for TestHTTPClient {
        fn get(
            &self,
            endpoint: &'_ str,
        ) -> Result<TimedResponse, crate::request::interface::RequestError> {
            let mut get_request_endpoints = self.get_request_endpoints.borrow_mut();
            get_request_endpoints.push(endpoint.to_string());

            Ok(TimedResponse::new(
                "alive".to_string(),
                Duration::from_millis(10),
            ))
        }
        fn post<'a>(
            &self,
            endpoint: &'_ str,
            body: &'a dyn Serialize,
        ) -> Result<TimedResponse, crate::request::interface::RequestError> {
            let mut post_request_endpoints = self.post_request_endpoints.borrow_mut();
            post_request_endpoints.push((endpoint.to_string(), json!(body).to_string()));

            Ok(TimedResponse::new(
                "user created".to_string(),
                Duration::from_millis(50),
            ))
        }
    }

    #[derive(serde::Serialize)]
    struct TestPayload<'a> {
        name: &'a str,
    }

    #[test]
    fn full_loadtest() {
        let client = TestHTTPClient::emtpy();

        let steven = TestPayload { name: "Steven" };
        let sarah = TestPayload { name: "Sarah" };

        let load_test = LoadTest::new(
            &client,
            vec![
                RequestDefinition::GET {
                    endpoint: "/healthz",
                },
                RequestDefinition::POST {
                    endpoint: "/add-user",
                    to_json: &steven,
                },
                RequestDefinition::POST {
                    endpoint: "/add-user",
                    to_json: &sarah,
                },
            ],
        );
        let result = load_test.run();

        assert_eq!(
            result,
            vec! {
                TimedResponse::new("alive".to_string(), Duration::from_millis(10)),
                TimedResponse::new("user created".to_string(), Duration::from_millis(50)),
                TimedResponse::new("user created".to_string(), Duration::from_millis(50)),
            }
        );

        assert_eq!(
            client.post_request_endpoints.into_inner(),
            vec![
                (
                    String::from("/add-user"),
                    String::from("{\"name\":\"Steven\"}")
                ),
                (
                    String::from("/add-user"),
                    String::from("{\"name\":\"Sarah\"}")
                )
            ]
        );

        assert_eq!(
            client.get_request_endpoints.into_inner(),
            vec![String::from("/healthz"),]
        );
    }
}
