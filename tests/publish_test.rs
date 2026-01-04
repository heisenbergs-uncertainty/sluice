//! Contract tests for the Publish RPC.
//!
//! Tests:
//! - T016: Valid publish returns message_id, sequence, timestamp
//! - T017: Publish to new topic creates it automatically

mod common;

use sluice::proto::sluice::v1::PublishRequest;
use std::collections::HashMap;

/// Helper to create a publish request.
fn make_publish_request(topic: &str, payload: &[u8]) -> PublishRequest {
    PublishRequest {
        topic: topic.to_string(),
        payload: payload.to_vec(),
        attributes: HashMap::new(),
    }
}

/// T016: Valid publish returns message_id, sequence, timestamp.
#[tokio::test]
async fn test_publish_returns_valid_response() {
    let server = common::TestServer::start().await;
    let mut client = server.client().await;

    let request = make_publish_request("test-topic", b"hello world");
    let response = client.publish(request).await.expect("publish failed");
    let resp = response.into_inner();

    // Verify message_id is a valid UUID
    assert!(!resp.message_id.is_empty(), "message_id should not be empty");
    assert!(
        uuid::Uuid::parse_str(&resp.message_id).is_ok(),
        "message_id should be valid UUID: {}",
        resp.message_id
    );

    // Verify sequence is assigned (first message should be 1)
    assert_eq!(resp.sequence, 1, "first message should have sequence 1");

    // Verify timestamp is reasonable (after year 2024)
    assert!(
        resp.timestamp > 1704067200000,
        "timestamp should be after 2024: {}",
        resp.timestamp
    );

    server.shutdown().await;
}

/// T016: Multiple publishes increment sequence.
#[tokio::test]
async fn test_publish_sequence_increments() {
    let server = common::TestServer::start().await;
    let mut client = server.client().await;

    // Publish three messages
    let resp1 = client
        .publish(make_publish_request("seq-topic", b"msg1"))
        .await
        .expect("publish 1 failed")
        .into_inner();
    let resp2 = client
        .publish(make_publish_request("seq-topic", b"msg2"))
        .await
        .expect("publish 2 failed")
        .into_inner();
    let resp3 = client
        .publish(make_publish_request("seq-topic", b"msg3"))
        .await
        .expect("publish 3 failed")
        .into_inner();

    assert_eq!(resp1.sequence, 1);
    assert_eq!(resp2.sequence, 2);
    assert_eq!(resp3.sequence, 3);

    // Each message should have unique IDs
    assert_ne!(resp1.message_id, resp2.message_id);
    assert_ne!(resp2.message_id, resp3.message_id);

    server.shutdown().await;
}

/// T017: Publish to new topic creates it automatically.
#[tokio::test]
async fn test_publish_creates_topic_automatically() {
    let server = common::TestServer::start().await;
    let mut client = server.client().await;

    // Publish to a topic that doesn't exist
    let request = make_publish_request("brand-new-topic", b"first message");
    let response = client.publish(request).await.expect("publish failed");
    let resp = response.into_inner();

    // Should succeed and get sequence 1
    assert_eq!(resp.sequence, 1, "new topic should start at sequence 1");

    // Publish another message to same topic - should work
    let request2 = make_publish_request("brand-new-topic", b"second message");
    let response2 = client.publish(request2).await.expect("publish 2 failed");
    let resp2 = response2.into_inner();

    assert_eq!(resp2.sequence, 2, "second message should be sequence 2");

    server.shutdown().await;
}

