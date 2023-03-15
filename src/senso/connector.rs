use std::{
    fs,
    io::{Read, Write},
    time::Duration,
};

use anyhow::{bail, Result};
use log::{debug, info, warn};
use thiserror::Error;

use ureq::{Agent, AgentBuilder, Request};

use super::{response, urls};

const SMARTPHONE_ID: &str = "rustSenso";

#[cfg(feature = "test_token")]
const PATH_TOKEN: &str = "token_test";
#[cfg(not(feature = "test_token"))]
const PATH_TOKEN: &str = "token";

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Token Outdated")]
    TokenOutdated,
    #[error("resource State is Outdated")]
    StateOutdated,
}

pub struct Connector {
    agent: Agent,
    serial: String,
}

impl Connector {
    pub fn new(serial: String) -> Connector {
        let enforce_https = !cfg!(feature = "local_url");

        let agent = AgentBuilder::new()
            .timeout_read(Duration::from_secs(5))
            .timeout_write(Duration::from_secs(5))
            .https_only(enforce_https)
            .build();
        Connector { agent, serial }
    }

    fn default_header(&self, req: Request) -> Request {
        req.set("Content-Type", "application/json; charset=UTF-8")
            .set("Accept", "application/json")
            .set("Vaillant-Mobile-App", "senso v3.13 b469 (Android)")
    }

    fn token_save_disk(&self, token: &str) {
        let mut file = match std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(PATH_TOKEN)
        {
            Ok(f) => f,
            Err(e) => {
                // print Error but don't propagate Error
                warn!("Could open/create File. Err: \"{}\".", e.to_string());
                return;
            }
        };

        if let Err(e) = file.write_all(token.as_bytes()) {
            warn!("Could write token to file. Err: \"{}\".", e.to_string())
        }
        debug!("Saved token: \"{}\" to disk.", &token);
    }

    fn token_from_disk(&self) -> Result<String> {
        let mut file = std::fs::OpenOptions::new().read(true).open(PATH_TOKEN)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        info!("Successfully read token from disk.");
        debug!("Read token: \"{}\" from disk.", &buf);
        Ok(buf)
    }

    /// Always prefers token on disk.
    /// If force is set => deletes token from disk and request a new token.
    /// If no token file is found call api to receive a new token.
    fn token(&self, user: &str, pwd: &str, force: bool) -> Result<String> {
        // force new token from api
        if force {
            debug!("Force new token.");
            let _ = fs::remove_file(PATH_TOKEN);
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
        debug!("Calling token api.");
        let resp = self
            .default_header(self.agent.post(urls::NEW_TOKEN))
            .send_json(ureq::json!({
                "smartphoneId": SMARTPHONE_ID,
                "username": user,
                "password": pwd,
            }))?;

        let resp_token: response::token::Root = resp.into_json()?;
        info!("Received new token.");

        Ok(resp_token.body.auth_token)
    }

    fn authenticate(&self, user: &str, token: &str) -> Result<()> {
        debug!("Calling authenticate api.");
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
                            info!("Given token is outdated/not valid.");
                            bail!(ApiError::TokenOutdated)
                        }
                        _ => bail!(
                            "Can't authenticate with current token. Response \"{}\".",
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
    pub fn login(&self, user: &str, pwd: &str) -> Result<()> {
        info!("Logging in as \"{}\".", &user);
        let mut token = self.token(user, pwd, false)?;
        if let Err(e) = self.authenticate(user, &token) {
            match e.downcast_ref() {
                // force new token
                Some(&ApiError::TokenOutdated) => {
                    info!("Trying to get a new token.");
                    token = self.token(user, pwd, true)?;
                    self.authenticate(user, &token)?;
                }
                _ => bail!(e),
            }
        }
        info!("Successfully logged in.");
        self.token_save_disk(&token);
        Ok(())
    }

    pub fn system_status(&self) -> Result<response::status::Root> {
        let resp = self
            .default_header(self.agent.get(&urls::SYSTEM_STATUS(&self.serial)))
            .call()?;

        Ok(resp.into_json()?)
    }

    pub fn live_report(&self) -> Result<response::live_report::Root> {
        let resp = self
            .default_header(self.agent.get(&urls::LIVE_REPORT(&self.serial)))
            .call()?;

        Ok(resp.into_json()?)
    }
}
