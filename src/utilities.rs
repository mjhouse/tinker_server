use tokio::signal::unix::{signal, SignalKind};
use tokio::task;

use tinker_records::messages::Message;
use crate::queries::Database;
use crate::routes::{INCOMING_QUEUE,OUTGOING_QUEUE,DATABASE_QUEUE,all_viewed};

async fn process_message(message: Message) {
    println!("MOVED TO OUTGOING: {:?}",message);

    // 1. copy message from incoming to outgoing queue
    let message1 = message.clone();
    OUTGOING_QUEUE.lock().await.push_back(message1);

    // 2. copy message from incoming to insertion queue
    let message2 = message.clone();
    DATABASE_QUEUE.lock().await.push_back(message2);

    // 4. clear viewed messages from the outgoing queue
    let mut queue = OUTGOING_QUEUE.lock().await;
    let mut i = 0;
    while i < queue.len() {
        if !all_viewed(queue[i].id()).await {
            i += 1;
        } else {
            queue.remove(i);
        }
    }
}

pub fn process_messages(pool: Database) {
    actix_web::rt::spawn(async {

        let canceled_task = tokio::signal::ctrl_c();
    
        let terminate_task = async {
            signal(SignalKind::terminate())
                .expect("failed to install SIGTERM handler")
                .recv()
                .await;
        };
    
        let processer_task = async move {
            loop {
                task::yield_now().await;
                if let Some(message) = INCOMING_QUEUE.lock().await.pop_front() {
                    process_message(message).await;
                }
            }
        };

        let inserter_task = async move {
            // take each message from the database queue and make the necessary
            // changes to the database. How exactly they should be translated
            // without a massive `match` statement is left as an exercise for
            // future-me.

            // match DATABASE_QUEUE.lock().await.pop_front() {
            //     Some(Message::Move(m)) => {
            //         queries::update_entity(
            //             &pool, 
            //             m.character_id, 
            //             m.x, 
            //             m.y
            //         ).await;
            //     },
            //     Some(Message::Attack(m)) => {
    
            //     },
            //     _ => ()
            // }
        };
        
        tokio::select! {
            _ = canceled_task => println!("Received Ctrl+C"),
            _ = terminate_task => println!("Received SIGTERM"),
            _ = processer_task => ()
        };
    });
}

pub mod token {
    use branca::Branca;
    use once_cell::sync::Lazy;
    use serde::{Deserialize, Serialize};
    use crate::errors::Result;
   

    static SECRET: Lazy<[u8;32]> = Lazy::new(|| {
        let mut key = [0u8; 32];
        getrandom::fill(&mut key).unwrap();
        key
    });
    
    pub fn encode<T: Serialize>(value: &T) -> Result<String> {
        let string = serde_json::to_string(value)?;
        let data = string.as_bytes();
        Ok(Branca::new(SECRET.as_ref())?.encode(data)?)
    }
    
    pub fn decode<R: for<'a> Deserialize<'a>, T: AsRef<str>>(value: T) -> Result<R> {
        let data = Branca::new(SECRET.as_ref())?.decode(value.as_ref(),0)?;
        let item = serde_json::from_slice(&data)?;
        Ok(item)
    }
}

pub mod password {
    use crate::errors::Result;
    use argon2::{
        password_hash::{rand_core::OsRng, SaltString},
        Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    };

    pub fn hash<T: ToString>(value: T) -> Result<String> {
        Ok(Argon2::default()
            .hash_password(
                value.to_string().as_bytes(),
                &SaltString::generate(&mut OsRng),
            )?
            .to_string())
    }
    
    pub fn valid<T: ToString>(value: T, password: T) -> Result<()> {
        Ok(Argon2::default().verify_password(
            password.to_string().as_bytes(),
            &PasswordHash::new(&value.to_string().as_ref())?,
        )?)
    }
}
