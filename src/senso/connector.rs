use std::{
    fs::{File, OpenOptions},
    io::BufReader,
    time::Duration,
};

use anyhow::{anyhow, Result};

use cookie_store::CookieStore;
use ureq::{Agent, AgentBuilder, Request};

use super::urls;

const SMARTPHONE_ID: &str = "rustSenso";

pub struct Connector {
    agent: Agent,
    serial: String,
}

impl Connector {
    // TODO add cookie store
    pub fn new(serial: String) -> Result<Connector> {
        let enforce_https = !cfg!(feature = "local_url");

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("cookies.json")?;
        let read = BufReader::new(file);
        let my_store = CookieStore::load_json(read).unwrap();

        let agent = AgentBuilder::new()
            .timeout_read(Duration::from_secs(5))
            .timeout_write(Duration::from_secs(5))
            .cookie_store(my_store)
            .https_only(enforce_https)
            .build();
        Ok(Connector { agent, serial })
    }

    fn post(&self, url: &str) -> Request {
        self.agent
            .post(url)
            .set("Content-Type", "application/json; charset=utf-8")
            .set("Accept", "application/json")
            .set("Vaillant-Mobile-App", "senso v3.13 b469 (Android)")
    }

    fn token(&self, user: &str, pwd: &str) -> Result<String> {
        let resp = self.post(urls::NEW_TOKEN).send_json(ureq::json!({
            "smartphoneId": SMARTPHONE_ID,
            "username": user,
            "password": pwd,
        }))?;

        // only convert to JSON if Code 200
        let resp_json: serde_json::Value = match resp.status() {
            200 => resp.into_json()?,
            _ => return Err(anyhow!("Cannot get token")),
        };

        println!("{}", serde_json::to_string_pretty(&resp_json)?);
        Ok(resp_json["body"]["authToken"].to_string())
    }

    fn authenticate(&self, user: &str, token: &str) -> Result<()> {
        let resp = self.post(urls::AUTHENTICATE).send_json(ureq::json!({
            "smartphoneId": SMARTPHONE_ID,
            "username": user,
            "authToken": token,
        }))?;

        println!("{}", resp.status());

        let resp_json: serde_json::Value = resp.into_json()?;
        println!("{}", serde_json::to_string_pretty(&resp_json)?);

        Ok(())
    }

    pub fn login(&self, user: &str, pwd: &str) -> Result<()> {
        let token = self.token(user, pwd)?;
        self.authenticate(user, &token)?;
        Ok(())
    }
}
