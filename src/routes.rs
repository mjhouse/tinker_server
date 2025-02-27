use actix_jwt_auth_middleware::{AuthError, AuthResult, TokenSigner};
use actix_web::{get, post, web, HttpResponse, Responder};
use jwt_compact::alg::Ed25519;
use crate::{data::{models::AccountInsert, payloads::{Account, Login, Register}}, queries::{self, Database}};
use argon2::{password_hash::{rand_core::OsRng, Salt, SaltString}, Argon2, PasswordHasher, PasswordVerifier};

// #[get("/login")]
// async fn login(
//     pool: web::Data<Database>,
//     info: web::Json<MovementSend>,
//     signer: web::Data<TokenSigner<User, Ed25519>>
// ) -> AuthResult<HttpResponse> {

//     let user = User { id: 1, name: "test".into() };
//     Ok(HttpResponse::Ok()
//         .cookie(signer.create_access_cookie(&user)?)
//         .cookie(signer.create_refresh_cookie(&user)?)
//         .body("You are now logged in"))
// }

#[post("/register")]
async fn register(
    pool: web::Data<Database>,
    info: web::Json<Register>
) -> AuthResult<impl Responder> {
    let form = info.into_inner();
    let username = form.username;
    let password1 = form.password1;
    let password2 = form.password2;

    if username.contains(char::is_whitespace) {
        println!("Name contains whitespace");
        return Err(AuthError::NoToken);
    }

    if password1.len() < 8 {
        println!("Password less than 8 characters");
        return Err(AuthError::NoToken);
    }

    if password1 != password2 {
        println!("Passwords not equal");
        return Err(AuthError::NoToken);
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password1.as_bytes(), &salt).unwrap().to_string();

    let record = queries::create_account(&pool, username, hash).await.unwrap();

    Ok(web::Json(Account {
        id: record.id,
        name: record.username
    }))
}

#[cfg(test)]
mod tests {
    use diesel::pg::PgConnection;
    use diesel::r2d2::ConnectionManager;
    use actix_web::{test, App};
    use actix_jwt_auth_middleware::{use_jwt::UseJWTOnApp as _, Authority, TokenSigner};
    use ed25519_compact::KeyPair;
    use jwt_compact::alg::Ed25519;
    use super::*;

    macro_rules! app {
        ($endpoint: ident) => {{
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
            
            test::init_service(
                App::new()
                    .app_data(web::Data::new(pool.clone()))
                    .service($endpoint)
                    .use_jwt(authority, web::scope(""))
            ).await
        }};
    }

    #[actix_web::test]
    async fn test_register() {
        let app = app!(register);

        let req = test::TestRequest::post()
            .uri("/register")
            .set_json(Register {
                username: "TEST".into(),
                password1: "PASSWORD".into(),
                password2: "PASSWORD".into(),
            })
            .to_request();

        let resp = test::call_service(&app, req).await;

        dbg!(&resp);

        assert!(resp.status().is_success());

        dbg!(resp);
    }

}