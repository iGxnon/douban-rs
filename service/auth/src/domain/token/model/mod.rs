use crate::domain::token::error::Error;
use crate::pb;
use crate::pb::TokenKind;
use common::infra::*;
use jsonwebtoken::jwk::*;
use jsonwebtoken::*;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub mod store;

pub type TokenId = Id<Token>;
pub type Claim = AuthClaim<Payload>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Token {
    pub id: TokenId,
    pub claim: TokenClaim,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "claim")]
pub enum TokenClaim {
    Access(Claim),
    Refresh(Claim),
}

impl TokenClaim {
    pub fn into_inner(self) -> Claim {
        match self {
            TokenClaim::Access(c) => c,
            TokenClaim::Refresh(c) => c,
        }
    }

    pub fn inner(&self) -> &Claim {
        match self {
            TokenClaim::Access(c) => c,
            TokenClaim::Refresh(c) => c,
        }
    }

    pub fn inner_mut(&mut self) -> &mut Claim {
        match self {
            TokenClaim::Access(c) => c,
            TokenClaim::Refresh(c) => c,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, Default)]
pub struct Payload {
    pub access: bool, // if it is access token
    pub secure: bool, // if it is secure token (auth secret in request)
}

impl FromStr for Token {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let claim: Claim = serde_json::from_str(s)?;
        let Payload { access, .. } = claim.as_payload().ok_or(Error::NoPayload)?;

        let res = match access {
            true => Token {
                id: TokenId::from(claim.as_sub()),
                claim: TokenClaim::Access(claim),
            },
            false => Token {
                id: TokenId::from(claim.as_sub()),
                claim: TokenClaim::Refresh(claim),
            },
        };
        Ok(res)
    }
}

impl TryFrom<String> for Token {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl TryFrom<Claim> for Token {
    type Error = Error;

    fn try_from(claim: Claim) -> Result<Self, Self::Error> {
        let payload = claim.as_payload().ok_or(Error::NoPayload)?;
        let res = match payload.access {
            true => Token {
                id: TokenId::from(claim.as_sub()),
                claim: TokenClaim::Access(claim),
            },
            false => Token {
                id: TokenId::from(claim.as_sub()),
                claim: TokenClaim::Refresh(claim),
            },
        };
        Ok(res)
    }
}

const URL_SAFE_ENGINE: base64::engine::fast_portable::FastPortable =
    base64::engine::fast_portable::FastPortable::from(
        &base64::alphabet::URL_SAFE,
        base64::engine::fast_portable::NO_PAD,
    );

fn decrypt(token_raw: impl AsRef<[u8]>, secret: Option<impl AsRef<[u8]>>) -> Result<String, Error> {
    let mut base64_decode =
        base64::decode_engine(token_raw, &URL_SAFE_ENGINE).map_err(|e| Error::Other(e.into()))?;
    let secret_key = secret.ok_or_else(|| Error::InvalidSecret("no secret found".into()))?;
    let key = bincode_aes::create_key(secret_key.as_ref().to_vec())
        .map_err(|_| Error::InvalidSecret("invalid secret length".into()))?;
    let cryptor = bincode_aes::with_key(key);
    let result: String = cryptor
        .deserialize(&mut base64_decode)
        .map_err(|_| Error::InvalidSecret("decrypt token failed".into()))?;
    Ok(result)
}

fn encrypt(token: impl Into<Vec<u8>>, secret: impl AsRef<[u8]>) -> Result<String, Error> {
    let key = bincode_aes::create_key(secret.as_ref().to_vec())
        .map_err(|_| Error::InvalidSecret("invalid secret length".into()))?;
    let cryptor = bincode_aes::with_key(key);
    let result = cryptor
        .serialize(&token.into())
        .map_err(|_| Error::EncryptFailed)?;
    Ok(base64::encode_engine(result, &URL_SAFE_ENGINE))
}

impl Token {
    pub fn from_pri_token(
        token: &pb::Token,
        decode_key: &DecodingKey,
        secret: Option<impl AsRef<[u8]>>,
    ) -> Result<Self, Error> {
        if token.kind() == TokenKind::Public {
            return Self::from_pub_token(token);
        }
        let jwt = &token.value;
        if !jwt.contains('.') {
            let jwt_decrypted = decrypt(jwt, secret)?;
            return decode::<Claim>(&jwt_decrypted, decode_key, &Validation::default())?
                .claims
                .try_into();
        }
        decode::<Claim>(jwt, decode_key, &Validation::default())?
            .claims
            .try_into()
    }

