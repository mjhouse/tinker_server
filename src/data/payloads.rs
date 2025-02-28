use actix_jwt_auth_middleware::FromRequest;
use serde::{Deserialize, Serialize};
use validator::Validate;

// ------------------------------------------------
// Forms
#[derive(Deserialize, Serialize, Clone, Validate)]
pub struct Register {
    #[validate(length(min = 4, max = 32), does_not_contain(pattern = " "))]
    pub username: String,
    #[validate(
        length(min = 8, max = 256),
        must_match(other = "password2"),
        does_not_contain(pattern = " ")
    )]
    pub password1: String,
    #[validate(
        length(min = 8, max = 256),
        must_match(other = "password1"),
        does_not_contain(pattern = " ")
    )]
    pub password2: String,
}

#[derive(Deserialize, Serialize, Clone, Validate)]
pub struct Login {
    #[validate(length(min = 4, max = 32), does_not_contain(pattern = " "))]
    pub username: String,
    #[validate(length(min = 8, max = 256), does_not_contain(pattern = " "))]
    pub password: String,
}
// ------------------------------------------------

// ------------------------------------------------
// JSWToken
#[derive(Serialize, Deserialize, Clone, Debug, FromRequest)]
pub struct AccountInfo {
    pub id: i32,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AccountKey {
    pub id: i32,
    pub name: String,
    pub token: String
}
// ------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_web::test]
    async fn test_validate_login1() {
        // all fields are correct
        let form = Login {
            username: "TEST".into(),
            password: "PASSWORD".into(),
        };
        assert!(form.validate().is_ok());
    }

    #[actix_web::test]
    async fn test_validate_login2() {
        // username is too short
        let form = Login {
            username: "TES".into(),
            password: "PASSWORD".into(),
        };
        assert!(form.validate().is_err());
    }

    #[actix_web::test]
    async fn test_validate_login3() {
        // password is too short
        let form = Login {
            username: "TEST".into(),
            password: "PASS".into(),
        };
        assert!(form.validate().is_err());
    }

    #[actix_web::test]
    async fn test_validate_login4() {
        // password contains a space
        let form = Login {
            username: "TEST".into(),
            password: "PASS WORD".into(),
        };
        assert!(form.validate().is_err());
    }

    #[actix_web::test]
    async fn test_validate_login5() {
        // username contains a space
        let form = Login {
            username: "TEST NAME".into(),
            password: "PASSWORD".into(),
        };
        assert!(form.validate().is_err());
    }

    #[actix_web::test]
    async fn test_validate_register1() {
        // all fields are correct
        let form = Register {
            username: "TEST".into(),
            password1: "PASSWORD".into(),
            password2: "PASSWORD".into(),
        };
        assert!(form.validate().is_ok());
    }

    #[actix_web::test]
    async fn test_validate_register2() {
        // name too short
        let form = Register {
            username: "TES".into(),
            password1: "PASSWORD".into(),
            password2: "PASSWORD".into(),
        };
        assert!(form.validate().is_err());
    }

    #[actix_web::test]
    async fn test_validate_register3() {
        // passwords not equal
        let form = Register {
            username: "TEST".into(),
            password1: "PASSWORD1".into(),
            password2: "PASSWORD2".into(),
        };
        assert!(form.validate().is_err());
    }

    #[actix_web::test]
    async fn test_validate_register4() {
        // passwords are too short
        let form = Register {
            username: "TEST".into(),
            password1: "PASS".into(),
            password2: "PASS".into(),
        };
        assert!(form.validate().is_err());
    }

    #[actix_web::test]
    async fn test_validate_register5() {
        // passwords contain a space
        let form = Register {
            username: "TEST".into(),
            password1: "PASS WORD".into(),
            password2: "PASS WORD".into(),
        };
        assert!(form.validate().is_err());
    }

    #[actix_web::test]
    async fn test_validate_register6() {
        // username contains a space
        let form = Register {
            username: "TEST NAME".into(),
            password1: "PASSWORD".into(),
            password2: "PASSWORD".into(),
        };
        assert!(form.validate().is_err());
    }
}
