use crate::load_test::latest_response_time::ResponseTimestamp;
use crate::request::definition::RequestDefinition;
use crate::request::interface::to_millisecond;
use crate::request::interface::{HTTPClient, TimedResponse};
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Clone)]
pub struct LoadTestDefinition<'a, R>
where
    R: HTTPClient,
{
    connection: R,
    to_call: Vec<RequestDefinition<'a>>,
}

impl<'a, R> LoadTestDefinition<'a, R>
where
    R: HTTPClient,
{
    pub fn new(connection: R, to_call: Vec<RequestDefinition<'a>>) -> Self {
        Self {
            connection,
            to_call,
        }
    }
    pub fn run(&self) -> ApiPerformance {
        let responses: Vec<ResponseTiming> = self
            .to_call
            .iter()
            .map(|post_request_data| match *post_request_data {
                RequestDefinition::POST { endpoint, to_json } => (
                    self.connection.post(endpoint, to_json),
                    ResponseTimestamp::from(std::time::Instant::now()),
                ),
                RequestDefinition::GET { endpoint } => (
                    self.connection.get(endpoint),
                    ResponseTimestamp::from(std::time::Instant::now()),
                ),
            })
            .filter_map(|(response_result, response_time)| {
                response_result
                    .ok()
                    .map(|response| ResponseTiming::from((response, response_time)))
            })
            .collect();
        ApiPerformance::from(responses)
    }
    pub fn client(self) -> R {
        self.connection
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResponseTiming {
    timed_response: TimedResponse,
    response_timestamp: ResponseTimestamp,
}
impl From<(TimedResponse, ResponseTimestamp)> for ResponseTiming {
    fn from(response: (TimedResponse, ResponseTimestamp)) -> Self {
        Self {
            timed_response: response.0,
            response_timestamp: response.1,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApiPerformance {
    responses: Vec<ResponseTiming>,
}
impl ApiPerformance {
    pub fn new(responses: Vec<ResponseTiming>) -> Self {
        Self { responses }
    }
    pub fn get_response_timestamps(&self) -> Vec<ResponseTimestamp> {
        self.responses
            .iter()
            .map(|timed_response| timed_response.response_timestamp)
            .collect()
    }
    pub fn get_response_times(&self) -> Vec<Duration> {
        self.responses
            .iter()
            .map(|time_response| time_response.timed_response.response_time)
            .collect()
    }
    pub fn get_responses(self) -> Vec<TimedResponse> {
        self.responses
            .into_iter()
            .map(|response| response.timed_response)
            .collect()
    }
    pub fn average_response_time(&self) -> f64 {
        let sum_of_response_times = self
            .get_response_times()
            .iter()
            .map(|&duration| to_millisecond(duration))
            .sum::<f64>();

        match self.responses.len() {
            0 => 0.0,
            number_of_responses => sum_of_response_times / number_of_responses as f64,
        }
    }
}
impl From<Vec<ResponseTiming>> for ApiPerformance {
    fn from(timed_responses: Vec<ResponseTiming>) -> Self {
        Self {
            responses: timed_responses,
        }
    }
}

pub fn run_loadtest_in_thread<R>(
    kill_switch: KillSwitch,
    send_to_controller: Sender<ApiPerformance>,
    load_test_definition: LoadTestDefinition<'_, R>,
) where
    R: HTTPClient,
{
    loop {
        let api_performance = load_test_definition.run();
        send_to_controller.send(api_performance).unwrap();
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

#[derive(Debug)]
pub struct ApiPerformanceCommunicator {
    send_to_controller: mpsc::Sender<ApiPerformance>,
    receive_performance: mpsc::Receiver<ApiPerformance>,
}
impl ApiPerformanceCommunicator {
    pub fn initialize() -> Self {
        let (send_to_controller, receive_api_performance) = mpsc::channel::<ApiPerformance>();
        Self {
            send_to_controller,
            receive_performance: receive_api_performance,
        }
    }
    pub fn new_sender(&self) -> Sender<ApiPerformance> {
        self.send_to_controller.clone()
    }
    pub fn extract_receiver(self) -> mpsc::Receiver<ApiPerformance> {
        std::mem::drop(self.send_to_controller);
        self.receive_performance
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    mod test_load {
        use super::*;
        use crate::request::interface::HTTPClient;
        use crate::request::interface::SerializableInThread;
        use crate::request::interface::StatusCodeGroup;
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
                    StatusCodeGroup::Success,
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
                    StatusCodeGroup::Success,
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

            let load_test = LoadTestDefinition::new(
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
            let api_performance = load_test.run();
            let client = load_test.client();

            assert_eq!(
                api_performance.get_responses(),
                vec! {
                    TimedResponse::new(StatusCodeGroup::Success, Duration::from_millis(10)),
                    TimedResponse::new(StatusCodeGroup::Success, Duration::from_millis(50)),
                    TimedResponse::new(StatusCodeGroup::Success, Duration::from_millis(50)),
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
                    StatusCodeGroup::Success,
                    Duration::from_millis(10),
                ))
            }
            fn post<'a>(
                &self,
                _endpoint: &'_ str,
                _body: &'a dyn SerializableInThread,
            ) -> Result<TimedResponse, crate::request::interface::RequestError> {
                Ok(TimedResponse::new(
                    StatusCodeGroup::Success,
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
                mpsc::channel::<ApiPerformance>();
            let load_test = LoadTestDefinition::new(
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
            let mut api_performances = thread::scope(|s| {
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
            let firsts_responses = api_performances
                .pop()
                .expect("Has at least one entry after statement before was successful")
                .get_responses();
            assert_eq!(
                firsts_responses,
                vec![
                    TimedResponse::new(StatusCodeGroup::Success, Duration::from_millis(10)),
                    TimedResponse::new(StatusCodeGroup::Success, Duration::from_millis(50)),
                    TimedResponse::new(StatusCodeGroup::Success, Duration::from_millis(50)),
                ]
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
    mod api_performance_communicator {
        use super::*;
        use crate::request::interface::StatusCodeGroup;
        use std::sync::mpsc;
        use std::thread;
        use std::time::{Duration, Instant};

        fn send_single_message(send_to_main: mpsc::Sender<ApiPerformance>) {
            send_to_main
                .send(ApiPerformance {
                    responses: vec![ResponseTiming::from((
                        TimedResponse::new(StatusCodeGroup::Success, Duration::from_millis(200)),
                        ResponseTimestamp::from(Instant::now()),
                    ))],
                })
                .unwrap();
            std::mem::drop(send_to_main)
        }

        #[test]
        fn thread() {
            let api_performance_communicator = ApiPerformanceCommunicator::initialize();
            let mut message_senders = Vec::new();

            for _ in 0..2 {
                let send_to_main = api_performance_communicator.new_sender();
                message_senders.push(thread::spawn(|| send_single_message(send_to_main)));
            }

            let api_performance_receiver = api_performance_communicator.extract_receiver();
            let mut messages = Vec::new();
            while let Ok(message) = api_performance_receiver.recv() {
                messages.push(message.get_responses().pop().unwrap());
            }

            let _: Vec<std::thread::Result<()>> = message_senders
                .into_iter()
                .map(|message_sender| message_sender.join())
                .collect();

            assert_eq!(
                messages,
                vec![
                    TimedResponse::new(StatusCodeGroup::Success, Duration::from_millis(200),),
                    TimedResponse::new(StatusCodeGroup::Success, Duration::from_millis(200),)
                ]
            )
        }
    }

    mod api_performance {
        use super::*;
        use crate::request::interface::StatusCodeGroup;
        use std::time::Instant;

        #[test]
        fn extract_two_response_times() {
            let api_performance = ApiPerformance {
                responses: vec![
                    ResponseTiming::from((
                        TimedResponse::new(StatusCodeGroup::Success, Duration::from_millis(100)),
                        ResponseTimestamp::from(Instant::now()),
                    )),
                    ResponseTiming::from((
                        TimedResponse::new(StatusCodeGroup::Success, Duration::from_millis(90)),
                        ResponseTimestamp::from(Instant::now()),
                    )),
                    ResponseTiming::from((
                        TimedResponse::new(StatusCodeGroup::Success, Duration::from_millis(250)),
                        ResponseTimestamp::from(Instant::now()),
                    )),
                ],
            };

            assert_eq!(
                api_performance.get_response_times(),
                vec![
                    Duration::from_millis(100),
                    Duration::from_millis(90),
                    Duration::from_millis(250)
                ]
            )
        }
        #[test]
        fn extract_no_response_times() {
            let api_performance = ApiPerformance::new(vec![]);
            assert_eq!(api_performance.get_response_times(), vec![])
        }
        #[test]
        fn average_of_three_responses() {
            let api_performance = ApiPerformance {
                responses: vec![
                    ResponseTiming::from((
                        TimedResponse::new(StatusCodeGroup::Success, Duration::from_millis(180)),
                        ResponseTimestamp::from(Instant::now()),
                    )),
                    ResponseTiming::from((
                        TimedResponse::new(StatusCodeGroup::Success, Duration::from_millis(90)),
                        ResponseTimestamp::from(Instant::now()),
                    )),
                    ResponseTiming::from((
                        TimedResponse::new(StatusCodeGroup::Success, Duration::from_millis(270)),
                        ResponseTimestamp::from(Instant::now()),
                    )),
                ],
            };
            assert_eq!(api_performance.average_response_time(), 180.0)
        }
        #[test]
        fn average_response_time_of_no_responses() {
            let api_performance = ApiPerformance::new(vec![]);
            assert_eq!(api_performance.average_response_time(), 0.0)
        }
    }
}
