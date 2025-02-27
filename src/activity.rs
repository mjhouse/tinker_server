use actix_web::{get, web, HttpRequest, Responder};
use actix_ws::Message as AXMessage;
use crate::data::messages::*;
use crate::data::payloads::Account;
use crate::queries::Database;

pub async fn handle_move(database: Database, message: MoveMessage) -> Message {
    println!("Got MOVE:\n{:?}",message);
    Message::success("")
}

pub async fn handle_attack(database: Database, message: AttackMessage) -> Message {
    println!("Got ATTACK:\n{:?}",message);
    Message::success("")
}

#[get("/connect")]
pub async fn connect(
    pool: web::Data<Database>, 
    user: Account,
    req: HttpRequest, 
    body: web::Payload
) -> actix_web::Result<impl Responder> {

    let (response, mut session, mut stream) = actix_ws::handle(&req, body)?;
    
    actix_web::rt::spawn(async move {
        while let Some(Ok(AXMessage::Text(msg))) = stream.recv().await {
            let result = match Message::from_bytes(msg.as_bytes()) {
                Ok(Message::Move(msg)) => handle_move(pool.get_ref().clone(), msg).await,
                Ok(Message::Attack(msg)) => handle_attack(pool.get_ref().clone(), msg).await,
                Ok(Message::Result(_)) => Message::failure("Bad message"),
                Err(e) => Message::failure(e),
            };

            if session.text(result.to_string().unwrap()).await.is_err() {
                return;
            }
        }
        let _ = session.close(None).await;
    });

    Ok(response)
}


#[cfg(test)]
mod tests {
    use diesel::pg::PgConnection;
    use diesel::r2d2::ConnectionManager;
    use actix_web::App;
    use futures_util::{SinkExt as _, StreamExt as _};
    use super::*;

    macro_rules! testapp {
        ($endpoint: ident) => {{
            let url = dotenv::var("DATABASE_URL").unwrap();
            let mgr = ConnectionManager::<PgConnection>::new(url);
    
            let pool = r2d2::Pool::builder()
                .build(mgr)
                .expect("could not build connection pool");
    
            let record = queries::create_player(
                pool.clone(),
                "TEST"
            ).await.unwrap();

            let app = actix_test::start(move ||
                App::new()
                .app_data(web::Data::new(pool.clone()))
                .route("/connect", web::get().to(connect))
            );

            (app, record)
        }};
    }

    // #[actix_web::test]
    // async fn test_connect() {
    //     let url = dotenv::var("DATABASE_URL").unwrap();
    //     let mgr = ConnectionManager::<PgConnection>::new(url);

    //     let pool = r2d2::Pool::builder()
    //         .build(mgr)
    //         .expect("could not build connection pool");

    //     let mut app = actix_test::start(move ||
    //         App::new()
    //         .app_data(web::Data::new(pool.clone()))
    //         .service(connect)
    //     );

    //     let mut framed = app.ws_at("/connect").await.unwrap();

    //     framed.send(
    //         Message::Move(
    //             MoveMessage { 
    //                 token: "".into(), 
    //                 x: 0.5, 
    //                 y: 0.5 
    //             }
    //         )
    //         .to_message()
    //         .unwrap()
    //     )
    //     .await
    //     .unwrap();

    //     let item = framed.next().await.unwrap().unwrap();
    //     dbg!(item);

    //     // framed.send(AXMessage::Text("text2".into())).await.unwrap();

    //     // let item = framed.next().await.unwrap().unwrap();
    //     // dbg!(item);

    // }

}