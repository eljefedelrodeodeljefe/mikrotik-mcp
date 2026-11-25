//! Diagnostics run on the router: ping, DNS resolve and fetch.

use serde_json::Value;

use super::Props;
use crate::client::RouterosClient;
use crate::params::{FetchFileParams, PingParams, ResolveDnsParams};

const DEFAULT_PING_COUNT: u32 = 4;
/// The REST call returns only when the ping is done, one second per request.
const MAX_PING_COUNT: u32 = 20;

/// `/ping`: one entry per echo request (`seq`, `host`, `time`, `ttl`,
/// `status`), the last carrying `sent`, `received`, `packet-loss` and the
/// round-trip times.
pub async fn ping(client: &RouterosClient, p: &PingParams) -> anyhow::Result<Value> {
    let count = p.count.unwrap_or(DEFAULT_PING_COUNT);
    anyhow::ensure!(
        (1..=MAX_PING_COUNT).contains(&count),
        "count must be 1..{MAX_PING_COUNT}"
    );
    let body = Props::new()
        .with("address", &p.address)
        .number("count", Some(count))
        .text("interface", p.interface.as_deref())
        .text("src-address", p.src_address.as_deref())
        .number("size", p.size)
        .text("routing-table", p.routing_table.as_deref())
        .into_body();
    client.post_read("ping", &body).await
}

/// `:resolve`, through the router's resolver or the given server.
pub async fn resolve(client: &RouterosClient, p: &ResolveDnsParams) -> anyhow::Result<Value> {
    let body = Props::new()
        .with("domain-name", &p.domain_name)
        .text("server", p.server.as_deref())
        .into_body();
    client.post_read("resolve", &body).await
}

/// `/tool fetch` to a file on the router. The call returns when the
/// download has finished or failed.
pub async fn fetch_file(client: &RouterosClient, p: &FetchFileParams) -> anyhow::Result<Value> {
    let body = Props::new()
        .with("url", &p.url)
        .with("dst-path", &p.dst_path)
        .with("output", "file")
        .text("check-certificate", p.check_certificate.as_deref())
        .into_body();
    client.post("tool/fetch", &body).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn ping_params(count: Option<u32>) -> PingParams {
        PingParams {
            address: "192.0.2.1".into(),
            count,
            interface: None,
            src_address: None,
            size: None,
            routing_table: None,
        }
    }

    #[tokio::test]
    async fn ping_sends_a_bounded_count() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/rest/ping"))
            .and(body_json(json!({"address": "192.0.2.1", "count": "4"})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {"seq": "0", "host": "192.0.2.1", "time": "1ms", "sent": "1", "received": "1"}
            ])))
            .expect(1)
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let out = ping(&client, &ping_params(None)).await.unwrap();
        assert_eq!(out[0]["received"], "1");
        for bad in [0, 21] {
            assert!(ping(&client, &ping_params(Some(bad))).await.is_err());
        }
    }

    #[tokio::test]
    async fn resolve_posts_the_domain_name() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/rest/resolve"))
            .and(body_json(json!({"domain-name": "router.example"})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ret": "192.0.2.1"})))
            .expect(1)
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = ResolveDnsParams {
            domain_name: "router.example".into(),
            server: None,
        };
        assert_eq!(resolve(&client, &p).await.unwrap()["ret"], "192.0.2.1");
    }

    #[tokio::test]
    async fn fetch_file_writes_to_dst_path() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/rest/tool/fetch"))
            .and(body_json(json!({
                "url": "http://192.0.2.10/routeros-7.20-arm.npk",
                "dst-path": "routeros-7.20-arm.npk",
                "output": "file"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {"status": "finished", "downloaded": "12288", "total": "12288"}
            ])))
            .expect(1)
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = FetchFileParams {
            url: "http://192.0.2.10/routeros-7.20-arm.npk".into(),
            dst_path: "routeros-7.20-arm.npk".into(),
            check_certificate: None,
        };
        let out = fetch_file(&client, &p).await.unwrap();
        assert_eq!(out[0]["status"], "finished");
    }
}
