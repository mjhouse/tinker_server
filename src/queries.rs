use crate::data::models::{AccountInsert, AccountSelect, CharacterInsert, CharacterSelect};
use actix_web::web;
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use diesel::ExpressionMethods;
use diesel::{query_dsl::methods::FilterDsl, RunQueryDsl};

pub type Database = r2d2::Pool<ConnectionManager<PgConnection>>;

pub async fn create_account<T: ToString>(
    database: &Database,
    username: T,
    password: T,
) -> diesel::QueryResult<AccountSelect> {
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
    .await
    .unwrap()
}

pub async fn fetch_account<T: ToString>(
    database: &Database,
    username: T,
) -> diesel::QueryResult<AccountSelect> {
    let username = username.to_string();
    let mut conn = database.get().expect("No database");
    web::block(move || {
        use crate::schema::accounts::dsl;

        dsl::accounts
            .filter(dsl::username.eq(username))
            .first(&mut conn)
    })
    .await
    .unwrap()
}

pub async fn create_character<T: ToString>(
    database: &Database,
    name: T,
    account_id: i32,
) -> diesel::QueryResult<CharacterSelect> {
    let name = name.to_string();
    let mut conn = database.get().expect("No database");
    web::block(move || {
        use crate::schema::characters::dsl;

        // insert the model into the database
        diesel::insert_into(dsl::characters)
            .values(CharacterInsert { name, account_id })
            .get_result::<CharacterSelect>(&mut conn)
    })
    .await
    .unwrap()
}

pub async fn fetch_characters(
    database: &Database,
    account_id: i32,
) -> diesel::QueryResult<Vec<CharacterSelect>> {
    let mut conn = database.get().expect("No database");
    web::block(move || {
        use crate::schema::characters::dsl;

        dsl::characters
            .filter(dsl::account_id.eq(account_id))
            .get_results(&mut conn)
    })
    .await
    .unwrap()
}

#[cfg(test)]
mod tests {
    use crate::test_utils;

    use super::*;

    #[actix_web::test]
    async fn test_create_account() {
        let database = "test_create_account";
        test_utils::setup(database).await;
        let pool = test_utils::pool(database).await;

        let result = create_account(&pool, "TEST", "PASSWORD").await;
        assert!(result.is_ok());

        let record = result.unwrap();
        assert_eq!(record.username, "TEST");
        assert_eq!(record.password, "PASSWORD");

        test_utils::teardown(database);
    }

    #[actix_web::test]
    async fn test_create_character() {
        let database = "test_create_character";
        test_utils::setup(database).await;
        let pool = test_utils::pool(database).await;

        let result = create_character(&pool, "NAME", 1).await;
        assert!(result.is_ok());

        let record = result.unwrap();
        assert_eq!(record.name, "NAME");
        assert_eq!(record.account_id, 1);

        test_utils::teardown(database);
    }
}
