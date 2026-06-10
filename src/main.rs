use zero2prod_email_newsletter::{configuration::get_configuration};
use std::net::TcpListener;
use zero2prod_email_newsletter::startup::run;

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {

    let configuration = get_configuration().expect("Failed to read configuration.");
    let address = format!("127.0.0.1:{}", configuration.application_port);

    let listener = TcpListener::bind(address)
        .expect("Failed to bind to address");
    run(listener)?.await

}