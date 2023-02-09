use loadtest::cities;
use loadtest::request::{RequestError, Response, SendRequest};
use loadtest::request_data::SolveTspData;
use loadtest::reqwest_connection::ReqwestConnection;

static HOST: &str = "http://localhost/";

fn main() {
    let request = ReqwestConnection::new(HOST);
    let six_cities = cities::six();
    let fivteen_cities = cities::fiveteen();
    let twenty_nine_cities = cities::twenty_nine();

    println!("{}", call_alive(&request).unwrap());
    println!("{}", call_tsp(&request, &six_cities).unwrap());
    println!("{}", call_tsp(&request, &fivteen_cities).unwrap());
    println!("{}", call_tsp(&request, &twenty_nine_cities).unwrap());
}
fn call_alive(request: &impl SendRequest) -> Result<Response, RequestError> {
    request.get("/alive")
}
fn call_tsp(request: &impl SendRequest, tsp: &SolveTspData) -> Result<Response, RequestError> {
    request.post("/tsp", tsp)
}
