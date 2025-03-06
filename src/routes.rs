use std::collections::VecDeque;

use crate::data::payloads::{AccountKey, CreateCharacterForm, FetchCharactersForm, SelectCharacterForm};
use crate::data::messages::*;
use crate::errors::{Error, Result};
use crate::schema::characters::account_id;
use crate::utilities;
use crate::{
    data::payloads::{AccountInfo, Login, Register},
    queries::{self, Database},
};
use actix_web::{get, post, web, HttpRequest, Responder};
use futures_util::lock::Mutex;
use futures_util::TryStreamExt;
use once_cell::sync::Lazy;
use validator::Validate;

static MESSAGE_QUEUE: Lazy<Mutex<VecDeque<Message>>> = Lazy::new(|| { Default::default() });

#[post("/characters")]
async fn create_character(
    pool: web::Data<Database>,
    form: web::Json<CreateCharacterForm>,
) -> Result<impl Responder> {
    // validate the form fields
    form.validate()?;

    // decode the account information
    let info: AccountInfo = utilities::token::decode(
        form.token.clone()
    )?;

    // fetch all of the characters for the account
    Ok(web::Json(queries::create_character(
        &pool,
        form.name.clone(),
        info.id
    ).await?))
}

#[get("/characters")]
async fn fetch_characters(
    pool: web::Data<Database>,
    form: web::Json<FetchCharactersForm>,
) -> Result<impl Responder> {
    // validate the form fields
    form.validate()?;

    // decode the account information
    let info: AccountInfo = utilities::token::decode(
        form.token.clone()
    )?;

    // fetch all of the characters for the account
    Ok(web::Json(queries::fetch_characters(
        &pool, 
        info.id
    ).await?))
}

#[get("/character")]
async fn select_character(
    pool: web::Data<Database>,
    form: web::Json<SelectCharacterForm>,
) -> Result<impl Responder> {
    // validate the form fields
    form.validate()?;

    // decode the account information
    let mut info: AccountInfo = utilities::token::decode(
        form.token.clone()
    )?;

    // make sure the character actually exists
    let character = queries::fetch_character(
        &pool, 
        info.id,
        form.character_id
    ).await?;

    // update the character id for the session
    info.character_id = Some(character.id);

    let token = utilities::token::encode(&info)?;

    Ok(web::Json(AccountKey {
        id: info.id,
        name: info.name,
        token: token,
    }))
}

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
    let account = queries::fetch_account(&pool, username).await?;

    let character = queries::fetch_characters(&pool, account.id).await?
        .iter()
        .next()
        .cloned()
        .ok_or(Error::NoCharacter)?;

    // validate the password hash
    utilities::password::valid(account.password, password)?;

    // create an authentication token from the account
    let token = utilities::token::encode(&AccountInfo {
        id: account.id,
        name: account.username.clone(),
        character_id: Some(character.id)
    })?;

    // return the account information
    Ok(web::Json(AccountKey {
        id: account.id,
        name: account.username,
        token: token,
    }))
}

#[post("/register")]
async fn register(
    pool: web::Data<Database>, 
    form: web::Json<Register>
) -> Result<impl Responder> {

    println!("IN REGISTER");
    dbg!(&form);

    // validate the form fields
    form.validate()?;

    // get the username and hash the password
    let username = form.username.clone();
    let password = utilities::password::hash(form.password1.clone())?;

    // create the database record
    let account = queries::create_account(&pool, username, password).await?;
    let character = queries::create_character(&pool, account.username.clone(), account.id).await?;

    // return the account information
    Ok(web::Json(AccountInfo {
        id: account.id,
        name: account.username,
        character_id: Some(character.id)
    }))
}

