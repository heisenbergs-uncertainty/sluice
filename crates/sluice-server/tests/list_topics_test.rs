//! Contract tests for the ListTopics RPC.
//!
//! Tests:
//! - T011: ListTopics returns lexicographically sorted known topics

mod common;

use sluice_server::proto::sluice::v1::{ListTopicsRequest, PublishRequest};
use std::collections::HashMap;

fn make_publish(topic: &str, payload: &[u8]) -> PublishRequest {
    PublishRequest {
        topic: topic.to_string(),
        payload: payload.to_vec(),
        attributes: HashMap::new(),
    }
}

/// T011: ListTopics returns known topics in lexicographic order.
#[tokio::test]
async fn test_list_topics_returns_sorted_topics() {
    let server = common::TestServer::start().await;
    let mut client = server.client().await;

    // Create topics by publishing to them.
    client
        .publish(make_publish("b-topic", b"b"))
        .await
        .expect("publish b-topic failed");
    client
        .publish(make_publish("a-topic", b"a"))
        .await
        .expect("publish a-topic failed");

    // Call ListTopics.
    let resp = client
        .list_topics(ListTopicsRequest {})
        .await
        .expect("list_topics failed")
        .into_inner();

    let names: Vec<String> = resp.topics.into_iter().map(|t| t.name).collect();

    assert!(names.contains(&"a-topic".to_string()));
    assert!(names.contains(&"b-topic".to_string()));

    // Server SHOULD return lexicographically sorted topics for stable UI ordering.
    assert_eq!(names, vec!["a-topic".to_string(), "b-topic".to_string()]);

    server.shutdown().await;
}
