use loadtest::cities;
use loadtest::load_test::{LoadTest, RequestData};
use loadtest::reqwest_connection::ReqwestConnection;

static HOST: &str = "http://localhost/";

fn main() {
    let six_cities = cities::six();
    let fivteen_cities = cities::fiveteen();
    let twenty_nine_cities = cities::twenty_nine();
    let load_test = LoadTest::new(
        ReqwestConnection::new(HOST),
        vec![
            RequestData::GET { endpoint: "/alive" },
            RequestData::POST {
                endpoint: "/tsp",
                to_json: &six_cities,
            },
            RequestData::POST {
                endpoint: "/tsp",
                to_json: &fivteen_cities,
            },
            RequestData::POST {
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