#[get("/connect/{token}")]
pub async fn connect(
    pool: web::Data<Database>,
    token: web::Path<String>,
    req: HttpRequest,
    body: web::Payload,
) -> Result<impl Responder> {

    // get the character information for the connecting account
    let info: AccountInfo = utilities::token::decode(&token.into_inner())?;
    let character_id = info.character_id.ok_or(Error::NoCharacter)?;
    let mut character = queries::fetch_character(&pool, info.id, character_id).await?;

    let (response, mut session, mut stream) = actix_ws::handle(&req, body)?;


    actix_web::rt::spawn(async move {

        // check for incoming messages
        loop {
            match stream.try_next().await {
                // if there are messages waiting add them to queue
                Ok(Some(actix_ws::Message::Text(text))) => {
                    if let Ok(message) = Message::from_bytes(text.as_bytes()) {
                        if let Message::Move(m) = message {
                            character.x = m.x;
                            character.y = m.y;
                            MESSAGE_QUEUE.lock().await.push_back(Message::Move(m));
                        } else {
                            MESSAGE_QUEUE.lock().await.push_back(message);
                        }
                    }
                },
                // if there was an error, close the session
                Err(_) => { 
                    let _ = session.close(None).await; 
                    break; 
                }
                // if Ok(None) or invalid message, ignore
                _ => break,
            }
        }

        // TODO: get modified records from database around character location
        //       using: https://github.com/ThinkAlexandria/diesel_geometry
        // let entities = queries::modified_entities(database, character.position)?;

        // TODO: for each modified record, broadcast them to the client
        // let data = serde_json::to_string(entities)?;
        // if session.text(data).await.is_err() {
        //     return; // and maybe close here?
        // }

    });

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{data::models::CharacterSelect, test_utils};
    use actix_web::{test, App};
    use diesel::pg::PgConnection;
    use diesel::r2d2::ConnectionManager;
    use futures_util::{SinkExt as _, StreamExt as _};

    mod query {

        #[macro_export]
        macro_rules! post {
            ($app: ident, $path: expr, $record: expr) => {
                test::call_service(
                    &$app,
                    test::TestRequest::post()
                        .uri($path)
                        .set_json($record)
                        .to_request(),
                )
                .await
            };
        }

        #[macro_export]
        macro_rules! get {
            ($app: ident, $path: expr, $record: expr) => {
                test::call_service(
                    &$app,
                    test::TestRequest::get()
                        .uri($path)
                        .set_json($record)
                        .to_request(),
                )
                .await
            };
        }

        pub(crate) use post;
        pub(crate) use get;
    }
    

    #[actix_web::test]
    async fn test_endpoint_register1() {
        let app = test_utils::setup("test_endpoint_register1").await;

        let resp = query::post!(app,"/register",Register {
            username: "TEST".into(),
            password1: "PASSWORD".into(),
            password2: "PASSWORD".into(),
        });

        assert!(resp.status().is_success());

        test_utils::teardown("test_endpoint_register1");
    }

    #[actix_web::test]
    async fn test_endpoint_register2() {
        let app = test_utils::setup("test_endpoint_register2").await;

        let resp = query::post!(app,"/register",Register {
            username: "BAD".into(), // must be >=4 chars
            password1: "PASSWORD".into(),
            password2: "PASSWORD".into(),
        });

        assert!(!resp.status().is_success());

        test_utils::teardown("test_endpoint_register2");
    }

    #[actix_web::test]
    async fn test_endpoint_register3() {
        let app = test_utils::setup("test_endpoint_register3").await;

        let resp = query::post!(app,"/register",Register {
            username: "USERNAME".into(), // must be unique
            password1: "PASSWORD".into(),
            password2: "PASSWORD".into(),
        });

        assert!(!resp.status().is_success());

        test_utils::teardown("test_endpoint_register3");
    }

    #[actix_web::test]
    async fn test_endpoint_login1() {
        let app = test_utils::setup("test_endpoint_login1").await;

        let resp = query::get!(app,"/login",Login {
            username: "USERNAME".into(),
            password: "PASSWORD".into(),
        });

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let account: AccountKey = serde_json::from_slice(&body).unwrap();

        assert_eq!(account.name,"USERNAME");
        assert!(account.token.len() > 50);

        test_utils::teardown("test_endpoint_login1");
    }

    #[actix_web::test]
    async fn test_endpoint_login2() {
        let app = test_utils::setup("test_endpoint_login2").await;

        // fails because the username doesn't exist
        let resp = query::get!(app,"/login",Login {
            username: "BADNAME".into(),
            password: "PASSWORD".into(),
        });

        assert!(!resp.status().is_success());

        test_utils::teardown("test_endpoint_login2");
    }

    #[actix_web::test]
    async fn test_endpoint_login3() {
        let app = test_utils::setup("test_endpoint_login3").await;

        // fails because the password is wrong
        let resp = query::get!(app,"/login",Login {
            username: "USERNAME".into(),
            password: "BADPASSWORD".into(),
        });

        assert!(!resp.status().is_success());

        test_utils::teardown("test_endpoint_login3");
    }

    #[actix_web::test]
    async fn test_endpoint_create_character() {
        let app = test_utils::setup("test_endpoint_create_character").await;

        let token = utilities::token::encode(&AccountInfo {
            id: 1, // default preloaded account
            name: "USERNAME".into(),
            character_id: None
        }).unwrap();

        // create new a character
        let resp = query::post!(app,"/characters",CreateCharacterForm {
            token,
            name: "NAME".to_string()
        });
        
        assert!(resp.status().is_success());

        test_utils::teardown("test_endpoint_create_character");
    }

    #[actix_web::test]
    async fn test_endpoint_fetch_characters() {
        let app = test_utils::setup("test_endpoint_fetch_characters").await;

        let token = utilities::token::encode(&AccountInfo {
            id: 1, // default preloaded account
            name: "USERNAME".into(),
            character_id: None
        }).unwrap();

        // fetch all characters
        let resp = query::get!(app,"/characters", FetchCharactersForm { token });

        let body = test::read_body(resp).await;
        let characters: Vec<CharacterSelect> = serde_json::from_slice(&body).unwrap();
        assert_eq!(characters.len(),1);

        test_utils::teardown("test_endpoint_fetch_characters");
    }

    #[actix_web::test]
    async fn test_endpoint_select_character() {
        let app = test_utils::setup("test_endpoint_select_character").await;

        let token = utilities::token::encode(&AccountInfo {
            id: 1, // default preloaded account
            name: "USERNAME".into(),
            character_id: None
        }).unwrap();

        // get updated token with character selected
        let resp = query::get!(app,"/character", SelectCharacterForm { 
            token,
            character_id: 1 // default preloaded character 
        });
        
        let body = test::read_body(resp).await;
        let key: AccountKey = serde_json::from_slice(&body).unwrap();
        let info: AccountInfo = utilities::token::decode(key.token).unwrap();

        dbg!(info);

        test_utils::teardown("test_endpoint_select_character");
    }

    // #[actix_web::test]
    // async fn test_socket_connect() {
    //     let url = dotenv::var("DATABASE_URL").unwrap();
    //     let mgr = ConnectionManager::<PgConnection>::new(url);

    //     let pool = r2d2::Pool::builder()
    //         .build(mgr)
    //         .expect("could not build connection pool");

    //     let mut app = actix_test::start(move ||
    //         App::new()
    //         .app_data(web::Data::new(pool.clone()))
    //         .service(connect)
    //     );

    //     let url = format!("/connect/{}","TEST");
    //     let mut framed = app.ws_at(&url).await.unwrap();

    //     framed.send(
    //         Message::Move(
    //             MoveMessage {
    //                 token: "".into(),
    //                 x: 0.5,
    //                 y: 0.5
    //             }
    //         )
    //         .to_message()
    //         .unwrap()
    //     )
    //     .await
    //     .unwrap();

    //     let item = framed.next().await.unwrap().unwrap();
    //     dbg!(item);

    //     // framed.send(AXMessage::Text("text2".into())).await.unwrap();

    //     // let item = framed.next().await.unwrap().unwrap();
    //     // dbg!(item);

    // }
}
