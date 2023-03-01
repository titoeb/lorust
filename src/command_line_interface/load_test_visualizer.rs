use crate::command_line_interface::plots::draw_plots;
use crossterm::event::{self, Event, KeyCode};
use std::time::Duration;
use tui::{backend::Backend, Terminal};

#[derive(Clone, Debug, PartialEq)]
pub struct VisualizerData {
    pub requests_per_second: Vec<(f64, f64)>,
    pub response_time: Vec<(f64, f64)>,
    pub users: Vec<(f64, f64)>,
    pub window: [f64; 2],
    pub window_length: f64,
}

#[derive(Debug)]
pub struct LoadtestVisualizer<'a, B>
where
    B: Backend,
{
    data: VisualizerData,
    terminal: &'a mut Terminal<B>,
    refresh_rate: Duration,
}

impl<'a, B> LoadtestVisualizer<'a, B>
where
    B: Backend,
{
    pub fn new(terminal: &'a mut Terminal<B>, refresh_rate: Duration) -> LoadtestVisualizer<'a, B> {
        LoadtestVisualizer {
            data: VisualizerData {
                requests_per_second: vec![],
                response_time: vec![],
                users: vec![],
                window: [0.0, 20.0],
                window_length: 20.0,
            },
            terminal,
            refresh_rate,
        }
    }
    pub fn update_request(&mut self, at_seconds: f64, request_rate: f64) {
        self.data
            .requests_per_second
            .push((at_seconds, request_rate));
        self.update_x_axis(
            self.data
                .requests_per_second
                .iter()
                .map(|(x, _)| *x)
                .fold(f64::NEG_INFINITY, f64::max),
            self.data
                .requests_per_second
                .iter()
                .map(|(x, _)| *x)
                .fold(f64::INFINITY, f64::min),
        );
    }
    pub fn update_x_axis(&mut self, max_time: f64, min_time: f64) {
        self.data.window[0] = min_time.max(max_time - self.data.window_length).ceil();
        self.data.window[1] = max_time.max(min_time + self.data.window_length).floor() + 2.0;
    }
    pub fn draw(&mut self) {
        self.terminal
            .draw(|frame| draw_plots(frame, &self.data))
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
