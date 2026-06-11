use sqlx::{Connection, PgConnection, PgPool};
use zero2prod_email_newsletter::{configuration::get_configuration};
use std::net::TcpListener;
use zero2prod_email_newsletter::startup::run;

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {

    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_pool = PgPool::connect(
        &configuration.database.connection_string()
        )
        .await
        .expect("Failed to connect to Postgres");

    let address = format!("127.0.0.1:{}", configuration.application_port);

    let listener = TcpListener::bind(address)
        .expect("Failed to bind to address");
    run(listener, connection_pool)?.await

}