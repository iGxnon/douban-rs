// 定义了最顶层的实体的抽象

use crate::model::UserId::{UserIdStr, UserIdU64};
use serde::{Deserialize, Serialize};
use std::fmt::Formatter;
use std::num::ParseIntError;
use std::str::FromStr;
use uuid::Uuid;

// 通用的返回类型
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Resp<Data> {
    code: usize,
    msg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Data>,
}

impl Default for Resp<()> {
    fn default() -> Self {
        Resp {
            code: 0,
            msg: String::new(),
            data: None,
        }
    }
}

// 用户 Id
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[serde(untagged)]
pub enum UserId {
    UserIdStr(String),
    UserIdU64(u64),
}

impl Default for UserId {
    fn default() -> Self {
        UserIdU64(0)
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

impl UserId {
    pub fn as_string(&self) -> String {
        match self {
            UserIdStr(id) => id.to_owned(),
            UserIdU64(id) => id.to_string(),
        }
    }

    pub fn as_u64(&self) -> Result<u64, ParseIntError> {
        match self {
            UserIdStr(id) => id.parse::<u64>(),
            UserIdU64(id) => Ok(id.to_owned()),
        }
    }
}

impl From<&'static str> for UserId {
    fn from(id: &'static str) -> Self {
        UserIdStr(id.to_string())
    }
}

impl From<u64> for UserId {
    fn from(id: u64) -> Self {
        UserIdU64(id)
    }
}

impl FromStr for UserId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(UserIdStr(s.to_string()))
    }
}

fn eq_zero(i: &usize) -> bool {
    *i == 0
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthClaim<T> {
    exp: usize, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    aud: String, // Optional. Audience
    #[serde(skip_serializing_if = "eq_zero")]
    #[serde(default)]
    iat: usize, // Optional. Issued at (as UTC timestamp)
    #[serde(skip_serializing_if = "eq_zero")]
    #[serde(default)]
    nbf: usize, // Optional. Not Before (as UTC timestamp)
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    iss: String, // Optional. Issuer
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    sub: String, // Optional. Subject (whom token refers to)
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    jti: String, // Optional. JWT ID (a unique identifier for the JWT.) 只使用一次的 jwt 使用，预防重放攻击
    #[serde(skip_serializing_if = "Option::is_none")]
    payload: Option<T>, // Optional. other payloads in JWT
}

impl<T> AuthClaim<T> {
    pub fn new(exp: usize) -> Self {
        AuthClaim {
            exp,
            aud: "".to_string(),
            iat: 0,
            nbf: 0,
            iss: "".to_string(),
            sub: "".to_string(),
            jti: "".to_string(),
            payload: None,
        }
    }

    pub fn new_with_payload(exp: usize, payload: T) -> Self {
        AuthClaim {
            exp,
            aud: "".to_string(),
            iat: 0,
            nbf: 0,
            iss: "".to_string(),
            sub: "".to_string(),
            jti: "".to_string(),
            payload: Some(payload),
        }
    }

    pub fn issuer(&mut self, iss: &str) -> &mut Self {
        self.iss = iss.to_string();
        self
    }

    pub fn subject(&mut self, sub: &str) -> &mut Self {
        self.sub = sub.to_string();
        self
    }

    pub fn audience(&mut self, aud: &str) -> &mut Self {
        self.aud = aud.to_string();
        self
    }

    pub fn not_before(&mut self, nbf: usize) -> &mut Self {
        self.nbf = nbf;
        self
    }

    pub fn issue_at(&mut self, iat: usize) -> &mut Self {
        self.iat = iat;
        self
    }

    pub fn payload(&mut self, payload: T) -> &mut Self {
        self.payload = Some(payload);
        self
    }

    pub fn jti(&mut self, jit: &str) -> &mut Self {
        self.jti = jit.to_string();
        self
    }

    pub fn uuid_jti(&mut self) -> &mut Self {
        self.jti = Uuid::new_v4().to_string();
        self
    }
}

mod test {
    use super::Resp;
    use crate::model::UserId::UserIdStr;
    use crate::model::{AuthClaim, UserId};
    use chrono::{Days, Utc};
    use serde::{Deserialize, Serialize};
    use serde_json::{from_str, to_string};
    use std::ops::Add;

    #[derive(Debug, Serialize, Deserialize)]
    struct ClaimPayload {
        name: String,
        uid: UserId,
    }

    type MyClaim = AuthClaim<ClaimPayload>;

    #[test]
    fn claim_serialize_test() {
        let exp = Utc::now().add(Days::new(2)).timestamp() as usize;
        let mut claim = MyClaim::new_with_payload(
            exp,
            ClaimPayload {
                name: "iGxnon".to_string(),
                uid: UserIdStr("114514".to_string()),
            },
        );
        claim
            .issue_at(Utc::now().timestamp() as usize)
            .issuer("douban.com")
            .audience("1234")
            .subject("test");
        if let Err(_) = to_string(&claim) {
            panic!("cannot serialize MyClaim")
        }
    }

    #[test]
    fn claim_deserialize_test() {
        let payload1 = r#"{"exp":1669790928,"aud":"1234","iat":1669618128,"iss":"douban.com","sub":"test","payload":{"name":"iGxnon", "uid": 114514}}"#;
        let payload2 = r#"{"exp":1669790928,"aud":"1234","iat":1669618128,"iss":"douban.com"}"#;
        let payload3 = r#"{"exp":1669790928,"aud":"1234","sub":"test","payload":{"name":"iGxnon", "uid": 114514}}"#;
        if let Err(_) = from_str::<MyClaim>(payload1) {
            panic!("unable to deserialize {}", payload1);
        }
        if let Err(_) = from_str::<MyClaim>(payload2) {
            panic!("unable to deserialize {}", payload2);
        }
        if let Err(_) = from_str::<MyClaim>(payload3) {
            panic!("unable to deserialize {}", payload3);
        }
    }

    #[test]
    fn resp_serialize_test() {
        assert_eq!(
            r#"{"code":100,"msg":"ok","data":"success"}"#,
            to_string(&Resp {
                code: 100,
                msg: "ok".to_string(),
                data: Some("success".to_string())
            })
            .unwrap()
        );
        assert_eq!(
            r#"{"code":0,"msg":""}"#,
            to_string(&Resp::default()).unwrap()
        );
    }

    #[test]
    fn resp_deserialize_test() {
        let resp_str1 = r#"{"code":100,"msg":"ok","data":"success"}"#;
        let resp_str2 = r#"{"code":0,"msg":"","data":null,"some_other_key": "value"}"#;
        let resp1 = from_str::<Resp<serde_json::Value>>(resp_str1);
        let resp2 = from_str::<Resp<serde_json::Value>>(resp_str2);
        if let Err(_) = resp1 {
            panic!("test failed! cannot parse {}", resp_str1);
        }
        if let Err(_) = resp2 {
            panic!("test failed! cannot parse {}", resp_str2)
        }
        if resp1.unwrap().data.unwrap().as_str().unwrap() != "success" {
            panic!("test failed! parse {} data field failed!", resp_str1);
        }
    }
}
