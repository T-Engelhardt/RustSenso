use std::{
    fs,
    io::{Read, Write},
    time::Duration,
};

use anyhow::{anyhow, bail, Result};
use log::{debug, info, warn};
use serde::de::DeserializeOwned;
use thiserror::Error;

use ureq::{Agent, AgentBuilder, Request};

use crate::request::emf;

use super::{response, urls};

const SMARTPHONE_ID: &str = "rustSenso";

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Token Outdated")]
    TokenOutdated,
    #[error("resource State is Outdated")]
    StateOutdated,
}

pub struct Connector {
    agent: Agent,
    disable_login_check: bool,
    urls: Box<dyn urls::Urls>,
    token_path: String,
    login_state: Result<(), anyhow::Error>,
}

impl Connector {
    pub fn new(url_base: urls::UrlBase, serial: String, token_path: String) -> Connector {
        let agent = AgentBuilder::new()
            .timeout_read(Duration::from_secs(5))
            .timeout_write(Duration::from_secs(5))
            .https_only(url_base.is_https())
            .build();
        Connector {
            agent,
            disable_login_check: url_base.can_disable_login_check(),
            urls: Box::new(urls::VaillantV4::new(url_base, serial)),
            token_path,
            login_state: Err(anyhow!("Please login.")),
        }
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
            .open(&self.token_path)
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
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .open(&self.token_path)?;
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
            let _ = fs::remove_file(&self.token_path);
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
            .default_header(self.agent.post(self.urls.NEW_TOKEN()))
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
            .default_header(self.agent.post(self.urls.AUTHENTICATE()))
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
                            "Can't authenticate with current token. Response: \"{}\".",
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

    fn login_unchecked(&self, user: &str, pwd: &str) -> Result<()> {
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

    fn call_api<'a, T, P>(&self, url: &str, query: P) -> Result<T>
    where
        T: DeserializeOwned,
        P: IntoIterator<Item = (&'a str, &'a str)>,
    {
        if !self.disable_login_check {
            if let Err(e) = &self.login_state {
                bail!(e.to_string())
            }
        }
        let resp = self
            .default_header(self.agent.get(url))
            .query_pairs(query)
            .call()?;

        Ok(resp.into_json()?)
    }
}

// PUBLIC INTERFACE //
impl Connector {
    /// Tries to login to vaillant api.
    /// On Error you can try again.
    /// On Ok all future calls will return OK. On Ok this will never call the api again.
    pub fn login(&mut self, user: &str, pwd: &str) -> Result<()> {
        if self.login_state.is_ok() {
            info!("Already logged in.");
            return Ok(());
        }

        // save new state
        self.login_state = self.login_unchecked(user, pwd);

        // return state for caller
        match &self.login_state {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!(e.to_string())),
        }
    }

    pub fn system_status(&self) -> Result<response::status::Root> {
        self.call_api(self.urls.SYSTEM_STATUS(), emf::empty_query())
    }

    pub fn live_report(&self) -> Result<response::live_report::Root> {
        self.call_api(self.urls.LIVE_REPORT(), emf::empty_query())
    }

    pub fn emf_devices(&self) -> Result<response::emf_devices::Root> {
        self.call_api(self.urls.EMF_DEVICES(), emf::empty_query())
    }

    pub fn emf_report_device<'a, P>(
        &self,
        device_id: &str,
        query: P,
    ) -> Result<response::emf_report_device::Root>
    where
        P: IntoIterator<Item = (&'a str, &'a str)>,
    {
        self.call_api(self.urls.EMF_REPORT_DEVICE(device_id).as_ref(), query)
    }
}
