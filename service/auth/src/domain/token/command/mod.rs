pub mod generate_token;

#[cfg(test)]
mod test {
    use crate::domain::token::command::generate_token::GenerateToken;
    use crate::domain::token::query::parse_token::ParseToken;
    use crate::domain::token::{Config, Resolver};
    use crate::pb::TokenKind;
    use common::infra::*;
    use jsonwebtoken::jwk::JwkSet;
    use serde_json::json;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test() {
        let jwks_json = json!({
            "keys": [
                {
                    "kty": "RSA",
                    "e": "AQAB",
                    "use": "sig",
                    "kid": "key1",
                    "alg": "RS256",
                    "n": "iuGt0KucfCBxSqovShdGyQ_1AdfWep3UBgFkemWCvEc40nWQ5WF50UBjvXeNDpTQSTSsjz6trP5u8BXJJdR3N7jZw845uOtk6BKUMJBrIrwKEZoflAxQXbvjEc6sfTZbY6mdcWLMkgweKfFIv6WJZkkcdggFKuiEaVeYbU53clejQbAQLA9_t39JLpdgPXH5bqqZb8g2spfa1R1rUavgwdhuV9idde1_Cv8H4VCYZ7c5qLqSaUxE6IfDVHHIu1Nwv79QLEIOdszmDY4tjNlrtueDuCA7OXQC_kHIpnz4tOIgOv9XudFVXKlhLva9wsTjt5Sai8HeKR5dnxg5EruJbQ",
                },
                {
                    "kty":"EC",
                    "crv":"P-256",
                    "x":"MKBCTNIcKUSDii11ySs3526iDZ8AiTo7Tu6KPAqv7D4",
                    "y":"4Etl6SRW2YiLUrN5vfvVHuhp7x8PxltmWWlbbM4IFyM",
                    "use":"enc",
                    "kid":"key2"
                }
            ]
        });
        let jwk_set: JwkSet = serde_json::from_value(jwks_json).unwrap();
        let resolver = Resolver::new(Config {
            hmac_key: "abc123".to_string(),
            clients: HashMap::from([(
                "service1".into(),
                "7pulg4JeihOYo1U9cmpto4Rq9YCah5tz".into(),
            )]),
            exp_delta: HashMap::from([(
                "service1".into(),
                chrono::Duration::days(2).num_seconds(),
            )]),
            refresh_delta_rate: 3,
            redis_dsn: "redis://127.0.0.1:6379/0".to_string(),
            pub_jwks: jwk_set,
            encode_pem: r#"
-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQCK4a3Qq5x8IHFK
qi9KF0bJD/UB19Z6ndQGAWR6ZYK8RzjSdZDlYXnRQGO9d40OlNBJNKyPPq2s/m7w
Fckl1Hc3uNnDzjm462ToEpQwkGsivAoRmh+UDFBdu+MRzqx9NltjqZ1xYsySDB4p
8Ui/pYlmSRx2CAUq6IRpV5htTndyV6NBsBAsD3+3f0kul2A9cfluqplvyDayl9rV
HWtRq+DB2G5X2J117X8K/wfhUJhntzmoupJpTEToh8NUcci7U3C/v1AsQg52zOYN
ji2M2Wu254O4IDs5dAL+QcimfPi04iA6/1e50VVcqWEu9r3CxOO3lJqLwd4pHl2f
GDkSu4ltAgMBAAECggEAbFLcTM8dzh9L3k3hdquzFW4xzs83xgnGbyy031a/4vS2
WElEy/T8m/7aDNTrm7zsvLyt/0iHFFCb3P1RGAWhO0Ad8kCu+xH3cZ/UIBD0z3HV
dKc/DC2SnZnH4YLPPRahr5mDaQYDw8JZ4KMG+Bw4kCRkY5eb2Dzl0nh1NoSmW/Lh
AAszE2FcE0WfdbI9JtmOFCtDcOVIGnV8g079y0CiG/QO3/E0Q7u5UBzgwGgtcD6B
cuLCn0Acpm5UT9l/kKhxglIuo8pnxB/Q31X/g3cr5IPUarLX7WvCMyZi+GV6j1I+
KQPhMt41fUbth3J6fr8/19p32EzEl1C5RjLp+mcIIQKBgQDUGr4zPp5ZQJpXA4HR
pzgbBH7tKw6/rjR6niaBEyiwI6ugoE0wN8U9PI9VBdi8mhGHSzHD6Kxfw1H3iVF6
9gX8zZgJrfzy4qQbrU9gle4eKrwjyOVHTjBVx8zbj48iTfKKq1fFclAfLb40OMR9
Ic+j9iFk9FGHwzrBg5fdCJ03aQKBgQCnn5lwZg9zGHKd8faOt3JFRdmaEXKevOJj
ClHSZjmGED589L+a88RHndtLM+1MvUGGoKRjifsolkN1ogU/c6PfRq1Ozo4MVh2o
UerwwVbAFGv0bC0xP0HQxAxgRHISdJgQn9XqW72RVWdpunRUcdpY9LyTJPXN7SWU
K4GkBk6lZQKBgQDHhKCF89Fgg7SrRVFItdPBFmmPD4HALU6QSVRO9oa/qc44OpJh
WRTglab+g5FtWEBE1Cbr8mKzcjgYccODtwnK1FrAQDpA/5D/t/eDE4X6Opjf0Ipc
mOA+0MOThWdPDaOpbaQSx4U2zwCsfvnV+4gm49Bl9qz62Frczbx83y2EgQKBgAS7
Up+DoggDtqiSvf0FXKpr8FqTB4NHnbRiBDFGRXVtW/Y3CnmbS/0hjaEv1BEIfqMI
Qdu1d2uL4ledvTwvTX7uBdJlrkjW3Xt05IbrTkGZ0fpSW2w6ducnwZmuFPJEbE57
1JQLBuzlVkf7xXDkzd8Y+YHF11J60Ua/e6dfrjSpAoGASfShV59FiFXPrcjTf+EM
VHaB/PehQgm/RYjLtVKzEZu/NGB57nKzuPNGB7DnX6vM+E70c3lqz+qlw3R6YtK0
69CyRTQoPOnvgYkd1k70frKF2lRfehGwmJBgQTSIzUASdrtJipOM02v1x8faV+DL
c18+HlaE4v5HeLpjEM0FHy0=
-----END PRIVATE KEY-----
        "#
            .to_string(),
        });
        let generate_token = resolver.create_generate_token();
        let resp = generate_token
            .execute(GenerateToken {
                id: "114514".to_string(),
                sid: "service1".to_string(),
                kind: TokenKind::Private,
                secret: None,
            })
            .await
            .unwrap();
        let parse_token = resolver.create_parse_token();
        let resp = parse_token
            .execute(ParseToken {
                token: resp.token.get(0).unwrap().clone(),
                secret: None,
            })
            .await;
        println!("{:?}", resp);
    }
}
