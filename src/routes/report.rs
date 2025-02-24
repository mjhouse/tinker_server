use actix_web::{post, web, Responder, Result};
use crate::routes::models::MovementReport;
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;

type Database = r2d2::Pool<ConnectionManager<PgConnection>>;

#[post("/movement")]
async fn movement(
    pool: web::Data<Database>,
    info: web::Json<MovementReport>
) -> Result<impl Responder> {
    Ok(info)
}

#[cfg(test)]
mod tests {
    use actix_web::{test, App};
    use super::*;

    
    #[actix_web::test]
    async fn test_index_post() {
        let url = "postgresql://user:password@localhost:5432/game";
        let mgr = ConnectionManager::<PgConnection>::new(url);
    
        let pool = r2d2::Pool::builder()
            .build(mgr)
            .expect("could not build connection pool");

        let app = test::init_service(App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(movement)).await;

        let req = test::TestRequest::post()
        .uri("/movement")
        .set_json(MovementReport {
            player: 1,
            vector: [0.,0.,0.],
        }).to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

}