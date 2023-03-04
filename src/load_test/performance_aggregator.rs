use crate::load_test::core::ApiPerformance;
use crate::load_test::latest_response_time::LastResponses;
use std::time::Duration;

use super::latest_response_time::ResponseTimestamp;

const SMOOTHING: f64 = 0.5;

#[derive(Debug, Clone)]
pub struct PerformanceAggregator {
    requests_per_second: f64,
    response_time_in_milliseconds: f64,
    last_responses: LastResponses,
}
impl PerformanceAggregator {
    pub fn new(
        requests_per_second: f64,
        response_time_in_milliseconds: f64,
        last_responses: LastResponses,
    ) -> Self {
        Self {
            requests_per_second,
            response_time_in_milliseconds,
            last_responses,
        }
    }
    pub fn empty() -> Self {
        Self {
            requests_per_second: 0.0,
            response_time_in_milliseconds: 0.0,
            last_responses: LastResponses::new(Duration::from_secs(5)),
        }
    }
    pub fn reset(&mut self) {
        *self = Self::empty();
    }
    pub fn update(&mut self, performance: ApiPerformance) {
        self.update_response_time(performance.average_response_time());
        self.update_requests_per_second(performance.get_response_timestamps());
    }
    fn update_response_time(&mut self, average_new_response_time: f64) {
        self.response_time_in_milliseconds = smooth_exponentially(
            self.response_time_in_milliseconds,
            average_new_response_time,
            SMOOTHING,
        )
    }
    pub fn update_requests_per_second(&mut self, response_timestamps: Vec<ResponseTimestamp>) {
        for response_timestamp in response_timestamps {
            self.last_responses.push(response_timestamp)
        }
        self.requests_per_second = smooth_exponentially(
            self.requests_per_second,
            self.last_responses.requests_per_seconds(),
            SMOOTHING,
        );
    }
    pub fn request_per_second(&self) -> f64 {
        self.requests_per_second
    }
    pub fn response_time(&self) -> f64 {
        self.response_time_in_milliseconds
    }
}
impl std::fmt::Display for PerformanceAggregator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "requests_per_second: '{}', average_response_time: '{}'",
            self.requests_per_second, self.response_time_in_milliseconds,
        )
    }
}

fn smooth_exponentially(last: f64, current: f64, damping: f64) -> f64 {
    damping * last + (1.0 - damping) * current
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exponential_smooting_as_average() {
        assert_eq!(smooth_exponentially(1.0, 2.0, 0.5), 1.5)
    }

    // #[test]
    // fn update_performance_aggregator() {
    //     let mut performance_aggregator = PerformanceAggregator::empty();
    //     assert_eq!(performance_aggregator.requests_per_second, 0.0);
    //     assert_eq!(performance_aggregator.response_time_in_milliseconds, 0.0);

    //     performance_aggregator.update(ApiPerformance::new(
    //         vec![
    //             TimedResponse::new(
    //                 crate::request::interface::StatusCodeGroup::Success,
    //                 Duration::from_millis(100),
    //             ),
    //             TimedResponse::new(
    //                 crate::request::interface::StatusCodeGroup::Success,
    //                 Duration::from_millis(200),
    //             ),
    //         ],
    //         Duration::from_millis(100),
    //     ));
    //     assert_eq!(performance_aggregator.requests_per_second, 2.0);
    //     assert_eq!(performance_aggregator.response_time_in_milliseconds, 75.0);
    // }
}