    pub fn from_pub_token(token: &pb::Token) -> Result<Self, Error> {
        // if token.kind() != TokenKind::Public {
        //     return Err(Error::InvalidToken(
        //         "only public token could be tried from".into(),
        //     ));
        // }
        debug_assert!(
            token.kind() == TokenKind::Public,
            "cannot parse a private token in `from_pub_token`"
        );
        let header = decode_header(&token.value)?;
        let jwk = header
            .jwk
            .ok_or_else(|| Error::InvalidToken("no jwk found in jwt header".to_string()))?;
        if let AlgorithmParameters::RSA(parm) = jwk.algorithm {
            let pub_key = DecodingKey::from_rsa_components(&parm.n, &parm.e)?;
            return decode::<Claim>(&token.value, &pub_key, &Validation::new(Algorithm::RS256))?
                .claims
                .try_into();
        }
        Err(Error::InvalidToken("only support rsa".into()))
    }

    pub fn private_token(
        &mut self,
        encode_key: &EncodingKey,
        secret: Option<impl AsRef<[u8]>>,
    ) -> Result<pb::Token, Error> {
        if secret.is_some() {
            let mut payload = *self.claim.inner().as_payload().ok_or(Error::NoPayload)?;
            payload.secure = true;
            self.claim.inner_mut().payload(payload);
        }
        let mut value = encode(
            &Header::new(Algorithm::HS256),
            &self.claim.inner(),
            encode_key,
        )?;
        if let Some(key) = secret {
            value = encrypt(value, key)?;
        }
        Ok(pb::Token {
            value,
            kind: TokenKind::Private as i32,
        })
    }

    pub fn public_token(
        &self,
        pub_key_set: &JwkSet,
        encode_key: &EncodingKey,
    ) -> Result<pb::Token, Error> {
        let mut header = Header::default();
        let kid = self.id.to_string();
        let jwk = pub_key_set.find(&kid).ok_or_else(|| {
            Error::InvalidKid(format!(
                "cannot find jwk with `Token.id({})` in jwk_set",
                kid
            ))
        })?;
        header.kid = Some(kid);
        header.jwk = Some(jwk.clone());
        header.alg = jwk.common.algorithm.ok_or_else(|| {
            Error::InvalidKid(
                "no alg found in jwk common fields, except a declared algorithm like `RS256`"
                    .to_string(),
            )
        })?;
        let value = encode(&header, &self.claim.inner(), encode_key)?;
        Ok(pb::Token {
            value,
            kind: TokenKind::Public as i32,
        })
    }
}

mod test {
    use jsonwebtoken::jwk::JwkSet;
    use jsonwebtoken::*;
    use serde_json::json;

    use super::*;

    #[test]
    fn cipher_test() {
        let key = "12345678901234567890123456789012";
        let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        let encrypted = encrypt(token, key).unwrap();
        let decrypted = decrypt(encrypted, Some(key)).unwrap();
        assert_eq!(decrypted, token);
    }

    #[test]
    fn ser_test() {
        let token = Token {
            id: Id::new_str(),
            claim: TokenClaim::Refresh(AuthClaim::new(120)),
        };
        let token_str = serde_json::to_string(&token).unwrap();
        assert!(matches!(
            serde_json::from_str::<Token>(&token_str).unwrap().claim,
            TokenClaim::Refresh(_)
        ));
    }

    #[test]
    fn from_test() {
        let token: Result<Token, Error> = "{\"exp\": 10}".parse();
        assert!(token.is_err());
        let token: Result<Token, Error> = r#"{
            "exp": 10,
            "sub": "114514",
            "payload": {
                "access": true,
                "secure": false
            }
        }"#
        .parse();
        assert!(token.is_ok());
        assert_eq!(token.unwrap().id.as_u64().unwrap(), 114514)
    }

