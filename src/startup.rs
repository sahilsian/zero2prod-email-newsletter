use std::net::TcpListener;
use actix_web::{App, HttpServer, web};
use actix_web::dev::{Server};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;
use crate::routes::diagnostics::health_check;
use crate::routes::newsletter::subscribe;

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(db_pool);
    
    let server = HttpServer::new(move || {
        App::new()
                .wrap(TracingLogger::default())
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
                .app_data(db_pool.clone())
        })
        .listen(listener)?
        .run();
    Ok(server)
}