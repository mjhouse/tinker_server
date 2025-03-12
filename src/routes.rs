use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::Duration;

use crate::data::models::CharacterSelect;
use crate::data::payloads::{AccountKey, CreateCharacterForm, FetchCharactersForm, SelectCharacterForm};
use crate::data::messages::*;
use crate::errors::{Error, Result};
use crate::utilities;
use crate::{
    data::payloads::{AccountInfo, Login, Register},
    queries::{self, Database},
};
use actix_web::{get, post, web, HttpRequest, Responder};
use futures_util::lock::Mutex;
use futures_util::StreamExt;
use futures_util::stream;
use once_cell::sync::Lazy;
use tokio::time::sleep;
use uuid::Uuid;
use validator::Validate;

pub static INCOMING_QUEUE: Lazy<Mutex<VecDeque<Message>>> = Lazy::new(|| { Default::default() });
pub static OUTGOING_QUEUE: Lazy<Mutex<VecDeque<Message>>> = Lazy::new(|| { Default::default() });
pub static DATABASE_QUEUE: Lazy<Mutex<VecDeque<Message>>> = Lazy::new(|| { Default::default() });

// TODO: merge USERS and REGISTRY. They should both just use the account id.
pub static REGISTRY: Lazy<Mutex<HashMap<i32,AccountInfo>>> = Lazy::new(|| { Default::default() });
pub static VIEWED: Lazy<Mutex<HashMap<Uuid,Vec<i32>>>> = Lazy::new(|| { Default::default() });

async fn get_initial(pool: &Database, account: AccountInfo) -> Vec<CharacterSelect> {
    let connected = REGISTRY
        .lock()
        .await
        .values()
        .filter_map(|a| a.character_id.clone())
        .collect::<Vec<i32>>();
    queries::local_entities(pool, account.character_id.unwrap(), connected).await.unwrap_or_default()
}

pub async fn register_handler(account: AccountInfo) -> i32 {
    println!("{} REGISTERED",account.id);
    REGISTRY.lock().await.insert(account.id,account.clone());
    account.id
}

pub async fn unregister_handler(id: i32) {
    println!("{} UNREGISTERED",id);
    REGISTRY.lock().await.remove(&id);
}

pub async fn registered_handler(account_id: i32) -> bool {
    REGISTRY.lock().await.contains_key(&account_id)
}

// mark a message as viewed by a particular handler
pub async fn set_viewed(account_id: i32, message_id: Uuid) {
    VIEWED.lock().await
        .entry(message_id)
        .or_insert_with(Vec::new)
        .push(account_id);
}

// check if a message is viewed by a particular handler
pub async fn get_viewed(account_id: i32, message_id: Uuid) -> bool {
    VIEWED.lock().await
        .entry(message_id)
        .or_insert_with(Vec::new)
        .contains(&account_id)
}

// check if the message has been viewed by all handlers
pub async fn all_viewed(message_id: Uuid) -> bool {
    let viewed = VIEWED.lock().await;
    REGISTRY.lock().await
        .keys()
        .all(|item| viewed
            .get(&message_id)
            .map(|v| v.contains(item))
            .unwrap_or(false))
}

// read all un-viewed messages for a particular handler (and set viewed)
pub async fn read_messages(account_id: i32) -> Vec<Message> {
    stream::iter(OUTGOING_QUEUE.lock().await.iter())
        .filter_map(|item| async move {
            if !get_viewed(account_id, item.id()).await {
                set_viewed(account_id, item.id()).await;
                Some(item.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .await
}

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
    
    let (response, mut session, mut stream) = actix_ws::handle(&req, body)?;

    actix_web::rt::spawn(async move {

        // keep track of incoming messages so we don't send back 
        // the messages we received.
        let mut messages: HashSet<Uuid> = HashSet::new();

        // decode the login token to get basic account information
        let account: AccountInfo = utilities::token::decode(token.to_string())
            .expect("Could not decode token");

        let character: CharacterSelect = queries::fetch_character(
            &pool, 
            account.id, 
            account.character_id.expect("No character id")
        ).await.expect("Could not find character");
    
        // the id for this particular connection
        let handler_id = register_handler(account.clone()).await;

        // 1. get current records for entities that are-
        //      - connected
        //      - in range
        let entities = get_initial(&pool,account.clone()).await;
        
        // 2. build an "InitialState" message for the client
        let item = Message::Initial(InitialMessage {
            token: token.clone(),
            id: Uuid::now_v7(),
            entities
        });

        // 3. send the initial state message to the client
        if let Ok(data) = serde_json::to_string(&item) {
            let _ = session.text(data).await.map_err(|_| {
                // TODO: log failure and maybe disconnect
            });
        }

        let id = Uuid::now_v7();
        messages.insert(id.clone());

        INCOMING_QUEUE.lock().await.push_back(Message::Connect(ConnectMessage { 
            token: token.clone(), 
            entity: character,
            id
        }));

        loop {
            // create timeout and stream futures
            let timeout = sleep(Duration::from_millis(100));
            let source = stream.next();

            // create a combined future that waits for either
            // the next message or a timeout, whichever is sooner.
            let result = tokio::select! {
                message = source => message,
                _ = timeout => None
            };

            // handle incoming messages
            match result {
                Some(Ok(actix_ws::Message::Text(text))) => {
                    if let Ok(m) = Message::from_bytes(text.as_bytes()) {
                        // track incoming so we don't send them back
                        messages.insert(m.id());
                        // enqueue for database insertion and response
                        INCOMING_QUEUE.lock().await.push_back(m.clone());
                    }
                },
                Some(_) => {
                    // close session and unregister handler
                    let _ = session.close(None).await;
                    unregister_handler(handler_id).await;
                    break;                    
                },
                None => ()
            }

            // read all un-viewed messages (marking as viewed) and send them
            for item in read_messages(handler_id).await {
                // if it wasn't a message we received from this conenction, then send it
                if !messages.remove(&item.id()) {
                    if let Ok(data) = serde_json::to_string(&item) {
                        let _ = session.text(data).await.map_err(|_| {
                            // TODO: log failure and maybe disconnect
                        });
                    }
                }
            }

        }
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
