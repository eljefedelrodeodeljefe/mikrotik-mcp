use rmcp::model::{ErrorCode, ErrorData};

use crate::client::RouterosError;

/// Maps a tool failure to an MCP error. The message carries the whole
/// context chain, down to RouterOS's own `message`/`detail`. A 4xx from
/// RouterOS means the request was wrong (bad value, no such item), so the
/// agent gets INVALID_PARAMS and can correct it; anything else is INTERNAL.
pub fn tool_error(e: anyhow::Error) -> ErrorData {
    let message = format!("{e:#}");
    let code = match e.downcast_ref::<RouterosError>() {
        Some(RouterosError::Status { status, .. }) if status.is_client_error() => {
            ErrorCode::INVALID_PARAMS
        }
        Some(RouterosError::Transport(_)) => {
            tracing::warn!(error = %message, "RouterOS unreachable");
            ErrorCode::INTERNAL_ERROR
        }
        _ => ErrorCode::INTERNAL_ERROR,
    };
    ErrorData::new(code, message, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::RouterosClient;
    use anyhow::Context;
    use serde_json::{Value, json};
    use wiremock::matchers::method;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn router_answering(response: ResponseTemplate) -> MockServer {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(response)
            .mount(&server)
            .await;
        server
    }

    #[tokio::test]
    async fn routeros_refusal_is_invalid_params_with_its_message_and_detail() {
        let body = json!({"error": 400, "message": "Bad Request", "detail": "no such item"});
        let server = router_answering(ResponseTemplate::new(400).set_body_json(body)).await;
        let client = RouterosClient::for_test(&server.uri());
        let err = client
            .get::<Value>("ip/route")
            .await
            .context("step 1: list routes")
            .unwrap_err();
        let data = tool_error(err);
        assert_eq!(data.code, ErrorCode::INVALID_PARAMS);
        for want in ["step 1: list routes", "400", "Bad Request", "no such item"] {
            assert!(
                data.message.contains(want),
                "{want:?} missing: {}",
                data.message
            );
        }
    }

    #[tokio::test]
    async fn non_json_error_body_is_kept_short() {
        let server =
            router_answering(ResponseTemplate::new(500).set_body_string("x".repeat(1000))).await;
        let client = RouterosClient::for_test(&server.uri());
        let data = tool_error(client.get::<Value>("ip/route").await.unwrap_err());
        assert_eq!(data.code, ErrorCode::INTERNAL_ERROR);
        assert!(data.message.contains("500"), "{}", data.message);
        assert!(data.message.len() < 300, "{}", data.message.len());
    }

    #[tokio::test]
    async fn unreachable_router_is_internal_error() {
        // Port 9 (discard) on loopback: nothing listens there in CI.
        let client = RouterosClient::for_test("http://127.0.0.1:9");
        let data = tool_error(client.get::<Value>("ip/route").await.unwrap_err());
        assert_eq!(data.code, ErrorCode::INTERNAL_ERROR);
        assert!(
            data.message.contains("cannot reach RouterOS"),
            "{}",
            data.message
        );
    }
}
