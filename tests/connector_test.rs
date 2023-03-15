use std::sync::Once;

use iso8601_timestamp::Timestamp;
use mockito::Server;
use serde_json::json;

extern crate senso;

static INIT: Once = Once::new();
static mut SERVER_GLOBAL: Option<Server> = None;

fn init() -> &'static mut Server {
    unsafe {
        INIT.call_once(|| {
            let _ = env_logger::builder().is_test(true).try_init();
            SERVER_GLOBAL = Some(mockito::Server::new_with_port(8080));
        });
        SERVER_GLOBAL.as_mut().unwrap()
    }
}

#[test]
fn login_test() {
    let server = init();

    // return authToken
    let token_mock = server
        .mock("POST", "/account/authentication/v1/token/new")
        .match_header("content-type", "application/json; charset=UTF-8")
        .match_header("Accept", "application/json")
        .match_header("Vaillant-Mobile-App", "senso v3.13 b469 (Android)")
        .match_body(r#"{"password":"p","smartphoneId":"rustSenso","username":"u"}"#)
        .with_status(200)
        .with_body(
            json!({
              "body": {
                "authToken": "12345678901234567890123456789012"
              },
              "meta": {}
            })
            .to_string(),
        )
        .create();

    // error on first request
    // authtoken is not valid anymore
    let auth_mock_401 = server
        .mock("POST", "/account/authentication/v1/authenticate")
        .with_status(401)
        .create();

    // retry is ok
    // connector should retry with a new tokens
    let auth_mock_valid = server
        .mock("POST", "/account/authentication/v1/authenticate")
        .with_status(200)
        .create();

    let mut c = senso::connector::Connector::new("1".into()).unwrap();
    c.login("u", "p").unwrap();

    // get first token from api/disk fail auth and retry
    token_mock.expect_at_most(2).assert();
    auth_mock_401.assert();
    auth_mock_valid.assert();
}

#[test]
fn status_test() {
    let server = init();
    let c = senso::connector::Connector::new("1".into()).unwrap();

    // STATUS MOCK 1

    let status_mock = server
        .mock("GET", "/facilities/1/systemcontrol/tli/v1/status")
        .with_body(
            json!({
                "body": {
                  "datetime": "2023-03-14T13:40:24.000Z",
                  "outside_temperature": 4.2
                },
                "meta": {
                  "resourceState": [
                    {
                      "link": {
                        "rel": "self",
                        "resourceLink": "/facilities/21223900202609620938071939N6/systemcontrol/tli/v1/status"
                      },
                      "state": "OUTDATED",
                      "timestamp": 1624441392223_i64
                    }
                  ]
                }
              })
            .to_string(),
        )
        .create();

    let status = c.system_status().unwrap();

    // check body iso8601 timestamp
    // convert to unixtimestamp
    assert_eq!(
        1678801224,
        status
            .body
            .datetime
            .duration_since(Timestamp::UNIX_EPOCH)
            .whole_seconds()
    );

    // check meta timestamp
    assert_eq!(
        1624441392_i64,
        status.meta.resource_state[0].timestamp.timestamp()
    );

    // meta State enum
    assert_eq!(senso::response::meta::State::OUTDATED, status.meta.resource_state[0].state);

    // meta Rel enum
    assert_eq!(senso::response::meta::Rel::_Self, status.meta.resource_state[0].link.rel);

    status_mock.assert();

    // STATUS MOCK 2
    // not a real status message
    // just a test for the enums Rel/State

    let status_mock = server
        .mock("GET", "/facilities/1/systemcontrol/tli/v1/status")
        .with_body(
            json!({
                "body": {
                  "datetime": "2023-03-14T13:40:24.000Z",
                  "outside_temperature": 4.2
                },
                "meta": {
                  "resourceState": [
                    {
                      "link": {
                        "rel": "child",
                        "resourceLink": "/facilities/21223900202609620938071939N6/systemcontrol/tli/v1/status"
                      },
                      "state": "SYNCED",
                      "timestamp": 1624441392223_i64
                    }
                  ]
                }
              })
            .to_string(),
        )
        .create();

    let status = c.system_status().unwrap();

    // meta State enum
    assert_eq!(senso::response::meta::State::SYNCED, status.meta.resource_state[0].state);

    // meta Rel enum
    assert_eq!(senso::response::meta::Rel::Child, status.meta.resource_state[0].link.rel);

    status_mock.assert();
}
