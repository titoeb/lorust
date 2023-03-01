use crate::command_line_interface::load_test_visualizer::LoadtestVisualizer;

use crate::load_test::core::PerformanceCommunicator;
use crate::load_test::core::{run_loadtest_in_thread, KillSwitch};
use crate::load_test::performance_aggregator::PerformanceAggregator;
use crate::request::interface::{to_seconds, HTTPClient};

use crate::LoadTestDefinition;
use crossbeam_utils::thread;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

pub fn run<C>(definition: LoadTestDefinition<C>) -> Result<(), Box<dyn Error>>
where
    C: HTTPClient + Send + Clone,
{
    let mut terminal = setup_terminal()?;
    let mut loadtest = LoadTest::new(definition, 2, Duration::from_millis(100), &mut terminal);

    log_error(loadtest.run());
    restore_terminal(terminal)?;

    Ok(())
}

fn log_error(potential_error: Result<(), std::io::Error>) {
    if let Err(err) = potential_error {
        println!("{:?}", err)
    }
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>, std::io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}
fn restore_terminal(
    mut terminal: Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<(), std::io::Error> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()
}

#[derive(Debug)]
pub struct LoadTest<'a, R: HTTPClient, B>
where
    B: Backend,
{
    kill_switch: KillSwitch,
    performance_communicator: PerformanceCommunicator,
    definition: LoadTestDefinition<'a, R>,
    visualizer: LoadtestVisualizer<'a, B>,
    n_user: usize,
}
impl<'a, R, B> LoadTest<'a, R, B>
where
    R: HTTPClient + Clone + Send,
    B: Backend,
{
    fn new(
        definition: LoadTestDefinition<'a, R>,
        n_user: usize,
        refresh_rate: Duration,
        terminal: &'a mut Terminal<B>,
    ) -> Self {
        Self {
            kill_switch: KillSwitch::new(),
            performance_communicator: PerformanceCommunicator::initialize(),
            visualizer: LoadtestVisualizer::new(terminal, refresh_rate),
            definition,
            n_user,
        }
    }
    fn run(&mut self) -> io::Result<()> {
        let start_time = Instant::now();

        // TODO: Better seperation of concerns:
        // - drawing should go to visualizer
        // - performance aggregator to visualizer?
        thread::scope(|s| {
            let mut users = Vec::new();
            for _ in 0..self.n_user {
                let send_to_controller = self.performance_communicator.new_sender();

                let load_test_definition = self.definition.clone();
                let kill_switch = self.kill_switch.clone();

                users.push(s.spawn(|_| {
                    run_loadtest_in_thread(kill_switch, send_to_controller, load_test_definition)
                }))
            }
            let mut performance = PerformanceAggregator::empty();

            let api_performance_receiver =
                std::mem::take(&mut self.performance_communicator).extract_receiver();
            loop {
                self.visualizer.draw();
                if let Ok(received_response) = api_performance_receiver.recv() {
                    performance.update(received_response);
                    self.visualizer.update_request(
                        to_seconds(start_time.elapsed()),
                        performance.request_per_second(),
                    )
                }

                if self.visualizer.was_killed() {
                    break;
                }
            }

            self.kill_switch.activate();
            let _: Vec<std::thread::Result<()>> =
                users.into_iter().map(|worker| worker.join()).collect();
        })
        .expect("to be checked.");
        Ok(())
    }
}