    #[test]
    fn jwt_priv_test() {
        let token: Result<Token, Error> = r#"{
            "exp": 10000000000000000,
            "sub": "114514",
            "payload": {
                "access": true,
                "secure": false
            }
        }"#
        .parse();
        let jwt = token
            .unwrap()
            .private_token(
                &EncodingKey::from_secret("secret".as_ref()),
                Some("12345678901234567890123456789012"),
            )
            .unwrap();
        let token = Token::from_pri_token(
            &jwt,
            &DecodingKey::from_secret("secret".as_ref()),
            Some("12345678901234567890123456789012"),
        )
        .unwrap();
        assert!(token.claim.inner().as_payload().unwrap().secure)
    }

    #[test]
    fn jwt_pub_sign_test() {
        let token = Token {
            id: Id::from("abc123"),
            claim: TokenClaim::Access(Claim::builder(0).subject("abc123").payload(Payload {
                access: true,
                secure: false,
            })),
        };
        let jwks_json = json!({
            "keys": [
                {
                    "kty": "RSA",
                    "e": "AQAB",
                    "use": "sig",
                    "kid": "abc123",
                    "alg": "RS256",
                    "n": "iuGt0KucfCBxSqovShdGyQ_1AdfWep3UBgFkemWCvEc40nWQ5WF50UBjvXeNDpTQSTSsjz6trP5u8BXJJdR3N7jZw845uOtk6BKUMJBrIrwKEZoflAxQXbvjEc6sfTZbY6mdcWLMkgweKfFIv6WJZkkcdggFKuiEaVeYbU53clejQbAQLA9_t39JLpdgPXH5bqqZb8g2spfa1R1rUavgwdhuV9idde1_Cv8H4VCYZ7c5qLqSaUxE6IfDVHHIu1Nwv79QLEIOdszmDY4tjNlrtueDuCA7OXQC_kHIpnz4tOIgOv9XudFVXKlhLva9wsTjt5Sai8HeKR5dnxg5EruJbQ",
                },
                {
                    "kty":"EC",
                    "crv":"P-256",
                    "x":"MKBCTNIcKUSDii11ySs3526iDZ8AiTo7Tu6KPAqv7D4",
                    "y":"4Etl6SRW2YiLUrN5vfvVHuhp7x8PxltmWWlbbM4IFyM",
                    "use":"enc",
                    "kid":"1"
                }
            ]
        });
        let priv_key_pem = r#"
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
        "#;
        let jwk_set: JwkSet = serde_json::from_value(jwks_json).unwrap();
        let token = token.public_token(
            &jwk_set,
            &EncodingKey::from_rsa_pem(priv_key_pem.as_ref()).unwrap(),
        );
        assert!(token.is_ok())
    }

    #[test]
    fn jwt_pub_valid_test() {
        let token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiIsImp3ayI6eyJ1c2UiOiJzaWciLCJhbGciOiJSUzI1NiIsImtpZCI6ImFiYzEyMyIsImt0eSI6IlJTQSIsIm4iOiJpdUd0MEt1Y2ZDQnhTcW92U2hkR3lRXzFBZGZXZXAzVUJnRmtlbVdDdkVjNDBuV1E1V0Y1MFVCanZYZU5EcFRRU1RTc2p6NnRyUDV1OEJYSkpkUjNON2padzg0NXVPdGs2QktVTUpCcklyd0tFWm9mbEF4UVhidmpFYzZzZlRaYlk2bWRjV0xNa2d3ZUtmRkl2NldKWmtrY2RnZ0ZLdWlFYVZlWWJVNTNjbGVqUWJBUUxBOV90MzlKTHBkZ1BYSDVicXFaYjhnMnNwZmExUjFyVWF2Z3dkaHVWOWlkZGUxX0N2OEg0VkNZWjdjNXFMcVNhVXhFNklmRFZISEl1MU53djc5UUxFSU9kc3ptRFk0dGpObHJ0dWVEdUNBN09YUUNfa0hJcG56NHRPSWdPdjlYdWRGVlhLbGhMdmE5d3NUanQ1U2FpOEhlS1I1ZG54ZzVFcnVKYlEiLCJlIjoiQVFBQiJ9LCJraWQiOiJhYmMxMjMifQ.eyJleHAiOjEwMDAwMDAwMDAwMDAwMDAwMCwic3ViIjoiYWJjMTIzIiwicGF5bG9hZCI6eyJhY2Nlc3MiOnRydWUsInZlcnNpb24iOjEsInNlY3VyZSI6ZmFsc2V9fQ.Ys6rzEbMp8AAe2HMoMszAbwkEIKOrBtrr6oVcdWF3oJpNLHnO-CW073gLbAlMJZgZd9FeQrWlDQV1KxAZTywOnLTfugsfj0WAAArFGBWKC0jTHTPSLu-sGPXLaHcnxkhLWLyPw-YaYVIB2zdHMqpeh6m9bU6I3YSW1HRogQFh7qzoG9Tc1TFqbNeJQk3HlJbbhb4ueDX7F6r9I11ZzwsSzAltUON-JC-JWb94FJsRYE92X6P_v3ZCZIKYAYps5orM4--orlDtkYsnFvd2_hAIwUuiAC7z1aA-nAYbkV8u7kF9_U5lLpKmg-N4HU54_CiZjIzj5snZyuFHWhT8EXJHQ";
        let header = decode_header(token).unwrap();

        if let AlgorithmParameters::RSA(parm) = header.jwk.unwrap().algorithm {
            let pub_key = DecodingKey::from_rsa_components(&parm.n, &parm.e).unwrap();
            let mut validation = Validation::new(Algorithm::RS256);
            validation.required_spec_claims.remove("exp");
            let result = decode::<Claim>(token, &pub_key, &validation);
            assert!(result.is_ok())
        }
    }

    #[test]
    fn try_from() {
        let token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiIsImp3ayI6eyJ1c2UiOiJzaWciLCJhbGciOiJSUzI1NiIsImtpZCI6ImFiYzEyMyIsImt0eSI6IlJTQSIsIm4iOiJpdUd0MEt1Y2ZDQnhTcW92U2hkR3lRXzFBZGZXZXAzVUJnRmtlbVdDdkVjNDBuV1E1V0Y1MFVCanZYZU5EcFRRU1RTc2p6NnRyUDV1OEJYSkpkUjNON2padzg0NXVPdGs2QktVTUpCcklyd0tFWm9mbEF4UVhidmpFYzZzZlRaYlk2bWRjV0xNa2d3ZUtmRkl2NldKWmtrY2RnZ0ZLdWlFYVZlWWJVNTNjbGVqUWJBUUxBOV90MzlKTHBkZ1BYSDVicXFaYjhnMnNwZmExUjFyVWF2Z3dkaHVWOWlkZGUxX0N2OEg0VkNZWjdjNXFMcVNhVXhFNklmRFZISEl1MU53djc5UUxFSU9kc3ptRFk0dGpObHJ0dWVEdUNBN09YUUNfa0hJcG56NHRPSWdPdjlYdWRGVlhLbGhMdmE5d3NUanQ1U2FpOEhlS1I1ZG54ZzVFcnVKYlEiLCJlIjoiQVFBQiJ9LCJraWQiOiJhYmMxMjMifQ.eyJleHAiOjAsInN1YiI6ImFiYzEyMyIsInBheWxvYWQiOnsiYWNjZXNzIjp0cnVlLCJ2ZXJzaW9uIjoxLCJzZWN1cmUiOmZhbHNlfX0.e64fUHM3LRXZiGWCjX4zQkS_utlsXatTii3RK1-U3a8RDVr_0b5buuaTPFeRcesReiH6xJ6c-8IhE3gPL9wl2BLbrkV7VkZf6PKv0ApptaWuIXP-xNFT5Fnz4orVIt8mjZLDylF17bCgXruMPqxEocQn5TyztzjvPHMIYh0FZzCLDTzwWTnAPvMtvo7eIFy9cG-j8PM_ixyjqdwP3AOSXvNQB2qJePcL2DN5kjiTaqipo0EG6EEszNPEFhfJipCmRwPIIgIEG9gOUtyNXDja15fXWXWX0Xo8SntVVsw6Mori7NlDqQETJX19acVQslQdev8gyivRJjm1SJ6qpazGcw";
        let token = pb::Token {
            value: token.to_string(),
            kind: 1,
        };
        let result = Token::from_pub_token(&token);
        assert!(matches!(result, Err(Error::JwtError(_))))
    }
}
