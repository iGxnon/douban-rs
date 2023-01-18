use super::Token;
use crate::domain::token::model::LEE_WAY;
use crate::domain::token::RedisStore;
use common::infra::{Command, GetFrom, Query, SetInto};
use common::*;
use redis::AsyncCommands;
use std::cmp::max;
use tonic::{async_trait, Status};

#[async_trait]
impl Query<GetFrom<Option<Self>, Status, RedisStore>> for Token {
    async fn execute(
        &self,
        input: GetFrom<Option<Self>, Status, RedisStore>,
    ) -> Result<Option<Self>, Status> {
        let store = input.src();
        let mut conn = store
            .0
            .get_async_connection()
            .await
            .map_err(|_| internal!("Get async connection from redis failed"))?;
        let key = format!("auth:token:{}:{}", self.id, self.claim.kind());
        let token_str: Option<String> = conn
            .get(&key)
            .await
            .map_err(|_| internal!(&format!("Failed to get key: {} value in redis", key)))?;
        if let Some(token_str) = token_str {
            let token = serde_json::from_str(token_str.as_str()).map_err(|_| {
                internal!(&format!(
                    "Failed to deserialize internal type {:?}",
                    token_str
                ))
            })?;
            return Ok(Some(token));
        }
        Ok(None)
    }
}

#[async_trait]
impl Command<SetInto<Status, RedisStore>> for Token {
    async fn execute(self, input: SetInto<Status, RedisStore>) -> Result<(), Status> {
        let store = input.dst();
        let mut conn = store
            .0
            .get_async_connection()
            .await
            .map_err(|_| internal!("Get async connection from redis failed"))?;
        let key = format!("auth:token:{}:{}", self.id, self.claim.kind());
        let token_str = serde_json::to_string(&self)
            .map_err(|_| internal!(&format!("Failed to serialize internal type {:?}", self)))?;
        let now = jsonwebtoken::get_current_timestamp();
        let exp = self.claim.inner().as_exp();
        let ex = max(exp - now, 2 * LEE_WAY);
        conn.set_ex(&key, &token_str, (ex - 2 * LEE_WAY) as usize)
            .await
            .map_err(|_| {
                internal!(&format!(
                    "Failed to set key: {} value: {} in redis",
                    key, token_str
                ))
            })?;
        Ok(())
    }
}
