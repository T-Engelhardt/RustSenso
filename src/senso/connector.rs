use std::time::Duration;

use anyhow::{bail, Result};

use ureq::{Agent, AgentBuilder, Request};

use super::urls;

const SMARTPHONE_ID: &str = "rustSenso";

pub struct Connector {
    agent: Agent,
    serial: String,
    login: bool,
}

impl Connector {
    pub fn new(serial: String) -> Result<Connector> {
        let enforce_https = !cfg!(feature = "local_url");

        let agent = AgentBuilder::new()
            .timeout_read(Duration::from_secs(5))
            .timeout_write(Duration::from_secs(5))
            .https_only(enforce_https)
            .build();
        Ok(Connector {
            agent,
            serial,
            login: false,
        })
    }

    fn default_header(&self, req: Request) -> Request {
        req.set("Content-Type", "application/json; charset=UTF-8")
            .set("Accept", "application/json")
            .set("Vaillant-Mobile-App", "senso v3.13 b469 (Android)")
    }

    fn token(&self, user: &str, pwd: &str) -> Result<String> {
        let resp = self
            .default_header(self.agent.post(urls::NEW_TOKEN))
            .send_json(ureq::json!({
                "smartphoneId": SMARTPHONE_ID,
                "username": user,
                "password": pwd,
            }))?;

        // only convert to JSON if Code 200
        let resp_json: serde_json::Value = match resp.status() {
            200 => resp.into_json()?,
            _ => bail!("Cannot get token"),
        };

        match resp_json["body"]["authToken"].as_str() {
            Some(auth_token) => Ok(auth_token.into()),
            None => bail!("Can't convert/find Json field authToken"),
        }
    }

    fn authenticate(&self, user: &str, token: &str) -> Result<()> {
        let resp = self
            .default_header(self.agent.post(urls::AUTHENTICATE))
            .send_json(ureq::json!({
                "smartphoneId": SMARTPHONE_ID,
                "username": user,
                "authToken": token,
            }))?;

        match resp.status() {
            200 => Ok(()),
            c => bail!("Unable to authenticate | status code: {}", c),
        }
    }

    fn login_ok(&mut self) {
        self.login = true;
    }
}

// PUBLIC INTERFACE //
impl Connector {
    pub fn login(&mut self, user: &str, pwd: &str) -> Result<()> {
        let token = self.token(user, pwd)?;
        self.authenticate(user, &token)?;
        self.login_ok();
        Ok(())
    }

    pub fn system_status(&self) -> Result<()> {
        if !self.login {
            bail!("Please login first.");
        };

        let resp = self
            .default_header(self.agent.get(&urls::SYSTEM_STATUS(&self.serial)))
            .call()?;

        // only convert to JSON if Code 200
        let resp_json: serde_json::Value = match resp.status() {
            200 => resp.into_json()?,
            c => bail!("Cannot get system_status | status code: {}", c),
        };

        println!("{}", serde_json::to_string_pretty(&resp_json)?);

        Ok(())
    }
}
