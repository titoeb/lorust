#[derive(Debug, Clone, PartialEq, Default)]
pub struct LoadtestVisualizer {
    pub requests_per_second: Vec<(f64, f64)>,
    pub response_time: Vec<(f64, f64)>,
    pub users: Vec<(f64, f64)>,
    pub window: [f64; 2],
    pub window_length: f64,
}

impl LoadtestVisualizer {
    pub fn new() -> LoadtestVisualizer {
        LoadtestVisualizer {
            requests_per_second: vec![],
            response_time: vec![],
            users: vec![],
            window: [0.0, 20.0],
            window_length: 20.0,
        }
    }
    pub fn update_request(&mut self, at_seconds: f64, request_rate: f64) {
        self.requests_per_second.push((at_seconds, request_rate));
        self.update_x_axis(
            self.requests_per_second
                .iter()
                .map(|(x, _)| *x)
                .fold(f64::NEG_INFINITY, f64::max),
            self.requests_per_second
                .iter()
                .map(|(x, _)| *x)
                .fold(f64::INFINITY, f64::min),
        );
    }
    pub fn update_x_axis(&mut self, max_time: f64, min_time: f64) {
        self.window[0] = min_time.max(max_time - self.window_length).ceil();
        self.window[1] = max_time.max(min_time + self.window_length).floor() + 2.0;
    }
}
