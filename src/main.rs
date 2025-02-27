use diesel::{r2d2::ConnectionManager, PgConnection};
use actix_web::{web, App, HttpServer};
use actix_jwt_auth_middleware::{use_jwt::UseJWTOnApp as _, Authority, TokenSigner};
use ed25519_compact::KeyPair;
use jwt_compact::alg::Ed25519;
use dotenv;

mod data;

mod activity;
mod queries;
mod routes;
mod schema;

use data::payloads::Account;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().unwrap();

    let KeyPair {
        pk: public_key,
        sk: secret_key,
    } = KeyPair::generate();

    let url = dotenv::var("DATABASE_URL").unwrap();
    let mgr = ConnectionManager::<PgConnection>::new(url);

    let pool = r2d2::Pool::builder()
        .build(mgr)
        .expect("could not build connection pool");

    HttpServer::new(move || {
        let authority = Authority::<Account, Ed25519, _, _>::new()
            .refresh_authorizer(|| async move { Ok(()) })
            .token_signer(Some(
                TokenSigner::new()
                    .signing_key(secret_key.clone())
                    .algorithm(Ed25519)
                    .build()
                    .expect(""),
            ))
            .verifying_key(public_key)
            .build()
            .expect("");
        
        App::new()
            .app_data(web::Data::new(pool.clone()))
            // .service(routes::login)
            .service(routes::register)
            .use_jwt(authority, web::scope("").service(activity::connect))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
