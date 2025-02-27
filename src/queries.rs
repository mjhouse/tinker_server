use actix_web::web;
use diesel::RunQueryDsl;
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use crate::data::models::{
    AccountInsert, AccountSelect, CharacterInsert, CharacterSelect
};

pub type Database = r2d2::Pool<ConnectionManager<PgConnection>>;

pub async fn create_account<T: ToString>(database: &Database, username: T, password: T) -> diesel::QueryResult<AccountSelect> {
    let username = username.to_string();
    let password = password.to_string();
    let mut conn = database.get().expect("No database");
    web::block(move || {
        use crate::schema::accounts::dsl;

        // insert the model into the database
        diesel::insert_into(dsl::accounts)
            .values(AccountInsert { username, password })
            .get_result::<AccountSelect>(&mut conn)
    })
    .await.unwrap()
}

pub async fn create_character<T: ToString>(database: &Database, name: T, account_id: i32) -> diesel::QueryResult<CharacterSelect> {
    let name = name.to_string();
    let mut conn = database.get().expect("No database");
    web::block(move || {
        use crate::schema::characters::dsl;

        // insert the model into the database
        diesel::insert_into(dsl::characters)
            .values(CharacterInsert { name, account_id })
            .get_result::<CharacterSelect>(&mut conn)
    })
    .await.unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! testdb {
        () => {{
            let url = dotenv::var("DATABASE_URL").unwrap();
            let mgr = ConnectionManager::<PgConnection>::new(url);
    
            r2d2::Pool::builder()
                .build(mgr)
                .expect("could not build connection pool")
        }};
    }

    #[actix_web::test]
    async fn test_create_account() {
        let pool = testdb!();

        let result = create_account(&pool, "USERNAME", "PASSWORD").await;
        assert!(result.is_ok());

        let record = result.unwrap();
        assert_eq!(record.username,"USERNAME");
        assert_eq!(record.password,"PASSWORD");
    }

    #[actix_web::test]
    async fn test_create_character() {
        let pool = testdb!();

        let result = create_account(&pool, "USERNAME", "PASSWORD").await;
        assert!(result.is_ok());

        let account = result.unwrap();

        let result = create_character(&pool, "NAME", account.id).await;
        assert!(result.is_ok());

        let record = result.unwrap();
        assert_eq!(record.name,"NAME");
        assert_eq!(record.account_id,account.id);
    }


}