use crossbeam_utils::thread;
use loadtest::command_line_interface::run::run;
use loadtest::load_test::core::ApiPerformanceCommunicator;
use loadtest::load_test::core::{run_loadtest_in_thread, KillSwitch};
use loadtest::load_test::performance_aggregator::PerformanceAggregator;
use loadtest::request::interface::HTTPClient;
use loadtest::tsp_specific::cities;
use loadtest::tsp_specific::example::get_load_test;
use loadtest::LoadTestDefinition;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    run()
}

fn previous_run() {
    let six_cities = cities::six();
    let fivteen_cities = cities::fiveteen();
    let twenty_nine_cities = cities::twenty_nine();
    run_loadtest(
        get_load_test(&six_cities, &fivteen_cities, &twenty_nine_cities),
        10,
        1_000_000_000,
    );
}

fn run_loadtest<R>(
    loadtest_definition: LoadTestDefinition<'_, R>,
    n_threads: usize,
    n_rounds: usize,
) where
    R: HTTPClient + Clone + Send,
{
    let kill_switch = KillSwitch::new();
    let performance_communicator = ApiPerformanceCommunicator::initialize();
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
        for _ in 1..n_rounds {
            if let Ok(received_response) = api_performance_receiver.recv() {
                performance.update(received_response);
                println!("{}", performance);
            }
        }
        kill_switch.activate();
        let _: Vec<std::thread::Result<()>> =
            users.into_iter().map(|worker| worker.join()).collect();
    })
    .unwrap();
}
