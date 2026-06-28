use secrecy::{ExposeSecret, Secret};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use zero2prod_email_newsletter::{configuration::DatabaseSettings, startup::run};
use std::net::TcpListener;
use std::sync::LazyLock;
use zero2prod_email_newsletter::{configuration::get_configuration};
use zero2prod_email_newsletter::telemetry::{get_subscriber, init_subscriber};

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool
}

static TRACING: LazyLock<()> = LazyLock::new(|| {
     
    if std::env::var("TEST LOG").is_ok() {
        let subscriber = get_subscriber(
        "zero2prod_email_newsletter".into(), 
        "info".into(), 
        std::io::stdout
        );
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(
        "zero2prod_email_newsletter".into(), 
        "info".into(), 
        std::io::sink
        );
        init_subscriber(subscriber);
    }
    
});

async fn spawn_app() -> TestApp {

    LazyLock::force(&TRACING);
    
    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind to address");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let connection_pool = configure_database(&configuration.database).await;


    let server = run(listener, connection_pool.clone())
        .expect("Failed to bind to address");
    
    let _ = tokio::spawn(server);

    TestApp {
        address: address,
        db_pool: connection_pool
    }

}

#[actix_web::test]
async fn health_check_returns_ok() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/api/v1/diagnostics/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[actix_web::test]
async fn subscribe_returns_400_when_fields_are_present_but_empty() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=sahil", "Name field is empty"),
        ("email=sahilsiantech%40gmail.com", "Email field is empty"),
        ("", "Both fields are empty")
    ];

    for (invalid_case, message) in test_cases {
        let body = invalid_case;

        let response = client
            .post(&format!("{}/api/v1/newsletter/subscribe", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(400, response.status().as_u16(), "The API did not return a 200 OK when the payload was: {}", message);
    }
}

#[actix_web::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let body = "name=sahil&email=sahilsiantech%40gmail.com";
    let response = client
        .post(&format!("{}/api/v1/newsletter/subscribe", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.name, "sahil");
    assert_eq!(saved.email, "sahilsiantech@gmail.com");
}

#[actix_web::test]
async fn subscribe_returns_400_for_when_data_is_missing() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=sahil", "Please enter a valid email address"),
        ("email=sahilsiantech%40gmail.com", "Please enter a valid name"),
        ("", "Please enter a valid email address and name")
    ];

    for (invalid_case, message) in test_cases {
        let body = invalid_case;

        let response = client
            .post(&format!("{}/api/v1/newsletter/subscribe", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(400, response.status().as_u16(), "{}", message);
    }

}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let maintenance_settings = DatabaseSettings {
        database_name: "postgres".to_string(),
        username: "postgres".to_string(),
        password: Secret::new("password".to_string()),
        ..config.clone()
    };

    let connection = PgPoolOptions::new()
        .connect_lazy_with(maintenance_settings.connect_options());

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create the database");

    let connection_pool = PgPool::connect_with(config.connect_options())
        .await
        .expect("Failed to connect to Postgres");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the dashboard");

    connection_pool
}