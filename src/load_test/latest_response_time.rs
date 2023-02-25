use crate::request::interface::to_seconds;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::fmt::Debug;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct ResponseTimestamp(Instant);
impl Ord for ResponseTimestamp {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.cmp(&self.0)
    }
}
impl ResponseTimestamp {
    pub fn elapsed(&self) -> Duration {
        self.0.elapsed()
    }
}
impl PartialOrd for ResponseTimestamp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl ResponseTimestamp {
    pub fn now() -> Self {
        Self(Instant::now())
    }
}
impl From<Instant> for ResponseTimestamp {
    fn from(instant: Instant) -> Self {
        Self(instant)
    }
}

#[derive(Debug, Clone)]
pub struct LastResponses {
    max_duration: Duration,
    heap: BinaryHeap<ResponseTimestamp>,
}
impl LastResponses {
    pub fn new(max_duration: Duration) -> Self {
        LastResponses {
            max_duration,
            heap: BinaryHeap::new(),
        }
    }
    pub fn push(&mut self, response_time: ResponseTimestamp) {
        self.removed_outdated_responses();
        if response_time.elapsed() < self.max_duration {
            self.heap.push(response_time);
        }
    }
    pub fn peek(&mut self) -> Option<&ResponseTimestamp> {
        self.removed_outdated_responses();
        self.heap.peek()
    }
    pub fn requests_per_seconds(&self) -> f64 {
        self.heap.len() as f64 / to_seconds(self.max_duration)
    }
    fn removed_outdated_responses(&mut self) {
        while let Some(response_time) = self.heap.peek() {
            if response_time.elapsed() >= self.max_duration {
                self.heap.pop();
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    mod response_time {

        use super::*;

        #[test]
        fn ordering_is_correct() {
            let earliest =
                ResponseTimestamp::from(Instant::now() - std::time::Duration::from_secs(2));
            let now = ResponseTimestamp::now();
            let later = ResponseTimestamp::from(Instant::now() + std::time::Duration::from_secs(2));

            assert!(earliest > now);
            assert!(earliest > later);
            assert!(earliest == earliest);
            assert!(now > later);
            assert!(now < earliest);
            assert!(now == now);
            assert!(later < now);
            assert!(later < earliest);
            assert!(later == later);
        }
    }

    mod response_time_heap {
        use super::*;

        #[test]
        fn earliest_time_stamp_is_on_top() {
            let earliest =
                ResponseTimestamp::from(Instant::now() - std::time::Duration::from_secs(2));
            let now = ResponseTimestamp::now();
            let later = ResponseTimestamp::from(Instant::now() + std::time::Duration::from_secs(2));

            let mut last_reponse_times = LastResponses::new(Duration::from_secs(100));

            last_reponse_times.push(now.clone());
            assert_eq!(last_reponse_times.peek(), Some(&now));

            last_reponse_times.push(later);
            assert_eq!(last_reponse_times.peek(), Some(&now));

            last_reponse_times.push(earliest.clone());
            assert_eq!(last_reponse_times.peek(), Some(&earliest));
        }
        #[test]
        fn peak_does_not_give_outdated_response_time() {
            let earliest =
                ResponseTimestamp::from(Instant::now() - std::time::Duration::from_secs(2));
            let mut heap = BinaryHeap::new();
            heap.push(earliest);

            let mut last_reponse_times = LastResponses {
                heap,
                max_duration: Duration::from_secs(1),
            };
            assert_eq!(last_reponse_times.peek(), None);
        }
        #[test]
        fn peak_gives_response_time_within_max_duration() {
            let earliest =
                ResponseTimestamp::from(Instant::now() - std::time::Duration::from_secs(1));
            let mut heap = BinaryHeap::new();
            heap.push(earliest.clone());

            let mut last_reponse_times = LastResponses {
                heap,
                max_duration: Duration::from_secs(2),
            };
            assert_eq!(last_reponse_times.peek(), Some(&earliest));
        }
        #[test]
        fn outdated_timestamp_is_not_inserted() {
            let earliest =
                ResponseTimestamp::from(Instant::now() - std::time::Duration::from_secs(2));
            let mut last_reponse_times = LastResponses::new(Duration::from_secs(1));
            last_reponse_times.push(earliest);
            assert!(last_reponse_times.heap.is_empty());
        }
        #[test]
        fn timestamp_within_range_is_inserted() {
            let earliest =
                ResponseTimestamp::from(Instant::now() - std::time::Duration::from_secs(1));
            let mut last_reponse_times = LastResponses::new(Duration::from_secs(2));
            last_reponse_times.push(earliest);
            assert!(last_reponse_times.heap.len() == 1);
        }
        #[test]
        fn response_time_heap_is_cleaned_of_outdated_responses() {
            let earliest =
                ResponseTimestamp::from(Instant::now() - std::time::Duration::from_secs(10));
            let still_to_early =
                ResponseTimestamp::from(Instant::now() - std::time::Duration::from_secs(5));

            let late_enough =
                ResponseTimestamp::from(Instant::now() - std::time::Duration::from_secs(3));
            let now = ResponseTimestamp::from(Instant::now());

            let mut heap = BinaryHeap::new();
            heap.push(earliest);
            heap.push(still_to_early);
            heap.push(late_enough.clone());
            heap.push(now.clone());

            let mut last_reponse_times = LastResponses {
                heap,
                max_duration: Duration::from_secs(4),
            };
            last_reponse_times.removed_outdated_responses();

            assert_eq!(last_reponse_times.heap.pop(), Some(late_enough));
            assert_eq!(last_reponse_times.heap.pop(), Some(now));
            assert_eq!(last_reponse_times.heap.pop(), None);
        }
        #[test]
        fn three_request_in_last_two_seconds() {
            let mut last_reponse_times = LastResponses::new(Duration::from_secs(2));

            last_reponse_times.push(ResponseTimestamp::now());
            last_reponse_times.push(ResponseTimestamp::from(
                Instant::now() - std::time::Duration::from_secs(1),
            ));
            last_reponse_times.push(ResponseTimestamp::from(
                Instant::now() - std::time::Duration::from_millis(1500),
            ));
            assert_eq!(last_reponse_times.requests_per_seconds(), 1.5)
        }
        #[test]
        fn no_request_in_last_period() {
            let last_reponse_times = LastResponses::new(Duration::from_secs(2));
            assert_eq!(last_reponse_times.requests_per_seconds(), 0.0)
        }
    }
}
