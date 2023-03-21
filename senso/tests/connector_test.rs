use chrono::{NaiveDate, NaiveDateTime};
use iso8601_timestamp::Timestamp;
use mockito::{Matcher, Mock, Server, ServerGuard};
use num_traits::cast::FromPrimitive;
use senso::{
    db::DB,
    request::emf,
    response::emf_devices::{EmfFunction, EmfDevice},
    yp::{build_yp_data_vec, UsageFunctionWeek},
};
use serde_json::json;
use std::{env, sync::Once};

extern crate senso;

static INIT: Once = Once::new();

// init env_logger once
fn init() {
    INIT.call_once(|| {
        // default senso=debug if RUST_LOG is not set
        if env::var("RUST_LOG").is_err() {
            env::set_var("RUST_LOG", "senso=debug")
        }
        let _ = env_logger::builder().is_test(true).try_init();
    });
}

// returns port of mockito Server as usize
// SAFTEY last value after : should always be the port
// this should only be used for test functions
fn port(server: &ServerGuard) -> usize {
    server
        .host_with_port()
        .split(":")
        .last()
        .unwrap()
        .parse()
        .unwrap()
}

#[test]
fn login_test() {
    init();
    let mut server = Server::new();
    let mut c = senso::connector::Connector::new(
        senso::urls::UrlBase::Localhost(port(&server)),
        "1".into(),
        "./token_test".into(),
    );

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
fn status_test() {
    init();
    let mut server = Server::new();
    let c = senso::connector::Connector::new(
        senso::urls::UrlBase::Localhost(port(&server)),
        "1".into(),
        "".into(),
    );

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
}

#[test]
fn live_report_test() {
    init();
    let mut server = Server::new();
    let c = senso::connector::Connector::new(
        senso::urls::UrlBase::Localhost(port(&server)),
        "1".into(),
        "".into(),
    );

    let live_report_mock = server
        .mock("GET", "/facilities/1/livereport/v1")
        .with_body_from_file("tests/responses/live_report2.json")
        .create();

    let live_report = c.live_report().unwrap();

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
fn emf_report_device_test() {
    init();
    let mut server = Server::new();
    let c = senso::connector::Connector::new(
        senso::urls::UrlBase::Localhost(port(&server)),
        "1".into(),
        "".into(),
    );

    let emf_report_device_mock = server
        .mock("GET", "/facilities/1/emf/v1/devices/x")
        .with_body_from_file("tests/responses/emf_report_device.json")
        .create();

    let emf_report_device = c.emf_report_device("x", emf::empty_query()).unwrap();

    assert_eq!(
        3000.0,
        emf_report_device
            .body
            .first()
            .unwrap()
            .dataset
            .first()
            .unwrap()
            .value
    );

    // this is in UTC+0
    // python used to be local time
    assert_eq!(
        1677456000,
        emf_report_device
            .body
            .first()
            .unwrap()
            .dataset
            .first()
            .unwrap()
            .key
            .duration_since(Timestamp::UNIX_EPOCH)
            .whole_seconds()
    );

    emf_report_device_mock.assert();
}

#[test]
fn emf_devices() {
    init();
    let mut server = Server::new();
    let c = senso::connector::Connector::new(
        senso::urls::UrlBase::Localhost(port(&server)),
        "1".into(),
        "".into(),
    );

    let emf_devices_mock = server
        .mock("GET", "/facilities/1/emf/v1/devices")
        .with_body_from_file("tests/responses/emf_devices.json")
        .create();

    let emf_devices = c.emf_devices().unwrap();

    assert_eq!(
        "VWL 55/6 A 230V",
        emf_devices.body.first().unwrap().marketing_name
    );

    emf_devices_mock.assert();
}

#[test]
fn insert_test() {
    use senso::db::DB;

    init();
    let mut server = Server::new();
    let c = senso::connector::Connector::new(
        senso::urls::UrlBase::Localhost(port(&server)),
        "2".into(),
        "".into(),
    );

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
    let data_db = db.get_sensor_data(1).unwrap();
    assert_eq!(data_eq, data_db);

    live_report_mock.assert();
}

#[test]
fn yp_test() {
    init();
    let mut server = Server::new();
    let c = senso::connector::Connector::new(
        senso::urls::UrlBase::Localhost(port(&server)),
        "1".into(),
        "".into(),
    );

    let same_match = Matcher::AllOf(vec![
        Matcher::UrlEncoded("timeRange".into(), "WEEK".into()),
        Matcher::UrlEncoded("start".into(), "2023-02-27".into()),
        Matcher::UrlEncoded("offset".into(), "0".into()),
    ]);

    // data from db id 19..=25, Week starting at the 13.02.2023 / 1676246400
    // json data in tests/responses/emf_report/
    // dates in json data don't match the with id 19..=25
    //
    // ch_yp: 4.1667, 4, 3.5, 4, 5, 5, 4
    // hw_yp: 3, 3, 3.3333, 3.25, 4, 3.5, 2.75
    // total_y: 25000, 25000, 27000, 27000, 2200, 2200, 25000
    // total_p: 9000, 9000, 11000, 10000, 6000, 7000, 10000
    // total_yp: 3.7778, 3.7778, 3.4545, 3.7, 4.6667, 4.1429, 3.5
    // ts: TODO 2023-02-27 till 2023-03-05

    let mut mocks: Vec<&Mock> = vec![];

    // CENTRAL_HEATING
    let m = &server
        .mock("GET", "/facilities/1/emf/v1/devices/hp")
        .with_body_from_file("tests/responses/emf_report/ch_hp_p.json")
        .match_query(Matcher::AllOf(vec![
            same_match.clone(),
            Matcher::UrlEncoded("energyType".into(), "CONSUMED_ELECTRICAL_POWER".into()),
            Matcher::UrlEncoded("function".into(), "CENTRAL_HEATING".into()),
        ]))
        .create();
    mocks.push(m);

    let m = &server
        .mock("GET", "/facilities/1/emf/v1/devices/hp")
        .with_body_from_file("tests/responses/emf_report/ch_hp_y.json")
        .match_query(Matcher::AllOf(vec![
            same_match.clone(),
            Matcher::UrlEncoded("energyType".into(), "ENVIRONMENTAL_YIELD".into()),
            Matcher::UrlEncoded("function".into(), "CENTRAL_HEATING".into()),
        ]))
        .create();
    mocks.push(m);

    let m = &server
        .mock("GET", "/facilities/1/emf/v1/devices/bo")
        .with_body_from_file("tests/responses/emf_report/ch_bo_p.json")
        .match_query(Matcher::AllOf(vec![
            same_match.clone(),
            Matcher::UrlEncoded("energyType".into(), "CONSUMED_ELECTRICAL_POWER".into()),
            Matcher::UrlEncoded("function".into(), "CENTRAL_HEATING".into()),
        ]))
        .create();
    mocks.push(m);

    // DHW
    let m = &server
        .mock("GET", "/facilities/1/emf/v1/devices/hp")
        .with_body_from_file("tests/responses/emf_report/hw_hp_p.json")
        .match_query(Matcher::AllOf(vec![
            same_match.clone(),
            Matcher::UrlEncoded("energyType".into(), "CONSUMED_ELECTRICAL_POWER".into()),
            Matcher::UrlEncoded("function".into(), "DHW".into()),
        ]))
        .create();
    mocks.push(m);

    let m = &server
        .mock("GET", "/facilities/1/emf/v1/devices/hp")
        .with_body_from_file("tests/responses/emf_report/hw_hp_y.json")
        .match_query(Matcher::AllOf(vec![
            same_match.clone(),
            Matcher::UrlEncoded("energyType".into(), "ENVIRONMENTAL_YIELD".into()),
            Matcher::UrlEncoded("function".into(), "DHW".into()),
        ]))
        .create();
    mocks.push(m);

    let m = &server
        .mock("GET", "/facilities/1/emf/v1/devices/bo")
        .with_body_from_file("tests/responses/emf_report/hw_bo_p.json")
        .match_query(Matcher::AllOf(vec![
            same_match.clone(),
            Matcher::UrlEncoded("energyType".into(), "CONSUMED_ELECTRICAL_POWER".into()),
            Matcher::UrlEncoded("function".into(), "DHW".into()),
        ]))
        .create();
    mocks.push(m);

    let devices = vec![(EmfDevice::HeatPump, "hp"), (EmfDevice::Boiler, "bo")];
    let mut usage_ch = UsageFunctionWeek::new(EmfFunction::CentralHeating, &devices, 2023, 9);
    let mut usage_dhw = UsageFunctionWeek::new(EmfFunction::DomesticHotWater, &devices, 2023, 9);
    usage_ch.retrieve_data(&c).unwrap();
    usage_dhw.retrieve_data(&c).unwrap();

    let result = build_yp_data_vec(usage_dhw, usage_ch).unwrap();

    // check yp
    let ch_yp_vec: Vec<f64> = result.iter().map(|f| f.ch_yp).collect();
    assert_eq!(vec![4.1667, 4.0, 3.5, 4.0, 5.0, 5.0, 4.0], ch_yp_vec);
    let hw_yp_vec: Vec<f64> = result.iter().map(|f| f.hw_yp).collect();
    assert_eq!(vec![3.0, 3.0, 3.3333, 3.25, 4.0, 3.5, 2.75], hw_yp_vec);
    let total_yp_vec: Vec<f64> = result.iter().map(|f| f.total_yp).collect();
    assert_eq!(
        vec![3.7778, 3.7778, 3.4545, 3.7, 4.6667, 4.1429, 3.5],
        total_yp_vec
    );

    // check ts
    let ts: Vec<NaiveDateTime> = result.iter().map(|f| f.ts).collect();
    let ts_eq: Vec<NaiveDateTime> = (0..7_u8)
        .map(|day| {
            NaiveDate::from_isoywd_opt(2023, 9, chrono::Weekday::from_u8(day).unwrap())
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
        })
        .collect();
    assert_eq!(ts_eq, ts);

    // create in memory db
    let db = DB::new(None).unwrap();

    // insert into DB
    db.insert_yp_data(&result[0]).unwrap();

    // assert every mock
    for x in mocks {
        x.assert();
    }
}
