#[macro_use] extern crate nickel;
use prometheus_client::encoding::{EncodeLabelSet, EncodeLabelValue};
use prometheus_client::encoding::text::encode;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::registry::Registry;
use nickel::{Nickel, HttpRouter, middleware};
use std::io::Write;


#[derive(Debug, Clone, Hash, PartialEq, Eq, EncodeLabelSet)]
struct Labels {
    method: Method,
    path: String,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, EncodeLabelValue)]
enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
    CONNECT,
    TRACE,
}

fn main() {
    let mut server = Nickel::new();
    let mut registry = <Registry>::default();

    let http_requests = Family::<Labels, Counter>::default();
    registry.register(
        "http_requests_total",
        "Total number of HTTP requests made.",
        http_requests.clone(),
    );

    server.get("/metrics", middleware! {
        http_requests.get_or_create(&Labels {
            method: Method::GET,
            path: "/metrics".to_string(),
        }).inc();
    });

    let mut buffer = String::new();
    encode(&mut buffer, &registry).unwrap();

    server.listen("0.0.0.0:8989").unwrap();
}
