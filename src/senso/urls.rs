#![allow(non_snake_case)]

use const_format::concatcp;

const BASE: &str = if cfg!(feature = "local_url") {
    "http://localhost:8080"
} else {
    "https://smart.vaillant.com/mobile/api/v4"
};

const BASE_AUTHENTICATE: &str = concatcp!(BASE, "/account/authentication/v1");
pub const AUTHENTICATE: &str = concatcp!(BASE_AUTHENTICATE, "/authenticate");
pub const NEW_TOKEN: &str = concatcp!(BASE_AUTHENTICATE, "/token/new");
pub const LOGOUT: &str = concatcp!(BASE_AUTHENTICATE, "/logout");

const FACILITIES_LIST: &str = concatcp!(BASE, "/facilities");

fn FACILITIES(serial: &str) -> String {
    format!("{}/{}", FACILITIES_LIST, serial)
}

pub fn LIVE_REPORT(serial: &str) -> String {
    format!("{}/livereport/v1", FACILITIES(serial))
}

fn EMF_DEVICES(serial: &str) -> String {
    format!("{}/emf/v1/devices", FACILITIES(serial))
}

pub fn EMF_REPORT_DEVICE(serial: &str, device_id: &str) -> String {
    format!("{}/{}", EMF_DEVICES(serial), device_id)
}

fn SYSTEM(serial: &str) -> String {
    format!("{}/systemcontrol/tli/v1", FACILITIES(serial))
}

pub fn SYSTEM_STATUS(serial: &str) -> String {
    format!("{}/status", SYSTEM(serial))
}
