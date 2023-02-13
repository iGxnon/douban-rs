use super::Token;
use crate::auth::domain::token::model::LEE_WAY;
use common::infra::*;
use common::internal;
use common::status::prelude::*;
use redis::AsyncCommands;
use std::cmp::max;
use tonic::async_trait;

#[async_trait]
impl Query<RedisGet<Option<Self>>> for Token {
    async fn execute(&self, input: RedisGet<Option<Self>>) -> GrpcResult<Option<Self>> {
        // TODO connection pool for redis
        let store = input.into_from();
        let mut conn = store
            .get_async_connection()
            .await
            .map_err(|_| internal!("Get async connection from redis failed"))?;
        let key = format!("auth:token:{}:{}", self.id, self.claim.kind());
        let token_str: Option<String> = conn
            .get(&key)
            .await
            .map_err(|_| internal!(format!("Failed to get key: {} value in redis", key)))?;
        if let Some(token_str) = token_str {
            let token = serde_json::from_str(token_str.as_str())
                .map_err(|_| internal!("Failed to deserialize internal type from redis"))?;
            return Ok(Some(token));
        }
        Ok(None)
    }
}

#[async_trait]
impl Command<RedisSet> for Token {
    async fn execute(self, input: RedisSet) -> GrpcResult<()> {
        let store = input.into_dst();
        let mut conn = store
            .get_async_connection()
            .await
            .map_err(|_| internal!("Get async connection from redis failed"))?;
        let key = format!("auth:token:{}:{}", self.id, self.claim.kind());
        let token_str = serde_json::to_string(&self)
            .map_err(|_| internal!(format!("Failed to serialize internal type {:?}", self)))?;
        let now = jsonwebtoken::get_current_timestamp();
        let exp = self.claim.inner().as_exp();
        let ex = max(exp - now, 2 * LEE_WAY) - LEE_WAY;
        conn.set_ex(&key, &token_str, ex as usize)
            .await
            .map_err(|_| internal!(format!("Failed to set key {}, ex {} in redis", key, ex)))?;
        Ok(())
    }
}

#[async_trait]
impl Command<RedisDel> for Token {
    async fn execute(self, input: RedisDel) -> GrpcResult<()> {
        let store = input.into_dst();
        let mut conn = store
            .get_async_connection()
            .await
            .map_err(|_| internal!("Get async connection from redis failed"))?;
        let key = format!("auth:token:{}:{}", self.id, self.claim.kind());
        conn.del(&key)
            .await
            .map_err(|_| internal!(format!("Failed to del key {}", key)))?;
        Ok(())
    }
}
