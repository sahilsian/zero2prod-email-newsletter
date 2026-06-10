use std::net::TcpListener;
use actix_web::{App, HttpServer, web};
use actix_web::dev::{Server};

use crate::routes::diagnostics::health_check;
use crate::routes::newsletter::subscribe;

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
   let server = HttpServer::new(|| {
        App::new()
            .service(
                web::scope("/api")
                .service(
                    // API VERSION 1
                    web::scope("/v1")
                        .service(
                            web::scope("/diagnostics")
                            .route("/health_check", web::get().to(health_check))
                        )
                        .service(
                            web::scope("/newsletter")
                            .route("/subscribe", web::post().to(subscribe))
                        )
                )
            )
    })
    .listen(listener)?
    .run();
    Ok(server)
}