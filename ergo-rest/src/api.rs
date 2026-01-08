//! REST API for the services in Ergo ecosystem (node, explorer, etc.)

use reqwest::{header::CONTENT_TYPE, Client, RequestBuilder};

use crate::NodeConf;

pub mod node;
mod peer_discovery_internals;

fn set_req_headers(rb: RequestBuilder, node: NodeConf) -> RequestBuilder {
    let rb = rb
        .header("accept", "application/json")
        .header("api_key", node.get_node_api_header())
        .header(CONTENT_TYPE, "application/json");
    if let Some(t) = node.timeout {
        rb.timeout(t)
    } else {
        rb
    }
}

fn build_client() -> Result<Client, reqwest::Error> {
    let builder = reqwest::Client::builder();
    builder.build()
}
