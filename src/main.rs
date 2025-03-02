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
    use diesel_migrations::{embed_migrations,EmbeddedMigrations, MigrationHarness};
    use diesel::{r2d2::ConnectionManager, Connection,RunQueryDsl,PgConnection}; 
    use actix_web::{dev::Service, test, web, App};
    use actix_http::Request;
    
    const TEST_DATABASE: &str = "test";
    const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");
    const SQL: &str = include_str!("../assets/setup.sql");
    
    pub async fn setup() -> impl Service<Request> {
        // get the test database url
        dotenv::dotenv().unwrap();
        let base = dotenv::var("DATABASE_URL").unwrap();
        let url = format!("{}/postgres", base);
    
        // get a connection to the database/postgres
        let mut conn = PgConnection::establish(&url).expect("Cannot connect to postgres database.");
    
        // create a test database named 'test'
        let query = diesel::sql_query(&format!("CREATE DATABASE {}", TEST_DATABASE));
        query.execute(&mut conn).expect(&format!("Could not create database {}", TEST_DATABASE));
    
        // get a connection to the database/test
        let url = format!("{}/{}", base, TEST_DATABASE);
        let mut conn = PgConnection::establish(&url).expect("Cannot connect to test database.");
    
        // run all migrations
        conn.run_pending_migrations(MIGRATIONS).expect("Could not run migrations");
    
        // execute assets/setup.sql
        let query = diesel::sql_query(SQL);
        query.execute(&mut conn).expect(&format!("Could not create records {}", TEST_DATABASE));
    
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
                .service(crate::routes::create_character)
                .service(crate::routes::fetch_characters)
                .service(crate::routes::connect)
        ).await
    }
    
    pub fn teardown() {
        // get the test database url
        dotenv::dotenv().unwrap();
        let base = dotenv::var("DATABASE_URL").unwrap();
        let url = format!("{}/postgres", base);

        // get a connection to the database/postgres
        let mut conn = PgConnection::establish(&url).expect("Cannot connect to postgres database.");

        
        // disconnect all users to the database
        let disconnect_users = format!("
            SELECT pg_terminate_backend(pid)
            FROM pg_stat_activity
            WHERE datname = '{}';",
            TEST_DATABASE
        );
    
        // drop the 'test' database
        diesel::sql_query(&disconnect_users)
            .execute(&mut conn)
            .unwrap();
    
        let query = diesel::sql_query(&format!("DROP DATABASE {}", TEST_DATABASE));
        query
            .execute(&mut conn)
            .expect(&format!("Couldn't drop database {}", TEST_DATABASE));
    }

    #[actix_web::test]
    async fn test_database_setup() {
        let app = setup().await;
    }

    // /*
    //     This macro needs to stay synchronized with the main
    //     function below. They should both have the same endpoints,
    //     data, and database connections (unless a different database
    //     is being used for testing). 
    // */
    // #[macro_export]
    // macro_rules! setup {
    //     () => {{
    //         dotenv::dotenv().unwrap();

    //         let url = dotenv::var("DATABASE_URL").unwrap();
    //         let mgr = ConnectionManager::<PgConnection>::new(url);

    //         let pool = r2d2::Pool::builder()
    //             .build(mgr)
    //             .expect("could not build connection pool");

    //         let app = test::init_service(
    //             App::new()
    //                 .app_data(web::Data::new(pool.clone()))
    //                 .service(crate::routes::login)
    //                 .service(crate::routes::register)
    //                 .service(crate::routes::create_character)
    //                 .service(crate::routes::fetch_characters)
    //                 .service(crate::routes::connect)
    //         )
    //         .await;

    //         (app,pool)
    //     }};
    // }

    // /*
    //     These tables need to be in reverse order of relationships so
    //     that deleting one doesn't fail due to foreign key references
    // */
    // #[macro_export]
    // macro_rules! teardown {
    //     ($pool: ident) => {{
    //         use diesel::RunQueryDsl;
    //         let mut conn = $pool.get().expect("No database");
    //         diesel::delete(crate::schema::join_character_abilities::table).execute(&mut conn).unwrap();
    //         diesel::delete(crate::schema::join_character_items::table).execute(&mut conn).unwrap();
    //         diesel::delete(crate::schema::items::table).execute(&mut conn).unwrap();
    //         diesel::delete(crate::schema::abilities::table).execute(&mut conn).unwrap();
    //         diesel::delete(crate::schema::characters::table).execute(&mut conn).unwrap();
    //         diesel::delete(crate::schema::accounts::table).execute(&mut conn).unwrap();
    //     }};
    // }

    // pub(crate) use setup;
    // pub(crate) use teardown;
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().unwrap();

    let url = dotenv::var("DATABASE_URL").unwrap();
    let mgr = ConnectionManager::<PgConnection>::new(url);

    let pool = r2d2::Pool::builder()
        .build(mgr)
        .expect("could not build connection pool");

    // TODO: use the configure method to add resources and abstract
    //       app construction into a standalone method: https://docs.rs/actix-web/latest/actix_web/struct.App.html#method.configure
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(routes::login)
            .service(routes::register)
            .service(routes::create_character)
            .service(routes::fetch_characters)
            .service(routes::connect)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
