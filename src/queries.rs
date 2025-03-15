use tinker_records::models::{CharacterInsert, CharacterSelect};

use actix_web::web;
use chrono::{DateTime, Utc};
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use diesel::ExpressionMethods;
use diesel::{query_dsl::methods::FilterDsl, RunQueryDsl};

pub type Database = r2d2::Pool<ConnectionManager<PgConnection>>;

pub async fn create_character<T: ToString>(
    database: &Database,
    username: T,
    password: T,
) -> diesel::QueryResult<CharacterSelect> {
    let username = username.to_string();
    let password = password.to_string();
    let mut conn = database.get().expect("No database");
    web::block(move || {
        use tinker_records::schema::characters::dsl;

        // insert the model into the database
        diesel::insert_into(dsl::characters)
            .values(CharacterInsert { username, password }) 
            .get_result::<CharacterSelect>(&mut conn)
    })
    .await
    .unwrap()
}

pub async fn fetch_character<T: ToString>(
    database: &Database,
    username: T,
) -> diesel::QueryResult<CharacterSelect> {
    let username = username.to_string();
    let mut conn = database.get().expect("No database");
    web::block(move || {
        use tinker_records::schema::characters::dsl;

        dsl::characters
            .filter(dsl::username.eq(username)) 
            .get_result(&mut conn)
    })
    .await
    .unwrap()
}

pub async fn modified_entities(
    database: &Database,
    character_id: i32, 
    timestamp: DateTime<Utc>
) -> diesel::QueryResult<Vec<CharacterSelect>> {
    let mut conn = database.get().expect("No database");
    web::block(move || {
        use tinker_records::schema::characters::dsl;

        dsl::characters
            // .filter(dsl::id.eq(character_id))
            .filter(dsl::modified.gt(timestamp))
            .get_results(&mut conn)
    })
    .await
    .unwrap()
}

pub async fn local_entities(
    database: &Database,
    character_id: i32,
    connected_ids: Vec<i32>,
) -> diesel::QueryResult<Vec<CharacterSelect>> {
    let mut conn = database.get().expect("No database");
    web::block(move || {
        use tinker_records::schema::characters::dsl;
        dsl::characters
            .filter(dsl::id.eq_any(connected_ids))
            .filter(dsl::id.ne(character_id))
            .get_results(&mut conn)
    })
    .await
    .unwrap()
}

pub async fn update_entity(
    database: &Database,
    character_id: i32,
    x: f32,
    y: f32,
) {
    let mut conn = database.get().expect("No database");
    web::block(move || {
        use tinker_records::schema::characters::dsl;

        diesel::update(dsl::characters
            .filter(dsl::id.eq(character_id)))
            .set((dsl::x.eq(x),dsl::y.eq(y)))
            .execute(&mut conn)
            .unwrap();
    })
    .await
    .unwrap() 
}

#[cfg(test)]
mod tests {
    use crate::test_utils;

    use super::*;

    #[actix_web::test]
    async fn test_create_character() {
        let database = "test_create_account";
        test_utils::setup(database).await; 
        let pool = test_utils::pool(database).await;

        let result = create_character(&pool, "TEST", "PASSWORD").await;
        assert!(result.is_ok());

        let record = result.unwrap();
        assert_eq!(record.username, "TEST");
        assert_eq!(record.password, "PASSWORD");

        test_utils::teardown(database);
    }

}
