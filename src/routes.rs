use crate::data::payloads::AccountKey;
use crate::data::messages::*;
use crate::errors::Result;
use crate::utilities;
use crate::{
    data::payloads::{AccountInfo, Login, Register},
    queries::{self, Database},
};
use actix_web::{get, post, web, HttpRequest, Responder};
use validator::Validate;

#[get("/login")]
async fn login(
    pool: web::Data<Database>,
    form: web::Json<Login>
) -> Result<impl Responder> {
    // validate the form fields
    form.validate()?;

    let username = form.username.clone();
    let password = form.password.clone();

    // fetch the database record by username
    let record = queries::fetch_account(&pool, username).await?;

    // validate the password hash
    utilities::password::valid(record.password, password)?;

    // create an authentication token from the account
    let token = utilities::token::encode(&AccountInfo {
        id: record.id,
        name: record.username.clone(),
    })?;

    // return the account information
    Ok(web::Json(AccountKey {
        id: record.id,
        name: record.username,
        token: token,
    }))
}

#[post("/register")]
async fn register(
    pool: web::Data<Database>, 
    form: web::Json<Register>
) -> Result<impl Responder> {
    // validate the form fields
    form.validate()?;

    // get the username and hash the password
    let username = form.username.clone();
    let password = utilities::password::hash(form.password1.clone())?;

    // create the database record
    let record = queries::create_account(&pool, username, password).await?;

    // return the account information
    Ok(web::Json(AccountInfo {
        id: record.id,
        name: record.username,
    }))
}



pub async fn handle_move(database: Database, message: MoveMessage) -> Message {
    println!("Got MOVE:\n{:?}", message);
    Message::success("SUCCESS")
}

pub async fn handle_attack(database: Database, message: AttackMessage) -> Message {
    println!("Got ATTACK:\n{:?}", message);
    Message::success("SUCCESS")
}

#[get("/connect/{token}")]
pub async fn connect(
    pool: web::Data<Database>,
    info: web::Path<String>,
    req: HttpRequest,
    body: web::Payload,
) -> Result<impl Responder> {
    let info: AccountInfo = utilities::token::decode(&info.into_inner())?;
    let (response, mut session, mut stream) = actix_ws::handle(&req, body)?;

    actix_web::rt::spawn(async move {

        // 1. check if there are incoming messages (figure out timeout)
        //    if there are:
        //          a. read the message and add to queue (if not heartbeat)
        //          b. read the message and respond (if heartbeat)
        //             then save time received.
        // 2. check if there are outgoing messages (figure out task id)
        //    if there are:
        //          a. send them to the connected client
        // 3. check the time received of the last heartbeat and close
        //    the connection if it's been too long.


        while let Some(Ok(actix_ws::Message::Text(msg))) = stream.recv().await {
            let result = match Message::from_bytes(msg.as_bytes()) {
                Ok(Message::Move(msg)) => handle_move(pool.get_ref().clone(), msg).await,
                Ok(Message::Attack(msg)) => handle_attack(pool.get_ref().clone(), msg).await,
                Ok(Message::Result(_)) => Message::failure("Bad message"),
                Err(e) => Message::failure(e),
            };

            if session.text(result.to_string().unwrap()).await.is_err() {
                return;
            }
        }
        let _ = session.close(None).await;
    });

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils;
    use actix_web::{test, App};
    use diesel::pg::PgConnection;
    use diesel::r2d2::ConnectionManager;
    use futures_util::{SinkExt as _, StreamExt as _};

    #[actix_web::test]
    async fn test_endpoint_register() {
        let app = test_utils::app!();

        let resp = test::call_service(
            &app,
            test::TestRequest::post()
                .uri("/register")
                .set_json(Register {
                    username: "TEST".into(),
                    password1: "PASSWORD".into(),
                    password2: "PASSWORD".into(),
                })
                .to_request(),
        )
        .await;

        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_endpoint_login() {
        let app = test_utils::app!();

        let resp = test::call_service(
            &app,
            test::TestRequest::post()
                .uri("/register")
                .set_json(Register {
                    username: "TEST".into(),
                    password1: "PASSWORD".into(),
                    password2: "PASSWORD".into(),
                })
                .to_request(),
        )
        .await;

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let account: AccountInfo = serde_json::from_slice(&body).unwrap();
        assert_eq!(account.name,"TEST".to_string());

        let resp = test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/login")
                .set_json(Login {
                    username: "TEST".into(),
                    password: "PASSWORD".into(),
                })
                .to_request(),
        )
        .await;

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let account: AccountKey = serde_json::from_slice(&body).unwrap();

        assert_eq!(account.name,"TEST".to_string());
        assert!(account.token.len() > 50);
    }

    #[actix_web::test]
    async fn test_endpoint_login_failure() {
        let app = test_utils::app!();

        let resp = test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/login")
                .set_json(Login {
                    username: "BADNAME".into(),
                    password: "PASSWORD".into(),
                })
                .to_request(),
        )
        .await;

        assert!(!resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_socket_connect() {
        let url = dotenv::var("DATABASE_URL").unwrap();
        let mgr = ConnectionManager::<PgConnection>::new(url);

        let pool = r2d2::Pool::builder()
            .build(mgr)
            .expect("could not build connection pool");

        let mut app = actix_test::start(move ||
            App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(connect)
        );

        let url = format!("/connect/{}","TEST");
        let mut framed = app.ws_at(&url).await.unwrap();

        framed.send(
            Message::Move(
                MoveMessage {
                    token: "".into(),
                    x: 0.5,
                    y: 0.5
                }
            )
            .to_message()
            .unwrap()
        )
        .await
        .unwrap();

        let item = framed.next().await.unwrap().unwrap();
        dbg!(item);

        // framed.send(AXMessage::Text("text2".into())).await.unwrap();

        // let item = framed.next().await.unwrap().unwrap();
        // dbg!(item);

    }
}
