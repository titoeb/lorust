use crate::request::interface::SerializableInThread;

#[derive(Clone, Debug)]
pub enum RequestDefinition<'a> {
    POST {
        endpoint: &'a str,
        to_json: &'a dyn SerializableInThread,
    },
    GET {
        endpoint: &'a str,
    },
}
