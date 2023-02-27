use crate::request::definition::RequestDefinition;
use crate::request::reqwest_based::ReqwestClient;
use crate::tsp_specific::payload::SolveTspData;
use crate::LoadTestDefinition;

pub fn get_load_test<'a>(
    six_cities: &'a SolveTspData,
    fivteen_cities: &'a SolveTspData,
    twenty_nine_cities: &'a SolveTspData,
) -> LoadTestDefinition<ReqwestClient<'a>> {
    LoadTestDefinition::new(
        ReqwestClient::new("http://localhost/"),
        vec![
            RequestDefinition::GET { endpoint: "/alive" },
            RequestDefinition::POST {
                endpoint: "/tsp",
                to_json: six_cities,
            },
            RequestDefinition::POST {
                endpoint: "/tsp",
                to_json: fivteen_cities,
            },
            RequestDefinition::POST {
                endpoint: "/tsp",
                to_json: twenty_nine_cities,
            },
        ],
    )
}
