use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, Method, Request, RequestBuilder};
use serde::Deserialize;
use sha2::Sha256;
use hmac::{Hmac, Mac};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::str;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio;
use url::Url;

#[derive(Clone)]
struct CoinexHttpClient {
    base_url: Url,
    coinex_key: String,
    coinex_secret: String,

    client: Client,
}

#[derive(Debug, Deserialize)]
struct CoinexResp<T> {
    code: i32,
    data: T,
    message: String,
}

impl CoinexHttpClient {
    fn new(
        base_url: &str,
        coinex_key: &str,
        coinex_secret: &str,
    ) -> Result<CoinexHttpClient, Box<dyn Error>> {
        Ok(CoinexHttpClient {
            base_url: Url::parse(base_url)?,
            coinex_key: coinex_key.to_string(),
            coinex_secret: coinex_secret.to_string(),
            client: Client::new(),
        })
    }

    async fn request(
        &self,
        method: Method,
        path: &str,
        params: Option<HashMap<&str, String>>,
        body: Option<HashMap<&str, serde_json::Value>>,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut url = self.base_url.join(path)?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();

        if let Some(params) = params {
            let query = serde_urlencoded::to_string(params)?;
            url.set_query(Some(&query));
        }

        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        headers.insert(
            "X-COINEX-KEY",
            HeaderValue::from_str(&self.coinex_key).unwrap(),
        );
        headers.insert(
            "X-COINEX-TIMESTAMP",
            HeaderValue::from_str(&now.to_string()).unwrap(),
        );

        let body_bytes = if let Some(body) = body {
            serde_json::to_vec(&body)?
        } else {
            Vec::new()
        };

        let query_string = match url.query() {
            Some(query) => format!("?{}", query),
            None => String::new(),
        };
        let path_and_query = format!("{}{}", url.path(), query_string);
        let sign = self.sign(
            &method.to_string(),
            &path_and_query,
            str::from_utf8(&body_bytes)?,
            now,
        )?;
        headers.insert("X-COINEX-SIGN", HeaderValue::from_str(&sign).unwrap());

        let request = Request::new(method, url);
        let request_builder = RequestBuilder::from_parts(self.client.clone(), request)
            .headers(headers)
            .body(body_bytes);

        let response = request_builder.send().await?;
        let status = response.status();
        let body_bytes = response.bytes().await?;

        if !status.is_success() {
            let body_str = String::from_utf8_lossy(&body_bytes);
            eprintln!(
                "request coinex api with wrong status: {} and body: {}",
                status, body_str
            );
            return Err("Request coinex api with wrong status".into());
        }

        let coinex_resp: CoinexResp<serde_json::Value> = serde_json::from_slice(&body_bytes)?;
        if coinex_resp.code != 0 {
            eprintln!(
                "request coinex api with wrong code: {} and message: {}",
                coinex_resp.code, coinex_resp.message
            );
            return Err("Request coinex api with wrong code".into());
        }

        Ok(serde_json::to_vec(&coinex_resp.data)?)
    }

    fn sign(
        &self,
        method: &str,
        path: &str,
        body: &str,
        timestamp: u128,
    ) -> Result<String, Box<dyn Error>> {
        let prepared_str = format!("{}{}{}{}", method, path, body, timestamp);
        type HmacSha256 = Hmac<Sha256>;

        let mut mac = HmacSha256::new_from_slice(self.coinex_secret.as_bytes())?;
        mac.update(prepared_str.as_bytes());
        let result = mac.finalize().into_bytes();
        Ok(hex::encode(&result))
    }

    async fn get(
        &self,
        path: &str,
        params: Option<HashMap<&str, String>>,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        self.request(Method::GET, path, params, None).await
    }

    async fn post(
        &self,
        path: &str,
        body: Option<HashMap<&str, serde_json::Value>>,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        self.request(Method::POST, path, None, body).await
    }
}

pub type SpotBalanceList = Vec<SpotBalance>;

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct SpotBalance {
    pub available: String,
    pub ccy: String,
    pub frozen: String,
}

impl CoinexHttpClient {
    pub async fn get_spot_balance(&self) -> Result<SpotBalanceList, Box<dyn Error>> {
        let response = self.get("/v2/assets/spot/balance", None).await?;
        let spot_balance_list: SpotBalanceList = serde_json::from_slice(&response).unwrap();
        Ok(spot_balance_list)
    }
}

pub type AccountList = Vec<Account>;

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct Account {
    pub sub_user_name: String,
    pub is_frozen: bool,
    pub is_authorized: bool,
    pub permissions: Vec<String>,
    pub balance_usd: String,
}

