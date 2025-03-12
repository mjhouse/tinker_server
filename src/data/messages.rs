use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::models::CharacterSelect;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Message {
    Move(MoveMessage),
    Attack(AttackMessage),
    Initial(InitialMessage),
    Connect(ConnectMessage),
}

impl Message {

    pub fn id(&self) -> Uuid {
        match self {
            Message::Move(m) => m.id,
            Message::Attack(m) => m.id,
            Message::Initial(m) => m.id,
            Message::Connect(m) => m.id,
        }
    }

    pub fn to_string(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }

    pub fn from_bytes<'a>(bytes: &'a [u8]) -> serde_json::Result<Self> {
        serde_json::from_slice::<Self>(bytes)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct InitialMessage {
    pub token: String,
    pub id: Uuid,
    pub entities: Vec<CharacterSelect>
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ConnectMessage {
    pub token: String,
    pub id: Uuid,
    pub entity: CharacterSelect
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MoveMessage {
    pub token: String,
    pub id: Uuid,
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AttackMessage {
    pub token: String,
    pub id: Uuid,
    pub target: i32,
    pub ability: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_ws::Message as AXMessage;
    use serde_json;

    impl Message {
        pub fn to_message(&self) -> serde_json::Result<AXMessage> {
            self.to_string().map(|s| AXMessage::Text(s.into()))
        }
    }

    #[actix_web::test]
    async fn test_serialize_message() {
        let message = serde_json::to_string(&Message::Move(MoveMessage {
            token: "test".to_string(),
            id: Uuid::now_v7(),
            x: 0.5,
            y: 0.5,
        }));

        dbg!(&message);
        assert!(message.is_ok());
    }

    #[actix_web::test]
    async fn test_deserialize_message() {
        let value = "{\"Move\":{\"token\":\"test\",\"x\":0.5,\"y\":0.5}}";
        let message = serde_json::from_str::<Message>(value);

        dbg!(&message);
        assert!(message.is_ok());
    }
}
