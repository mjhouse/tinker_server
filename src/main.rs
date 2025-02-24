use actix_web::{web, App, HttpServer};

mod routes;
mod schema;

use diesel::{r2d2::ConnectionManager, PgConnection};
use routes::report;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let url = "postgresql://user:password@localhost:5432/game";
    let mgr = ConnectionManager::<PgConnection>::new(url);

    let pool = r2d2::Pool::builder()
        .build(mgr)
        .expect("could not build connection pool");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(report::movement)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
