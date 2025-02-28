use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    Move(MoveMessage),
    Attack(AttackMessage),
    Result(ResultMessage),
}

impl Message {
    pub fn success<T: ToString>(v: T) -> Self {
        Self::Result(ResultMessage {
            error: false,
            text: v.to_string(),
        })
    }

    pub fn failure<T: ToString>(v: T) -> Self {
        Self::Result(ResultMessage {
            error: true,
            text: v.to_string(),
        })
    }

    pub fn to_string(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }

    pub fn from_bytes<'a>(bytes: &'a [u8]) -> serde_json::Result<Self> {
        serde_json::from_slice::<Self>(bytes)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MoveMessage {
    pub token: String,
    pub x: f64,
    pub y: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AttackMessage {
    pub token: String,
    pub target: i32,
    pub ability: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResultMessage {
    pub error: bool,
    pub text: String,
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
