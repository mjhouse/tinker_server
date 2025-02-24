use serde::{Deserialize, Serialize};
use time::PrimitiveDateTime;
use diesel::prelude::*;

#[derive(Deserialize,Serialize)]
pub struct MovementReport {
    pub player: usize,
    pub vector: [f32;3]
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::movements)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Movement {
    pub id: i32,
    pub player: i32,
    pub vector: Vec<Option<f64>>,
    pub created: PrimitiveDateTime
}