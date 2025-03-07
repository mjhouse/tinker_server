use std::time::Duration;

use tokio::signal::unix::{signal, SignalKind};
use tokio::task;

use crate::data::payloads::AccountInfo;
use crate::queries;
use crate::{data::messages::Message, queries::Database};
use crate::routes::INCOMING_QUEUE;

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
                match INCOMING_QUEUE.lock().await.pop_front() {
                    Some(Message::Move(m)) => {
                        
                        if let Ok(info) = token::decode::<AccountInfo,String>(m.token) {

                            let id = info.character_id.unwrap();
                            println!("{} -> ({},{})",id,m.x,m.y);

                            queries::update_entity(
                                &pool, 
                                id, 
                                m.x, 
                                m.y
                            ).await;
                        }

                    },
                    Some(Message::Attack(m)) => {
        
                    },
                    _ => ()
                }
            }
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
