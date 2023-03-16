#![allow(non_snake_case, dead_code)]

pub trait Urls {
    fn AUTHENTICATE(&self) -> &str;
    fn NEW_TOKEN(&self) -> &str;
    fn LOGOUT(&self) -> &str;

    fn LIVE_REPORT(&self) -> &str;

    fn SYSTEM(&self) -> &str;
    fn SYSTEM_STATUS(&self) -> &str;
}

pub enum UrlBase {
    VaillantAPI,
    Localhost(usize),
}

impl UrlBase {
    /// returns true if we need to enforce https
    pub fn is_https(&self) -> bool {
        match &self {
            UrlBase::VaillantAPI => true,
            UrlBase::Localhost(_) => false,
        }
    }

    /// check if we can disable the login state check in connector
    pub fn can_disable_login_check(&self) -> bool {
        match &self {
            UrlBase::VaillantAPI => false,
            UrlBase::Localhost(_) => true,
        }
    }
}

#[derive(Debug)]
pub struct VaillantV4 {
    serial: String,
    authenticate: String,
    new_token: String,
    logout: String,
    live_report: String,
    system: String,
    system_status: String,
}

impl VaillantV4 {
    pub fn new(base_enum: UrlBase, serial: String) -> VaillantV4 {
        let base = match base_enum {
            UrlBase::VaillantAPI => "https://smart.vaillant.com/mobile/api/v4".into(),
            UrlBase::Localhost(port) => format!("http://localhost:{}", port),
        };

        let base_authenticate = base.to_owned() + "/account/authentication/v1";

        let facilities_list = base + "/facilities";
        let facilities = format!("{}/{}", facilities_list, serial);

        let system = facilities.clone() + "/systemcontrol/tli/v1";

        VaillantV4 {
            serial,
            authenticate: base_authenticate.clone() + "/authenticate",
            new_token: base_authenticate.clone() + "/token/new",
            logout: base_authenticate + "/logout",
            live_report: facilities + "/livereport/v1",
            system: system.clone(),
            system_status: system + "/status",
        }
    }
}

impl Urls for VaillantV4 {
    fn AUTHENTICATE(&self) -> &str {
        &self.authenticate
    }

    fn NEW_TOKEN(&self) -> &str {
        &self.new_token
    }

    fn LOGOUT(&self) -> &str {
        &self.logout
    }

    fn LIVE_REPORT(&self) -> &str {
        &self.live_report
    }

    fn SYSTEM(&self) -> &str {
        &self.system
    }

    fn SYSTEM_STATUS(&self) -> &str {
        &self.system_status
    }
}
