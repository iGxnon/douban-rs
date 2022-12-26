pub mod auth_token;
pub mod parse_token;

#[cfg(test)]
mod test {
    use crate::domain::token::query::auth_token::AuthToken;
    use crate::domain::token::query::parse_token::ParseToken;
    use crate::domain::token::{Config, Resolver};
    use crate::pb;
    use crate::pb::TokenKind;
    use common::infra::*;
    use jsonwebtoken::jwk::JwkSet;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test() {
        let token = pb::Token {
            value: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjEwMDAwMDAwMDAwMCwiYXVkIjoic2VydmljZTEiLCJpc3MiOiJhdXRoIiwic3ViIjoiMTE0NTE0IiwicGF5bG9hZCI6eyJhY2Nlc3MiOnRydWUsInNlY3VyZSI6dHJ1ZX19.R36tiAl4P1CWysXyg2w0qxr9YogUCwacZJ4J0o8nsB4".to_string(),
            kind: TokenKind::Private.into(),
        };
        let resolver = Resolver::new(Config {
            hmac_key: "abc123".to_string(),
            clients: HashMap::from([(
                "service1".into(),
                "12345678901234567890123456789012".into(),
            )]),
            exp_delta: Default::default(),
            refresh_delta_rate: 3,
            redis_dsn: "".to_string(),
            pub_jwks: JwkSet { keys: vec![] },
            encode_pem: "".to_string(),
        });
        let auth_token = resolver.create_auth_token();
        let resp = auth_token
            .execute(AuthToken {
                token,
                secret: Some("12345678901234567890123456789012".into()),
            })
            .await;
        assert!(resp.is_ok());
        let token = pb::Token {
            value: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjEwMDAwMDAwMDAwMCwiYXVkIjoic2VydmljZTEiLCJpc3MiOiJhdXRoIiwic3ViIjoiMTE0NTE0IiwicGF5bG9hZCI6eyJhY2Nlc3MiOnRydWUsInNlY3VyZSI6dHJ1ZX19.R36tiAl4P1CWysXyg2w0qxr9YogUCwacZJ4J0o8nsB4".to_string(),
            kind: TokenKind::Private.into(),
        };
        let parse_token = resolver.create_parse_token();
        let resp = parse_token
            .execute(ParseToken {
                token,
                secret: Some("12345678901234567890123456789012".into()),
            })
            .await;
        assert!(resp.is_ok());
    }
}
