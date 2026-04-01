use std::time::Duration;

use anyhow::{Error, Result};
use reqwest::Client;
use serde_json::{Value, json};
use serde::Deserialize;

const PINCHTAB_URL: &str = "http://localhost:9867";

#[derive(Deserialize, Debug)]
pub struct PinchTabStartInstaceResponse {
    id: String,
    profileId: String,
    profileName: String,
    port: String,
    headless: bool,
    status: String,
    startTime: String
}

#[derive(Deserialize, Debug, Clone)]
pub struct PinchTabOpenTabResponse {
    pub tabId: String,
    pub title: String,
    pub url: String 
}

impl PinchTabOpenTabResponse {

    fn to_string(self, ) -> String {
        return String::from(format!("tabId: {}, title: {}, url: {}", self.tabId, self.title, self.url));
    }
}

#[derive(Clone)]
pub struct PinchTab {
    pub client: Client,
    pub instance_id: String,
}

impl PinchTab {

    pub async fn new() -> Result<PinchTab> {

        let client = Client::new();

        let resp = client
            .post(format!("{PINCHTAB_URL}/instances/launch"))
            .json(&json!({ "mode": "headed" }))
            .send()
            .await?
            .text()
            .await?;


        let json_resp: PinchTabStartInstaceResponse = serde_json::from_str(&resp).unwrap();

        println!("Instance created: {:#?}", json_resp);

        Ok(PinchTab {
            client,
            instance_id: json_resp.id,
        })
    }

    async fn post_request(
        &self,
        endpoint: &str,
        body: serde_json::Value,
    ) -> Result<String> {

        let resp = self.client
            .post(format!("{PINCHTAB_URL}/{endpoint}"))
            .json(&body)
            .send()
            .await?
            .text()
            .await?;

        Ok(resp)
    }

    async fn get_request(
        &self,
        endpoint: &str,
    ) -> Result<String> {

        let resp = self.client
            .get(format!("{PINCHTAB_URL}/{endpoint}"))
            .send()
            .await?
            .text()
            .await?;

        Ok(resp)
    }

    pub async fn open_tab(&self, url: Option<String>) -> Result<PinchTabOpenTabResponse> {

        let url = url.unwrap_or("about:blank".to_string());

        let resp = self.post_request(
            &format!("instances/{}/tabs/open", self.instance_id),
            json!({ "url": url }),
        )
        .await?;

        let resp: PinchTabOpenTabResponse = serde_json::from_str(&resp).unwrap();
        Ok(resp)
    }

    pub async fn navigate(&self, tab_id: String, url: String) -> Result<String> {

        self.post_request(
            &format!("tabs/{}/navigate", tab_id),
            json!({ "url": url }),
        )
        .await
    }

    pub async fn snapshot(&self, tab_id: String) -> Result<String> {

        self.get_request(
            &format!("tabs/{}/snapshot", tab_id)
        )
        .await
    }

    pub async fn text(&self, tab_id: String) -> Result<String> {

        self.get_request(
            &format!("tabs/{}/text", tab_id)
        )
        .await
    }

    pub async fn click(&self, tab_id: String, element_ref: String) -> Result<String> {

        self.post_request(
            &format!("tabs/{}/action", tab_id),
            json!({
                "kind": "click",
                "ref": element_ref
            }),
        )
        .await
    }

    pub async fn fill(
        &self,
        tab_id: String,
        element_ref: String,
        text: String,
    ) -> Result<String> {

        self.post_request(
            &format!("tabs/{}/action", tab_id),
            json!({
                "kind": "fill",
                "ref": element_ref,
                "text": text
            }),
        )
        .await
    }

    pub async fn screenshot(&self, tab_id: String) -> Result<String> {

        self.get_request(
            &format!("tabs/{}/screenshot?raw=false", tab_id)
        )
        .await
    }

    pub async fn pdf(&self, tab_id: String) -> Result<String> {

        self.get_request(
            &format!("tabs/{}/pdf?raw=false", tab_id)
        )
        .await
    }

    pub async fn close_tab(&self, tab_id: String) -> Result<String> {

        self.post_request(
            &format!("tabs/{}/close", tab_id),
            json!({}),
        )
        .await
    }

    pub async fn close(&self) -> Result<()> {

        let resp = self.post_request(
            &format!("instances/{}/stop", self.instance_id),
            serde_json::json!({}),
        )
        .await?;

        println!("{}", resp);

        Ok(())
    }
}