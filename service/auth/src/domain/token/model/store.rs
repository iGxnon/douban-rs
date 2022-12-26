use crate::domain::token::error::Error;
use crate::domain::token::model::TokenId;
use crate::domain::token::RedisStore;
use crate::pb;
use std::fmt::Formatter;

use async_trait::async_trait;
use redis::AsyncCommands;
use serde::de::{Deserialize, Deserializer, MapAccess, Visitor};
use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::time::Duration;

#[derive(Debug)]
pub(in crate::domain) struct StoredToken {
    pub access: pb::Token,
    pub refresh: pb::Token,
}

impl Serialize for StoredToken {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("StoredToken", 2)?;
        s.serialize_field("access_value", &self.access.value)?;
        s.serialize_field("access_kind", &self.access.kind)?;
        s.serialize_field("refresh_value", &self.refresh.value)?;
        s.serialize_field("refresh_kind", &self.refresh.kind)?;
        s.end()
    }
}

impl<'de> Deserialize<'de> for StoredToken {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StoredTokenVisitor;
        impl<'de> Visitor<'de> for StoredTokenVisitor {
            type Value = StoredToken;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                write!(formatter, "a stored token value")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                map.next_key::<String>()?;
                let access_value = map.next_value::<String>()?;
                map.next_key::<String>()?;
                let access_kind = map.next_value::<i32>()?;
                map.next_key::<String>()?;
                let refresh_value = map.next_value::<String>()?;
                map.next_key::<String>()?;
                let refresh_kind = map.next_value::<i32>()?;
                Ok(StoredToken {
                    access: pb::Token {
                        value: access_value,
                        kind: access_kind,
                    },
                    refresh: pb::Token {
                        value: refresh_value,
                        kind: refresh_kind,
                    },
                })
            }
        }
        let token = deserializer.deserialize_struct(
            "StoredToken",
            &[
                "access_value",
                "access_kind",
                "refresh_value",
                "refresh_kind",
            ],
            StoredTokenVisitor,
        )?;
        Ok(token)
    }
}

#[async_trait]
pub(in crate::domain) trait TokenStore {
    async fn get_token(&self, id: TokenId, encrypt: bool) -> Result<Option<StoredToken>, Error>;
    async fn set_token(
        &self,
        id: TokenId,
        encrypt: bool,
        token: &StoredToken,
        ttl: Duration,
    ) -> Result<(), Error>;
}

#[async_trait]
impl TokenStore for RedisStore {
    async fn get_token(&self, id: TokenId, encrypt: bool) -> Result<Option<StoredToken>, Error> {
        let mut conn = self.get_async_connection().await?;
        let redis_key = if encrypt {
            format!("auth:token:{}:encrypt", id)
        } else {
            format!("auth:token:{}", id)
        };
        let token_str: Option<String> = conn.get(redis_key).await?;
        if let Some(token_str) = token_str {
            let result =
                serde_json::from_str::<StoredToken>(&token_str).map_err(Error::SerializerError)?;
            return Ok(Some(result));
        }
        Ok(None)
    }

    async fn set_token(
        &self,
        id: TokenId,
        encrypt: bool,
        token: &StoredToken,
        ttl: Duration,
    ) -> Result<(), Error> {
        let mut conn = self.get_async_connection().await?;
        let token_str = serde_json::to_string(token).map_err(Error::SerializerError)?;
        let redis_key = if encrypt {
            format!("auth:token:{}:encrypt", id)
        } else {
            format!("auth:token:{}", id)
        };
        conn.set_ex(redis_key, token_str, ttl.as_secs() as usize)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::domain::token::model::store::StoredToken;

    #[test]
    fn test() {
        let token = StoredToken {
            access: Default::default(),
            refresh: Default::default(),
        };
        let token_str = serde_json::to_string(&token).unwrap();
        let _ = serde_json::from_str::<StoredToken>(&token_str).unwrap();
    }
}
