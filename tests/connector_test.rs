#[cfg(feature = "local_url")]
use std::sync::Once;

#[cfg(feature = "local_url")]
use iso8601_timestamp::Timestamp;
#[cfg(feature = "local_url")]
use mockito::Server;
#[cfg(feature = "local_url")]
use serde_json::json;

#[cfg(feature = "local_url")]
extern crate senso;

#[cfg(feature = "local_url")]
static INIT: Once = Once::new();
#[cfg(feature = "local_url")]
static mut SERVER_GLOBAL: Option<Server> = None;

#[cfg(feature = "local_url")]
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
#[cfg(feature = "local_url")]
fn login_test() {
    let server = init();
    let mut c = senso::connector::Connector::new("1".into());

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

    // check login
    // should return Ok
    assert_eq!(Ok(()), c.login("u", "p").map_err(|_| Err::<(), ()>(())));

    // check that on second call of login we get also get a Ok
    // map Error since we can't compare anyhow::Error
    // log should only show info for one login
    assert_eq!(Ok(()), c.login("u", "p").map_err(|_| Err::<(), ()>(())));

    // get first token from api/disk fail auth and retry
    token_mock.expect_at_most(2).assert();
    auth_mock_401.assert();
    auth_mock_valid.assert();
}

#[test]
#[cfg(feature = "local_url")]
fn status_test() {
    let server = init();
    let c = senso::connector::Connector::new("1".into());

    // STATUS MOCK 1

    let status_mock = server
        .mock("GET", "/facilities/1/systemcontrol/tli/v1/status")
        .with_body_from_file("tests/responses/status.json")
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
    assert_eq!(
        senso::response::meta::State::Outdated,
        status.meta.resource_state[0].state
    );

    // meta Rel enum
    assert_eq!(
        senso::response::meta::Rel::_Self,
        status.meta.resource_state[0].link.rel
    );

    status_mock.assert();

    // STATUS MOCK 2
    // not a real status message
    // just a test for the enums Rel/State

    let status_mock = server
        .mock("GET", "/facilities/1/systemcontrol/tli/v1/status")
        .with_body_from_file("tests/responses/status2.json")
        .create();

    let status = c.system_status().unwrap();

    // meta State enum
    assert_eq!(
        senso::response::meta::State::Synced,
        status.meta.resource_state[0].state
    );

    // meta Rel enum
    assert_eq!(
        senso::response::meta::Rel::Child,
        status.meta.resource_state[0].link.rel
    );

    status_mock.assert();
}

#[test]
#[cfg(feature = "local_url")]
fn live_report_test() {
    let server = init();
    let c = senso::connector::Connector::new("1".into());

    let live_report_mock = server
        .mock("GET", "/facilities/1/livereport/v1")
        .with_body_from_file("tests/responses/live_report2.json")
        .create();

    let live_report = c.live_report().unwrap();

    // test timestamp milli or not
    assert_eq!(
        1678877419_i64,
        live_report
            .meta
            .resource_state
            .last()
            .unwrap()
            .timestamp
            .timestamp()
    );

    assert_eq!(
        1536127535_i64,
        live_report.meta.resource_state[0].timestamp.timestamp()
    );

    assert_eq!(
        44.5,
        live_report
            .body
            .find_report_for_device("Control_DHW", "DomesticHotWaterTankTemperature")
            .unwrap()
            .value
    );

    live_report_mock.assert();
}

#[test]
#[cfg(feature = "local_url")]
fn insert_test() {
    use senso::db::DB;

    let server = init();
    let c = senso::connector::Connector::new("2".into());

    let live_report_mock = server
        .mock("GET", "/facilities/2/livereport/v1")
        .with_body_from_file("tests/responses/live_report.json")
        .create();

    let live_report = c.live_report().unwrap();

    assert_eq!(
        45.5,
        live_report
            .body
            .find_report_for_device("Control_DHW", "DomesticHotWaterTankTemperature")
            .unwrap()
            .value
    );
    assert_eq!(
        1.3,
        live_report
            .body
            .find_report_for_device("Control_SYS_senso", "WaterPressureSensor")
            .unwrap()
            .value
    );
    assert_eq!(
        38.5,
        live_report
            .body
            .find_report_for_device("Control_CC1", "FlowTemperatureSensor")
            .unwrap()
            .value
    );

    // test insert into SensorData

    // first test api to SensorData
    let data = senso::db::SensorData::new(&Err(()), &Ok(live_report));
    let data_eq = senso::db::SensorData::new_raw(None, Some(45.5), Some(1.3), Some(38.5));
    assert_eq!(data_eq, data);

    let db = DB::new(None).unwrap();

    // insert and retrieve from DB
    db.insert_sensor_data(data).unwrap();
    let data_db = db.get_sensor_data(Some(1)).unwrap();
    assert_eq!(data_eq, data_db);

    live_report_mock.assert();
}
