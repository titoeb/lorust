use crate::command_line_interface::load_test_visualizer::LoadtestVisualizer;
use crate::command_line_interface::plots::draw_plots;
use crate::load_test::core::ApiPerformanceCommunicator;
use crate::load_test::core::{run_loadtest_in_thread, KillSwitch};
use crate::load_test::performance_aggregator::PerformanceAggregator;
use crate::request::interface::to_seconds;
use crate::tsp_specific::cities;
use crate::tsp_specific::example::get_load_test;
use crossbeam_utils::thread;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
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

pub fn run() -> Result<(), Box<dyn Error>> {
    let mut terminal = setup_terminal()?;
    log_error(run_gui(
        &mut terminal,
        LoadtestVisualizer::new(),
        Duration::from_millis(100),
    ));
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

fn run_gui<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: LoadtestVisualizer,
    refresh_rate: Duration,
) -> io::Result<()> {
    let start_time = Instant::now();
    let kill_switch = KillSwitch::new();
    let performance_communicator = ApiPerformanceCommunicator::initialize();
    let n_threads = 2;
    let six_cities = cities::six();
    let fivteen_cities = cities::fiveteen();
    let twenty_nine_cities = cities::twenty_nine();
    let loadtest_definition = get_load_test(&six_cities, &fivteen_cities, &twenty_nine_cities);

    thread::scope(|s| {
        let mut users = Vec::new();
        for _ in 0..n_threads {
            let send_to_controller = performance_communicator.new_sender();

            let load_test_definition = loadtest_definition.clone();
            let kill_switch = kill_switch.clone();

            users.push(s.spawn(|_| {
                run_loadtest_in_thread(kill_switch, send_to_controller, load_test_definition)
            }))
        }
        let mut performance = PerformanceAggregator::empty();
        let api_performance_receiver = performance_communicator.extract_receiver();
        loop {
            terminal
                .draw(|frame| draw_plots(frame, &app))
                .expect("To be checked");
            if let Ok(received_response) = api_performance_receiver.recv() {
                performance.update(received_response);
                app.update_request(
                    to_seconds(start_time.elapsed()),
                    performance.request_per_second(),
                )
            }

            let timeout = refresh_rate
                .checked_sub(start_time.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));
            if crossterm::event::poll(timeout).expect("to be checked") {
                if let Event::Key(key) = event::read().expect("to_be_checked") {
                    if let KeyCode::Char('q') = key.code {
                        kill_switch.activate();
                        let _: Vec<std::thread::Result<()>> =
                            users.into_iter().map(|worker| worker.join()).collect();

                        break;
                    }
                }
            }
        }
    })
    .expect("to be checked.");
    Ok(())
}
