
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
