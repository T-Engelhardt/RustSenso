use std::{
    fs,
    io::{Read, Write},
    time::Duration,
};

use anyhow::{bail, Result};
use log::{debug, info, warn};
use thiserror::Error;

use ureq::{Agent, AgentBuilder, Request};

use super::urls;

const SMARTPHONE_ID: &str = "rustSenso";

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Token Outdated")]
    TokenOutdated,
    #[error("resource State is Outdated")]
    StateOutdated
}

pub struct Connector {
    agent: Agent,
    serial: String,
}

impl Connector {
    pub fn new(serial: String) -> Result<Connector> {
        let enforce_https = !cfg!(feature = "local_url");

        let agent = AgentBuilder::new()
            .timeout_read(Duration::from_secs(5))
            .timeout_write(Duration::from_secs(5))
            .https_only(enforce_https)
            .build();
        Ok(Connector { agent, serial })
    }

    fn default_header(&self, req: Request) -> Request {
        req.set("Content-Type", "application/json; charset=UTF-8")
            .set("Accept", "application/json")
            .set("Vaillant-Mobile-App", "senso v3.13 b469 (Android)")
    }

    fn token_save_disk(&self, token: &str) {
        #[cfg(feature = "test_token")]
        let path = "token_test";
        #[cfg(not(feature = "test_token"))]
        let path = "token";

        let mut file = match std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)
        {
            Ok(f) => f,
            Err(e) => {
                // print Error but don't propagate Error
                warn!("Could open/create File. Err: {}", e.to_string());
                return;
            }
        };

        if let Err(e) = file.write_all(token.as_bytes()) {
            warn!("Could write token to file. Err: {}", e.to_string())
        }
        debug!("Saved token: \"{}\" to disk", &token);
    }

    fn token_from_disk(&self) -> Result<String> {
        #[cfg(feature = "test_token")]
        let path = "token_test";
        #[cfg(not(feature = "test_token"))]
        let path = "token";

        let mut file = std::fs::OpenOptions::new().read(true).open(path)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        debug!("Read token: \"{}\" from disk", &buf);
        Ok(buf)
    }

    /// Always prefers token on disk.
    /// If force is set => deletes token from disk and request a new token.
    /// If no token file is found call api to receive a new token.
    fn token(&self, user: &str, pwd: &str, force: bool) -> Result<String> {
        // force new token from api
        if force {
            debug!("Force new token.");
            let _ = fs::remove_file("token");
            return self.token_api(user, pwd);
        }
        // token not found on disk
        if let Ok(token) = self.token_from_disk() {
            Ok(token)
        } else {
            self.token_api(user, pwd)
        }
    }

    fn token_api(&self, user: &str, pwd: &str) -> Result<String> {
        debug!("Calling token API");
        let resp = self
            .default_header(self.agent.post(urls::NEW_TOKEN))
            .send_json(ureq::json!({
                "smartphoneId": SMARTPHONE_ID,
                "username": user,
                "password": pwd,
            }))?;

        let resp_json: serde_json::Value = resp.into_json()?;

        match resp_json["body"]["authToken"].as_str() {
            Some(auth_token) => Ok(auth_token.into()),
            None => bail!("Can't convert/find Json field authToken"),
        }
    }

    fn authenticate(&self, user: &str, token: &str) -> Result<()> {
        debug!("Calling authenticate API");
        if let Err(e) = self
            .default_header(self.agent.post(urls::AUTHENTICATE))
            .send_json(ureq::json!({
                "smartphoneId": SMARTPHONE_ID,
                "username": user,
                "authToken": token,
            }))
        {
            match e.kind() {
                ureq::ErrorKind::HTTP => {
                    // SAFTEY into_response should never fail since we matched on ErrorKind::HTTP
                    let resp = e.into_response().unwrap();
                    match resp.status() {
                        401 => {
                            debug!("Given token is outdated.");
                            bail!(ApiError::TokenOutdated)
                        }
                        _ => bail!(
                            "Can't authenticate with current token: {:}",
                            resp.into_string()?
                        ),
                    }
                }
                _ => Err(e.into()),
            }
        } else {
            Ok(())
        }
    }
}

// PUBLIC INTERFACE //
impl Connector {
    pub fn login(&mut self, user: &str, pwd: &str) -> Result<()> {
        info!("Logging in as {}", user);
        let mut token = self.token(user, pwd, false)?;
        if let Err(e) = self.authenticate(user, &token) {
            match e.downcast_ref() {
                // force new token
                Some(&ApiError::TokenOutdated) => {
                    token = self.token(user, pwd, true)?;
                    self.authenticate(user, &token)?;
                }
                _ => bail!(e)
            }
        }
        info!("Successfully logged in");
        self.token_save_disk(&token);
        Ok(())
    }

    pub fn system_status(&self) -> Result<()> {
        let resp = self
            .default_header(self.agent.get(&urls::SYSTEM_STATUS(&self.serial)))
            .call()?;

        let resp_json: serde_json::Value = resp.into_json()?;

        println!("{}", serde_json::to_string_pretty(&resp_json)?);

        Ok(())
    }

    pub fn live_report(&self) -> Result<()> {
        let resp = self
            .default_header(self.agent.get(&urls::LIVE_REPORT(&self.serial)))
            .call()?;

        let resp_json: serde_json::Value = resp.into_json()?;

        println!("{}", serde_json::to_string_pretty(&resp_json)?);

        Ok(())
    }
}
