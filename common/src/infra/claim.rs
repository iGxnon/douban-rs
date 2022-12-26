use serde::{Deserialize, Serialize};
use uuid::Uuid;

fn eq_zero(i: &usize) -> bool {
    *i == 0
}

pub struct Builder<T> {
    inner: AuthClaim<T>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

impl<T> AuthClaim<T>
where
    T: Clone,
{
    pub fn builder(exp: usize) -> Builder<T> {
        Builder {
            inner: AuthClaim::new(exp),
        }
    }

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

    pub fn as_payload(&self) -> Option<&T> {
        self.payload.as_ref()
    }

    pub fn into_payload(self) -> Option<T> {
        self.payload
    }

    pub fn as_aud(&self) -> &str {
        &self.aud
    }

    pub fn as_iss(&self) -> &str {
        &self.iss
    }

    pub fn as_sub(&self) -> &str {
        &self.sub
    }

    pub fn as_jti(&self) -> &str {
        &self.jti
    }

    pub fn as_exp(&self) -> usize {
        self.exp
    }

    pub fn as_iat(&self) -> usize {
        self.iat
    }

    pub fn as_nbf(&self) -> usize {
        self.nbf
    }
}

impl<T> Builder<T> {
    pub fn issuer(mut self, iss: &str) -> Self {
        self.inner.iss = iss.to_string();
        self
    }

    pub fn subject(mut self, sub: &str) -> Self {
        self.inner.sub = sub.to_string();
        self
    }

    pub fn audience(mut self, aud: &str) -> Self {
        self.inner.aud = aud.to_string();
        self
    }

    pub fn not_before(mut self, nbf: usize) -> Self {
        self.inner.nbf = nbf;
        self
    }

    pub fn issue_at(mut self, iat: usize) -> Self {
        self.inner.iat = iat;
        self
    }

    pub fn jti(mut self, jit: &str) -> Self {
        self.inner.jti = jit.to_string();
        self
    }

    pub fn uuid_jti(mut self) -> Self {
        self.inner.jti = Uuid::new_v4().to_string();
        self
    }

    pub fn payload(mut self, payload: T) -> AuthClaim<T> {
        self.inner.payload = Some(payload);
        self.inner
    }
}
