use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use diesel::prelude::*;

#[derive(Deserialize,Serialize,Clone)]
pub struct MovementReport {
    pub player: usize,
    pub vector: [f32;3]
}

#[derive(Deserialize,Serialize,Clone)]
pub struct MovementResponse {
    pub id: i32,
    pub player: i32,
    pub vector: Vec<Option<f64>>,
    pub created: String
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::movements)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Movement {
    pub id: i32,
    pub player: i32,
    pub vector: Vec<Option<f64>>,
    pub created: NaiveDateTime
}