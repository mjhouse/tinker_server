use chrono::{DateTime, Utc};
use diesel::prelude::*;

// ------------------------------------------------
// Accounts
#[derive(Queryable, Insertable)]
#[diesel(table_name = crate::schema::accounts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AccountInsert {
    pub username: String,
    pub password: String,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::accounts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AccountSelect {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub created: DateTime<Utc>
}
// ------------------------------------------------

// ------------------------------------------------
// Characters
#[derive(Queryable, Insertable)]
#[diesel(table_name = crate::schema::characters)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CharacterInsert {
    pub name: String,
    pub account_id: i32,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::characters)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CharacterSelect {
    pub id: i32,
    pub name: String,
    pub created: DateTime<Utc>,
    pub account_id: i32,
}
// ------------------------------------------------

// ------------------------------------------------
// Abilities
#[derive(Queryable, Insertable)]
#[diesel(table_name = crate::schema::abilities)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AbilityInsert {
    pub name: String
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::abilities)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AbilitySelect {
    pub id: i32,
    pub name: String
}
// ------------------------------------------------

// ------------------------------------------------
// Items
#[derive(Queryable, Insertable)]
#[diesel(table_name = crate::schema::items)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ItemInsert {
    pub name: String
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::items)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ItemSelect {
    pub id: i32,
    pub name: String
}
// ------------------------------------------------