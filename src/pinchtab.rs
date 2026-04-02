use std::time::Duration;

use anyhow::{Error, Ok, Result};
use anyhow::{Context, anyhow};
use reqwest::Client;
use serde_json::{Value, json};
use serde::Deserialize;

const PINCHTAB_URL: &str = "http://localhost:9867";

#[derive(Deserialize, Debug)]
pub struct PinchTabInstaceResponse {
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

    pub fn to_string(&self, ) -> String {
        return String::from(format!("tabId: {}, title: {}, url: {}", self.tabId, self.title, self.url));
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct PinchTabTabResponse {
    pub id: String,
    pub r#type: String,
    pub url: String,
    pub title: String,
}

impl PinchTabTabResponse {

    pub fn to_string(&self, ) -> String {
        return String::from(format!("id: {}, instanceId: {}, title: {}, url: {}", self.id, self.r#type, self.title, self.url));
    }
}

#[derive(Deserialize, Debug)]
pub struct TabsResponse {
    pub tabs: Vec<PinchTabTabResponse>,
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


        let json_resp: PinchTabInstaceResponse = serde_json::from_str(&resp).unwrap();

        println!("Instance created: {:#?}", json_resp);

        let pinchtab = PinchTab {
            client,
            instance_id: json_resp.id,
        };

        pinchtab.wait_until_running().await?;

        Ok(pinchtab)
    }

    async fn wait_until_running(&self) -> Result<u8> {
        const MAX_TRIES: u8 = 30;
        const SLEEP_DURATION: Duration = Duration::from_millis(400);

        for attempt in 1..=MAX_TRIES {

            let instance = self.get_instance().await
                .context("Falha ao buscar status da instância")?;

            if instance.status.contains("running") {
                return Ok(attempt);
            }

            if attempt == MAX_TRIES {
                return Err(anyhow!("Excedeu o limite de {} tentativas", MAX_TRIES));
            }

            tokio::time::sleep(SLEEP_DURATION).await;
        }

        Err(anyhow!("Erro inesperado no loop de espera"))
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

    pub async fn get_instance(&self, ) -> Result<PinchTabInstaceResponse> {
        
        let resp = self.get_request(
            &format!("instances/{}", self.instance_id)
        ).await?;

        let instance_info: PinchTabInstaceResponse = serde_json::from_str(&resp)?;
        Ok(instance_info)
    }
    
    pub async fn open_tab(&self, url: Option<String>) -> Result<PinchTabOpenTabResponse> {

        let url = url.unwrap_or("about:blank".to_string());

        let resp = self.post_request(
            &format!("instances/{}/tabs/open", self.instance_id),
            json!({ "url": url }),
        )
        .await?;

        let resp: PinchTabOpenTabResponse = serde_json::from_str(&resp)?;
        Ok(resp)
    }

    pub async fn get_tabs(&self, ) -> Result<Vec<PinchTabTabResponse>> {
        
        let resp = self.get_request(
            &format!("instances/{}/tabs", self.instance_id)
        ).await?;

        let tabs: TabsResponse = serde_json::from_str(&resp).unwrap();

        Ok(tabs.tabs)
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

        Ok(())
    }
}