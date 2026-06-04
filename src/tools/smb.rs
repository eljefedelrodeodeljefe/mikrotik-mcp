use serde_json::{Value, json};

use crate::client::RouterosClient;
use crate::params::{AddSmbShareParams, AddSmbUserParams, SetSmbParams, SetSmbUserParams};

// ── Service ────────────────────────────────────────────────────────────────

pub async fn get_smb(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("ip/smb").await
}

pub async fn set_smb(client: &RouterosClient, p: &SetSmbParams) -> anyhow::Result<Value> {
    let mut body = json!({});
    if let Some(v) = &p.enabled {
        body["enabled"] = json!(v);
    }
    if let Some(v) = &p.domain {
        body["domain"] = json!(v);
    }
    if let Some(v) = &p.interfaces {
        body["interfaces"] = json!(v);
    }
    client.post("ip/smb/set", &body).await
}

// ── Shares ─────────────────────────────────────────────────────────────────

pub async fn list_shares(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("ip/smb/shares").await
}

pub async fn add_share(client: &RouterosClient, p: &AddSmbShareParams) -> anyhow::Result<Value> {
    let mut body = json!({"name": p.name, "directory": p.directory});
    if let Some(v) = &p.comment {
        body["comment"] = json!(v);
    }
    if let Some(v) = p.disabled {
        body["disabled"] = json!(v);
    }
    client.put("ip/smb/shares", &body).await
}

pub async fn remove_share(client: &RouterosClient, id: &str) -> anyhow::Result<()> {
    client.delete("ip/smb/shares", id).await
}

// ── Users ──────────────────────────────────────────────────────────────────

pub async fn list_users(client: &RouterosClient) -> anyhow::Result<Value> {
    client.get("ip/smb/users").await
}

pub async fn add_user(client: &RouterosClient, p: &AddSmbUserParams) -> anyhow::Result<Value> {
    // Default to read-only access — an SMB user with read-write should be opt-in.
    let read_only = p.read_only.unwrap_or(true);
    let mut body = json!({"name": p.name, "password": p.password, "read-only": read_only});
    if let Some(v) = p.disabled {
        body["disabled"] = json!(v);
    }
    client.put("ip/smb/users", &body).await
}

pub async fn set_user(client: &RouterosClient, p: &SetSmbUserParams) -> anyhow::Result<Value> {
    let mut body = json!({".id": p.id});
    if let Some(v) = &p.password {
        body["password"] = json!(v);
    }
    if let Some(v) = p.read_only {
        body["read-only"] = json!(v);
    }
    if let Some(v) = p.disabled {
        body["disabled"] = json!(v);
    }
    client.post("ip/smb/users/set", &body).await
}

pub async fn remove_user(client: &RouterosClient, id: &str) -> anyhow::Result<()> {
    client.delete("ip/smb/users", id).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn get_smb_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/ip/smb"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"enabled": "no", "interfaces": "all"})),
            )
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = get_smb(&client).await.unwrap();
        assert_eq!(result["interfaces"], "all");
    }

    #[tokio::test]
    async fn set_smb_posts_to_set_path() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/rest/ip/smb/set"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"interfaces": "bridge"})))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = SetSmbParams {
            enabled: Some("yes".into()),
            domain: None,
            interfaces: Some("bridge".into()),
        };
        let result = set_smb(&client, &p).await.unwrap();
        assert_eq!(result["interfaces"], "bridge");
    }

    #[tokio::test]
    async fn list_shares_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/ip/smb/shares"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {".id": "*1", "name": "usb1-part1", "directory": "usb1-part1"}
            ])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = list_shares(&client).await.unwrap();
        assert_eq!(result.as_array().unwrap()[0]["directory"], "usb1-part1");
    }

    #[tokio::test]
    async fn add_share_puts_to_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/rest/ip/smb/shares"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                ".id": "*2", "name": "media", "directory": "usb1-part1"
            })))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = AddSmbShareParams {
            name: "media".into(),
            directory: "usb1-part1".into(),
            comment: None,
            disabled: None,
        };
        let result = add_share(&client, &p).await.unwrap();
        assert_eq!(result["name"], "media");
    }

    #[tokio::test]
    async fn list_users_calls_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/ip/smb/users"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {".id": "*1", "name": "guest", "read-only": "true"}
            ])))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let result = list_users(&client).await.unwrap();
        assert_eq!(result.as_array().unwrap()[0]["name"], "guest");
    }

    #[tokio::test]
    async fn add_user_defaults_to_read_only() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/rest/ip/smb/users"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                ".id": "*2", "name": "files", "read-only": "true"
            })))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = AddSmbUserParams {
            name: "files".into(),
            password: "secret".into(),
            read_only: None,
            disabled: None,
        };
        let result = add_user(&client, &p).await.unwrap();
        assert_eq!(result["name"], "files");
    }

    #[tokio::test]
    async fn set_user_posts_to_set_path() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/rest/ip/smb/users/set"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({".id": "*2"})))
            .mount(&server)
            .await;

        let client = RouterosClient::for_test(&server.uri());
        let p = SetSmbUserParams {
            id: "*2".into(),
            password: None,
            read_only: Some(false),
            disabled: None,
        };
        let result = set_user(&client, &p).await.unwrap();
        assert_eq!(result[".id"], "*2");
    }
}