impl CoinexHttpClient {
    pub async fn get_account_list(
        &self,
        sub_user_name: &str,
        is_frozen: bool,
    ) -> Result<AccountList, Box<dyn Error>> {
        let mut params = HashMap::new();
        params.insert("is_frozen", is_frozen.to_string());
        params.insert("sub_user_name", sub_user_name.to_string());

        let response = self.get("/v2/account/subs", Some(params)).await.unwrap();
        let account_list: AccountList = serde_json::from_slice(&response).unwrap();
        Ok(account_list)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct SpotOrder {
    pub amount: String,
    pub base_fee: String,
    pub ccy: String,
    pub client_id: String,
    pub created_at: i64,
    pub discount_fee: String,
    pub filled_amount: String,
    pub last_fill_amount: String,
    pub last_fill_price: String,
    pub maker_fee_rate: String,
    pub market: String,
    pub market_type: String,
    pub order_id: i64,
    pub price: String,
    pub quote_fee: String,
    pub side: String,
    pub taker_fee_rate: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub unfilled_amount: String,
    pub updated_at: i64,
}

impl CoinexHttpClient {
    pub async fn put_spot_order(
        &self,
        market: &str,
        order_type: &str,
        side: &str,
        amount: &str,
        price: &str,
    ) -> Result<SpotOrder, Box<dyn Error>> {
        let mut body = HashMap::new();
        body.insert("market", serde_json::to_value(market).unwrap());
        body.insert("market_type", serde_json::to_value("spot").unwrap());
        body.insert("type", serde_json::to_value(order_type).unwrap());
        body.insert("side", serde_json::to_value(side).unwrap());
        body.insert("amount", serde_json::to_value(amount).unwrap());
        body.insert("price", serde_json::to_value(price).unwrap());

        let response = self.post("/v2/spot/order", Some(body)).await?;
        let spot_order: SpotOrder = serde_json::from_slice(&response).unwrap();
        Ok(spot_order)
    }
}

pub type SpotOrderList = Vec<SpotOrder>;

impl CoinexHttpClient {
    pub async fn list_pending_spot_order(
        &self,
        market: &str,
        side: &str,
        page: i32,
        limit: i32,
    ) -> Result<SpotOrderList, Box<dyn Error>> {
        let mut params = HashMap::new();
        params.insert("market", market.to_string());
        params.insert("market_type", "spot".to_string());
        params.insert("side", side.to_string());
        params.insert("page", page.to_string());
        params.insert("limit", limit.to_string());

        let response = self.get("/v2/spot/pending-order", Some(params)).await?;
        let spot_order: SpotOrderList = serde_json::from_slice(&response).unwrap();
        Ok(spot_order)
    }
}

impl CoinexHttpClient {
    pub async fn list_finished_spot_order(
        &self,
        market: &str,
        side: &str,
        page: i32,
        limit: i32,
    ) -> Result<SpotOrderList, Box<dyn Error>> {
        let mut params = HashMap::new();
        params.insert("market", market.to_string());
        params.insert("market_type", "spot".to_string());
        params.insert("side", side.to_string());
        params.insert("page", page.to_string());
        params.insert("limit", limit.to_string());

        let response = self.get("/v2/spot/finished-order", Some(params)).await?;
        println!("{:?}", std::str::from_utf8(&response).unwrap());
        let spot_order: SpotOrderList = serde_json::from_slice(&response).unwrap();
        Ok(spot_order)
    }
}

#[tokio::main]
async fn main() {
    let base_url = "";
    let coinex_key = "";
    let coinex_secret = "";

    let client = CoinexHttpClient::new(base_url, coinex_key, coinex_secret).unwrap();

    println!("{:?}", client.get_spot_balance().await.unwrap());
    println!("{:?}", client.get_account_list("", false).await.unwrap());
    println!(
        "{:?}",
        client
            .put_spot_order("BTCUSDT", "limit", "buy", "0.01", "30000")
            .await
            .unwrap()
    );
    println!(
        "{:?}",
        client
            .put_spot_order("BTCUSDT", "market", "buy", "0.01", "30000")
            .await
            .unwrap()
    );
    println!(
        "{:?}",
        client
            .list_pending_spot_order("BTCUSDT", "buy", 1, 10)
            .await
            .unwrap()
    );
    println!(
        "{:?}",
        client
            .list_finished_spot_order("BTCUSDT", "buy", 1, 10)
            .await
            .unwrap()
    );
}
