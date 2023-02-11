use erased_serde::Serialize;

pub enum RequestDefinition<'a> {
    POST {
        endpoint: &'a str,
        to_json: &'a dyn Serialize,
    },
    GET {
        endpoint: &'a str,
    },
}
