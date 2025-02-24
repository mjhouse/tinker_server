use actix_web::{error, post, web, Responder, Result};
use diesel::RunQueryDsl;
use crate::routes::models::{Movement, MovementReport, MovementResponse};
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use diesel::query_dsl::QueryDsl;
use diesel::ExpressionMethods;
use chrono;

type Database = r2d2::Pool<ConnectionManager<PgConnection>>;

fn insert_movement(
    conn: &mut PgConnection,
    report: MovementReport,
) -> diesel::QueryResult<Movement> {
    use crate::schema::movements::dsl::*;
    let vector_value = report.vector.into_iter().map(|v| Some(v as f64)).collect();
    let created_value = chrono::offset::Utc::now().naive_utc();

    // Create insertion model
    let record = Movement {
        id: 1,
        player: 1,
        vector: vector_value,
        created: created_value,
    };

    // normal diesel operations
    diesel::insert_into(movements)
        .values(&record)
        .execute(conn)
        .expect("Error inserting movement");

    let record = movements
        .filter(id.eq(1))
        .first::<Movement>(conn)
        .expect("Error loading person that was just inserted");

    Ok(record)
}

#[post("/movement")]
async fn movement(
    pool: web::Data<Database>,
    info: web::Json<MovementReport>
) -> Result<impl Responder> {
    let newinfo = info.clone();

    let record = web::block(move || {
        let mut conn = pool.get().expect("couldn't get db connection from pool");
        insert_movement(&mut conn, newinfo)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;

    let response = MovementResponse {
        id: record.id,
        player: record.player,
        vector: record.vector,
        created: "".into()
    };

    Ok(web::Json(response))
}

#[cfg(test)]
mod tests {
    use actix_web::{test, App};
    use super::*;

    macro_rules! testapp {
        ($endpoint: ident) => {{
            let url = "postgresql://user:password@localhost:5432/game";
            let mgr = ConnectionManager::<PgConnection>::new(url);
        
            let pool = r2d2::Pool::builder()
                .build(mgr)
                .expect("could not build connection pool");
    
            let app = test::init_service(App::new()
                .app_data(web::Data::new(pool.clone()))
                .service($endpoint)).await;
        
            app
        }};
    }

    #[actix_web::test]
    async fn test_index_post() {
        let app = testapp!(movement);
        let data = MovementReport {
            player: 1,
            vector: [0.,0.,0.],
        };

        let req = test::TestRequest::post()
            .uri("/movement")
            .set_json(data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

}