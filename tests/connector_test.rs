use serde_json::json;

extern crate senso;

#[test]
fn login_test() {
    let _ = env_logger::builder().is_test(true).try_init();

    let mut server = mockito::Server::new_with_port(8080);

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
    let auth_mock_401 = server
        .mock("POST", "/account/authentication/v1/authenticate")
        .with_status(401)
        .create();

    // retry is ok
    let auth_mock_valid = server
        .mock("POST", "/account/authentication/v1/authenticate")
        .with_status(200)
        .create();

    let mut c = senso::connector::Connector::new("".into()).unwrap();
    c.login("u", "p").unwrap();

    token_mock.expect_at_most(2).assert();
    auth_mock_401.assert();
    auth_mock_valid.assert();
}
