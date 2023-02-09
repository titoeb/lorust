use loadtest::request::{RequestError, ReqwestConnection, SendRequest};
use serde::Deserialize;
use serde::Serialize;

static HOST: &str = "http://localhost/";

#[tokio::main]
async fn main() {
    let request = ReqwestConnection::new(HOST);
    println!("{}", call_alive(&request).await.unwrap());
    let six_cities = SolveTspData {
        distances: vec![
            vec![0.0, 64.0, 378.0, 519.0, 434.0, 200.0],
            vec![64.0, 0.0, 318.0, 455.0, 375.0, 164.0],
            vec![378.0, 318.0, 0.0, 170.0, 265.0, 344.0],
            vec![519.0, 455.0, 170.0, 0.0, 223.0, 428.0],
            vec![434.0, 375.0, 265.0, 223.0, 0.0, 273.0],
            vec![200.0, 164.0, 344.0, 428.0, 273.0, 0.0],
        ],
        n_generations: 600,
    };
    println!("{}", call_tsp(&request, &six_cities).await.unwrap());
}

#[derive(Serialize, Deserialize)]
struct SolveTspData {
    distances: Vec<Vec<f64>>,
    n_generations: usize,
}

async fn call_alive(request: &impl SendRequest) -> Result<String, RequestError> {
    request.get("/alive").await
}
async fn call_tsp(request: &impl SendRequest, tsp: &SolveTspData) -> Result<String, RequestError> {
    request.post("/tsp", tsp).await
}
