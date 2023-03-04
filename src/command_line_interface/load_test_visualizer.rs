use crate::load_test::core::ApiPerformance;
use crate::{
    command_line_interface::plots::draw_plots,
    load_test::performance_aggregator::PerformanceAggregator,
};
use crossterm::event::{self, Event, KeyCode};
use std::time::Duration;
use tui::{backend::Backend, Terminal};

#[derive(Clone, Debug, PartialEq)]
pub struct Data {
    pub requests_per_second: Vec<(f64, f64)>,
    pub response_time: Vec<(f64, f64)>,
    pub users: Vec<(f64, f64)>,
}
impl Data {
    fn empty() -> Self {
        Self {
            requests_per_second: vec![],
            response_time: vec![],
            users: vec![],
        }
    }

    fn min_request_per_second(&self) -> f64 {
        self.requests_per_second
            .iter()
            .map(|(x, _)| *x)
            .fold(f64::INFINITY, f64::min)
    }
    fn max_request_per_second(&self) -> f64 {
        self.requests_per_second
            .iter()
            .map(|(x, _)| *x)
            .fold(f64::NEG_INFINITY, f64::max)
    }
    fn reset(&mut self) {
        *self = Self::empty();
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub window: [f64; 2],
    pub window_length: f64,
}

#[derive(Debug)]
pub struct LoadtestVisualizer<'a, B>
where
    B: Backend,
{
    data: Data,
    settings: Settings,
    performance_aggregator: PerformanceAggregator,
    terminal: &'a mut Terminal<B>,
}

impl<'a, B> LoadtestVisualizer<'a, B>
where
    B: Backend,
{
    pub fn new(terminal: &'a mut Terminal<B>) -> LoadtestVisualizer<'a, B> {
        LoadtestVisualizer {
            data: Data::empty(),
            settings: Settings {
                window: [0.0, 20.0],
                window_length: 20.0,
            },
            performance_aggregator: PerformanceAggregator::empty(),
            terminal,
        }
    }
    pub fn reset(&'a mut self) {
        self.performance_aggregator.reset();
        self.data.reset();
    }
    pub fn update(&mut self, performance: ApiPerformance, at_seconds: f64) {
        self.performance_aggregator.update(performance);

        self.update_request(at_seconds, self.performance_aggregator.request_per_second());
        self.update_response_time(at_seconds, self.performance_aggregator.response_time());
    }
    pub fn update_request(&mut self, at_seconds: f64, request_rate: f64) {
        self.data
            .requests_per_second
            .push((at_seconds, request_rate));
    }
    pub fn update_response_time(&mut self, at_seconds: f64, response_time: f64) {
        self.data.response_time.push((at_seconds, response_time));
    }
    pub fn update_x_axis(&mut self, max_time: f64, min_time: f64) {
        self.settings.window[0] = min_time.max(max_time - self.settings.window_length).ceil();
        self.settings.window[1] =
            max_time.max(min_time + self.settings.window_length).floor() + 2.0;
    }
    pub fn draw(&mut self) {
        self.update_x_axis(
            self.data.max_request_per_second(),
            self.data.min_request_per_second(),
        );
        self.terminal
            .draw(|frame| draw_plots(frame, &self.settings, &self.data))
            .expect("To be checked");
    }
    pub fn was_killed(&self) -> bool {
        if crossterm::event::poll(Duration::from_secs(0)).expect("to be checked") {
            if let Event::Key(key) = event::read().expect("to_be_checked") {
                if let KeyCode::Char('q') = key.code {
                    return true;
                }
            }
        }
        false
    }
}
