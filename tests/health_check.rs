use sqlx::{Connection, PgConnection};
use zero2prod_email_newsletter::startup::run;
use std::net::TcpListener;
use zero2prod_email_newsletter::{configuration::get_configuration};

#[actix_web::test]
async fn health_check_returns_ok() {
    let address = spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/api/v1/diagnostics/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[actix_web::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let app_address = spawn_app();
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_string = configuration.database.connection_string();

    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres");

    let client = reqwest::Client::new();

    let body = "name=sahil&email=sahilsiantech%40gmail.com";
    let response = client
        .post(&format!("{}/api/v1/newsletter/subscribe", &app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.name, "sahil");
    assert_eq!(saved.email, "sahilsiantech@gmail.com");
}

#[actix_web::test]
async fn subscribe_returns_400_for_when_data_is_missing() {
    let address = spawn_app();

    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=sahil", "Please enter a valid email address"),
        ("email=sahilsiantech%40gmail.com", "Please enter a valid name"),
        ("", "Please enter a valid email address and name")
    ];

    for (invalid_case, message) in test_cases {
        let body = invalid_case;

        let response = client
            .post(&format!("{}/api/v1/newsletter/subscribe", &address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(400, response.status().as_u16(), "{}", message);
    }

}

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind to address");
    let port = listener.local_addr().unwrap().port();
    let server = run(listener)
        .expect("Failed to bind to address");
    
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}