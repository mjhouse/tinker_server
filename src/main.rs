use diesel::{r2d2::ConnectionManager, PgConnection};
use actix_web::{web, App, HttpServer};
use dotenv;
use utilities::process_messages;

mod payloads;
mod errors;
mod queries;
mod routes;
mod utilities;

#[cfg(test)]
pub mod test_utils {
    use diesel_migrations::MigrationHarness;
    use diesel::{r2d2::ConnectionManager, Connection,RunQueryDsl,PgConnection}; 
    use actix_web::{dev::Service, test, web, App};
    use actix_http::Request;
    use url::Url;
    use tinker_records::tests::MIGRATIONS;

    use crate::{queries::Database, utilities};
    
    const SQL: &str = include_str!("../assets/setup.sql");
    
    pub async fn pool(database: &str) -> Database {
        let mut url = Url::parse(&dotenv::var("DATABASE_URL").unwrap()).unwrap();
        url.set_path(database);

        let mgr = ConnectionManager::<PgConnection>::new(url);

        r2d2::Pool::builder()
            .build(mgr)
            .expect("could not build connection pool")
    }

    pub async fn setup(database: &str) -> impl Service<Request, Response = actix_web::dev::ServiceResponse, Error = actix_web::Error> {
        // get the test database url
        dotenv::dotenv().unwrap();

        let mut base = Url::parse(&dotenv::var("DATABASE_URL").unwrap()).unwrap();
        base.set_path("");
        let base = base.to_string();
        let url = format!("{}/postgres", base);
    
        // get a connection to the database/postgres
        let mut conn = PgConnection::establish(&url).expect("Cannot connect to postgres database.");
    
        // create a test database named 'test'
        let query = diesel::sql_query(&format!("CREATE DATABASE {}", database));
        query.execute(&mut conn).expect(&format!("Could not create database {}", database));
    
        // get a connection to the database/test
        let url = format!("{}/{}", base, database);
        let mut conn = PgConnection::establish(&url).expect("Cannot connect to test database.");
    
        // run all migrations
        conn.run_pending_migrations(MIGRATIONS).expect("Could not run migrations");
    
        // create and insert a hashed password
        let password = utilities::password::hash("PASSWORD").unwrap();
        let sql = SQL.replace("<PASSWORD>",&password);

        // execute assets/setup.sql
        let query = diesel::sql_query(sql);
        query.execute(&mut conn).expect(&format!("Could not create records {}", database));
    
        // build a connection manager for the test database
        let mgr = ConnectionManager::<PgConnection>::new(url);
        
        // build a connection pool from the manager
        let pool = r2d2::Pool::builder()
            .build(mgr)
            .expect("Could not build connection pool");
    
        // create the actix App and return it
        test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(crate::routes::login)
                .service(crate::routes::register)
                .service(crate::routes::connect)
        ).await
    }
    
    pub fn teardown(database: &str) {
        // get the test database url
        dotenv::dotenv().unwrap();
        let mut base = Url::parse(&dotenv::var("DATABASE_URL").unwrap()).unwrap();
        base.set_path("");
        let base = base.to_string();
        let url = format!("{}/postgres", base);

        // get a connection to the database/postgres
        let mut conn = PgConnection::establish(&url).expect("Cannot connect to postgres database.");

        
        // disconnect all users to the database
        let disconnect_users = format!("
            SELECT pg_terminate_backend(pid)
            FROM pg_stat_activity
            WHERE datname = '{}';",
            database
        );
    
        // drop the 'test' database
        diesel::sql_query(&disconnect_users)
            .execute(&mut conn)
            .unwrap();
    
        let query = diesel::sql_query(&format!("DROP DATABASE {}", database));
        query
            .execute(&mut conn)
            .expect(&format!("Couldn't drop database {}", database));
    }

    #[actix_web::test]
    async fn test_database_setup() {
        let _ = setup("test_database_setup").await;
        teardown("test_database_setup");
    }

}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().unwrap();

    let url = dotenv::var("DATABASE_URL").unwrap();
    let mgr = ConnectionManager::<PgConnection>::new(url);

    let pool = r2d2::Pool::builder()
        .build(mgr)
        .expect("could not build connection pool");

    // start the message processing background task
    process_messages(pool.clone());

    // TODO: use the configure method to add resources and abstract
    //       app construction into a standalone method: https://docs.rs/actix-web/latest/actix_web/struct.App.html#method.configure
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
