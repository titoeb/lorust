use loadtest::request::definition::RequestDefinition;
use loadtest::request::reqwest_based::ReqwestConnection;
use loadtest::tsp_specific::cities;
use loadtest::LoadTest;

static HOST: &str = "http://localhost/";

fn main() {
    let six_cities = cities::six();
    let fivteen_cities = cities::fiveteen();
    let twenty_nine_cities = cities::twenty_nine();

    let load_test = LoadTest::new(
        ReqwestConnection::new(HOST),
        vec![
            RequestDefinition::GET { endpoint: "/alive" },
            RequestDefinition::POST {
                endpoint: "/tsp",
                to_json: &six_cities,
            },
            RequestDefinition::POST {
                endpoint: "/tsp",
                to_json: &fivteen_cities,
            },
            RequestDefinition::POST {
                endpoint: "/tsp",
                to_json: &twenty_nine_cities,
            },
        ],
    );

    let reponses = load_test.run();
    for response in reponses {
        println!("{:?}", response)
    }
}
