use crate::request::definition::RequestDefinition;
use crate::request::interface::{HTTPClient, TimedResponse};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct LoadTest<'a, R>
where
    R: HTTPClient,
{
    connection: R,
    to_call: Vec<RequestDefinition<'a>>,
}

impl<'a, R> LoadTest<'a, R>
where
    R: HTTPClient,
{
    pub fn new(connection: R, to_call: Vec<RequestDefinition<'a>>) -> Self {
        Self {
            connection,
            to_call,
        }
    }
    pub fn run(&self) -> Vec<TimedResponse> {
        self.to_call
            .iter()
            .map(|post_request_data| match *post_request_data {
                RequestDefinition::POST { endpoint, to_json } => {
                    self.connection.post(endpoint, to_json)
                }
                RequestDefinition::GET { endpoint } => self.connection.get(endpoint),
            })
            .filter_map(|response_result| response_result.ok())
            .collect()
    }
    pub fn client(self) -> R {
        self.connection
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct APIPerformance {
    timed_responses: Vec<TimedResponse>,
}

impl From<Vec<TimedResponse>> for APIPerformance {
    fn from(timed_responses: Vec<TimedResponse>) -> Self {
        Self { timed_responses }
    }
}

pub fn run_loadtest_in_thread<R>(
    kill_switch: KillSwitch,
    send_api_performance: Sender<APIPerformance>,
    load_test: LoadTest<'_, R>,
) where
    R: HTTPClient,
{
    loop {
        let api_performance = load_test.run();
        send_api_performance.send(api_performance.into()).unwrap();
        if kill_switch.is_activated() {
            break;
        }
    }
}

#[derive(Default, Debug)]
pub struct KillSwitch {
    signal: Arc<Mutex<bool>>,
}

impl KillSwitch {
    pub fn new() -> Self {
        Self {
            signal: Arc::new(Mutex::new(false)),
        }
    }
    pub fn is_activated(&self) -> bool {
        match self.signal.lock() {
            Err(_) => true,
            Ok(should_be_killed) => *should_be_killed,
        }
    }

    pub fn activate(&self) {
        let mut execution_should_be_ended = self.signal.lock().unwrap();
        *execution_should_be_ended = true;
    }
}
impl Clone for KillSwitch {
    fn clone(&self) -> Self {
        Self {
            signal: self.signal.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    mod test_load {
        use super::*;
        use crate::request::interface::HTTPClient;
        use crate::request::interface::SerializableInThread;
        use crate::request::interface::TimedResponse;
        use crossbeam_utils::thread;
        use std::cell::RefCell;
        use std::sync::mpsc;
        use std::time::Duration;

        #[derive(Clone)]
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
                body: &'a dyn SerializableInThread,
            ) -> Result<TimedResponse, crate::request::interface::RequestError> {
                let mut post_request_endpoints = self.post_request_endpoints.borrow_mut();
                post_request_endpoints
                    .push((endpoint.to_string(), serde_json::to_string(body).unwrap()));

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
                client,
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
            let client = load_test.client();

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

        #[derive(Clone)]
        struct ThreadSafeTestClient {}

        impl ThreadSafeTestClient {
            fn new() -> Self {
                Self {}
            }
        }

        impl HTTPClient for ThreadSafeTestClient {
            fn get(
                &self,
                _endpoint: &'_ str,
            ) -> Result<TimedResponse, crate::request::interface::RequestError> {
                Ok(TimedResponse::new(
                    "alive".to_string(),
                    Duration::from_millis(10),
                ))
            }
            fn post<'a>(
                &self,
                _endpoint: &'_ str,
                _body: &'a dyn SerializableInThread,
            ) -> Result<TimedResponse, crate::request::interface::RequestError> {
                Ok(TimedResponse::new(
                    "user created".to_string(),
                    Duration::from_millis(50),
                ))
            }
        }

        #[test]
        fn run_threads() {
            let steven = TestPayload { name: "Steven" };
            let sarah = TestPayload { name: "Sarah" };
            let client = ThreadSafeTestClient::new();
            let kill_switch = KillSwitch::new();

            let (send_api_performance, receive_load_performance) =
                mpsc::channel::<APIPerformance>();
            let load_test = LoadTest::new(
                client,
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
            let api_performances = thread::scope(|s| {
                let mut users = Vec::new();
                for _ in 0..2 {
                    let send_api_performance = send_api_performance.clone();
                    let load_test = load_test.clone();
                    let kill_switch = kill_switch.clone();
                    users.push(s.spawn(|_| {
                        run_loadtest_in_thread(kill_switch, send_api_performance, load_test)
                    }))
                }

                let mut api_performances = Vec::new();
                for _ in 1..10 {
                    let received_response = receive_load_performance.recv().unwrap();
                    api_performances.push(received_response);
                }
                kill_switch.activate();
                let _: Vec<std::thread::Result<()>> =
                    users.into_iter().map(|worker| worker.join()).collect();

                api_performances
            })
            .unwrap();

            assert!(api_performances.len() > 2, "Two threads are running the tests continously until they receive the kill-signal. Therfore, we have at least two measurements.");
            assert_eq!(
                api_performances[0],
                APIPerformance {
                    timed_responses: vec![
                        TimedResponse::new(String::from("alive"), Duration::from_millis(10)),
                        TimedResponse::new(String::from("user created"), Duration::from_millis(50)),
                        TimedResponse::new(String::from("user created"), Duration::from_millis(50)),
                    ]
                }
            )
        }
    }

    mod test_kill_switch {
        use super::*;
        use std::thread;
        use std::time::Duration;

        fn run_and_wait_for_kill(kill_switch: KillSwitch) {
            loop {
                thread::sleep(Duration::from_millis(100));
                if kill_switch.is_activated() {
                    break;
                }
            }
        }

        fn run_threads_then_kill(n_threads: usize) {
            let kill_switch = KillSwitch::new();

            let mut workers = Vec::new();
            for _ in 0..n_threads {
                let kill_switch = kill_switch.clone();
                workers.push(thread::spawn(|| {
                    run_and_wait_for_kill(kill_switch);
                }));
            }
            kill_switch.activate();

            let _: Vec<std::thread::Result<()>> =
                workers.into_iter().map(|worker| worker.join()).collect();
        }
        #[test]
        fn zero_threads() {
            run_threads_then_kill(0);
        }
        #[test]
        fn one_threads() {
            run_threads_then_kill(0);
        }

        #[test]
        fn three_threads() {
            run_threads_then_kill(3);
        }
    }
}
