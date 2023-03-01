use loadtest::command_line_interface::run::run;
use loadtest::tsp_specific::cities;
use loadtest::tsp_specific::example::get_load_test;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let six_cities = cities::six();
    let fivteen_cities = cities::fiveteen();
    let twenty_nine_cities = cities::twenty_nine();

    run(get_load_test(
        &six_cities,
        &fivteen_cities,
        &twenty_nine_cities,
    ))
}