/// T017: Separate topics have independent sequences.
#[tokio::test]
async fn test_separate_topics_have_independent_sequences() {
    let server = common::TestServer::start().await;
    let mut client = server.client().await;

    // Publish to topic A
    let resp_a1 = client
        .publish(make_publish_request("topic-a", b"a1"))
        .await
        .expect("publish failed")
        .into_inner();
    let resp_a2 = client
        .publish(make_publish_request("topic-a", b"a2"))
        .await
        .expect("publish failed")
        .into_inner();

    // Publish to topic B
    let resp_b1 = client
        .publish(make_publish_request("topic-b", b"b1"))
        .await
        .expect("publish failed")
        .into_inner();

    // Topic A should be at sequence 2
    assert_eq!(resp_a1.sequence, 1);
    assert_eq!(resp_a2.sequence, 2);

    // Topic B should start at sequence 1 (independent)
    assert_eq!(resp_b1.sequence, 1);

    server.shutdown().await;
}

/// Test publish with attributes (W3C trace context).
#[tokio::test]
async fn test_publish_with_attributes() {
    let server = common::TestServer::start().await;
    let mut client = server.client().await;

    let mut attributes = HashMap::new();
    attributes.insert(
        "traceparent".to_string(),
        "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01".to_string(),
    );
    attributes.insert("custom-header".to_string(), "custom-value".to_string());

    let request = PublishRequest {
        topic: "attributed-topic".to_string(),
        payload: b"message with attributes".to_vec(),
        attributes,
    };

    let response = client.publish(request).await.expect("publish failed");
    let resp = response.into_inner();

    assert_eq!(resp.sequence, 1);
    assert!(!resp.message_id.is_empty());

    server.shutdown().await;
}

/// Test publish validation - empty topic should fail.
#[tokio::test]
async fn test_publish_empty_topic_fails() {
    let server = common::TestServer::start().await;
    let mut client = server.client().await;

    let request = make_publish_request("", b"message");
    let result = client.publish(request).await;

    assert!(result.is_err(), "empty topic should be rejected");
    let status = result.unwrap_err();
    assert_eq!(
        status.code(),
        tonic::Code::InvalidArgument,
        "should return INVALID_ARGUMENT"
    );

    server.shutdown().await;
}

/// Test publish validation - topic name too long should fail.
#[tokio::test]
async fn test_publish_topic_too_long_fails() {
    let server = common::TestServer::start().await;
    let mut client = server.client().await;

    let long_topic = "a".repeat(256);
    let request = make_publish_request(&long_topic, b"message");
    let result = client.publish(request).await;

    assert!(result.is_err(), "topic > 255 chars should be rejected");
    let status = result.unwrap_err();
    assert_eq!(
        status.code(),
        tonic::Code::InvalidArgument,
        "should return INVALID_ARGUMENT"
    );

    server.shutdown().await;
}

/// Test publish validation - invalid characters in topic should fail.
#[tokio::test]
async fn test_publish_topic_invalid_chars_fails() {
    let server = common::TestServer::start().await;
    let mut client = server.client().await;

    // Try various invalid characters
    for invalid_topic in &["topic/name", "topic:name", "topic name", "topic@name"] {
        let request = make_publish_request(invalid_topic, b"message");
        let result = client.publish(request).await;

        assert!(
            result.is_err(),
            "topic '{}' with invalid chars should be rejected",
            invalid_topic
        );
        let status = result.unwrap_err();
        assert_eq!(
            status.code(),
            tonic::Code::InvalidArgument,
            "should return INVALID_ARGUMENT for '{}'",
            invalid_topic
        );
    }

    server.shutdown().await;
}

/// Test valid topic characters are accepted.
#[tokio::test]
async fn test_publish_topic_valid_chars_accepted() {
    let server = common::TestServer::start().await;
    let mut client = server.client().await;

    // Valid topic names with allowed characters
    for valid_topic in &[
        "simple",
        "with-dashes",
        "with_underscores",
        "with.dots",
        "Mixed-Case_123.topic",
        "numbers123",
    ] {
        let request = make_publish_request(valid_topic, b"message");
        let result = client.publish(request).await;

        assert!(
            result.is_ok(),
            "topic '{}' should be accepted, got error: {:?}",
            valid_topic,
            result.err()
        );
    }

    server.shutdown().await;
}
