use const_format::concatcp;

const BASE: &str = "https://smart.vaillant.com/mobile/api/v4";

const BASE_AUTHENTICATE: &str = concatcp!(BASE, "/account/authentication/v1");
pub const AUTHENTICATE: &str = concatcp!(BASE_AUTHENTICATE, "/authenticate");
pub const NEW_TOKEN: &str = concatcp!(BASE_AUTHENTICATE, "/token/new");
pub const LOGOUT: &str = concatcp!(BASE_AUTHENTICATE, "/logout");

