use diesel::{r2d2::ConnectionManager, PgConnection};
use actix_web::{web, App, HttpServer};
use dotenv;

mod data;
mod errors;
mod queries;
mod routes;
mod schema;
mod utilities;

#[cfg(test)]
pub mod test_utils {

    /*
        This macro needs to stay synchronized with the main
        function below. They should both have the same endpoints,
        data, and database connections (unless a different database
        is being used for testing). 
    */
    #[macro_export]
    macro_rules! app {
        () => {{
            dotenv::dotenv().unwrap();

            let url = dotenv::var("DATABASE_URL").unwrap();
            let mgr = ConnectionManager::<PgConnection>::new(url);

            let pool = r2d2::Pool::builder()
                .build(mgr)
                .expect("could not build connection pool");

            test::init_service(
                App::new()
                    .app_data(web::Data::new(pool.clone()))
                    .service(crate::routes::login)
                    .service(crate::routes::register)
                    .service(crate::routes::connect)
            )
            .await
        }};
    }

    pub(crate) use app;
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().unwrap();

    let url = dotenv::var("DATABASE_URL").unwrap();
    let mgr = ConnectionManager::<PgConnection>::new(url);

    let pool = r2d2::Pool::builder()
        .build(mgr)
        .expect("could not build connection pool");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(routes::login)
            .service(routes::register)
            .service(routes::connect)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
