use crate::user::domain::user::model::oauth::{GithubId, NewOAuth, PutOAuth};
use crate::user::domain::user::{RoleGroup, UserResolver};
use base64::Engine;
use chrono::NaiveDateTime;
use common::infra::Resolver;
use common::status::prelude::*;
use common::utils::regex::{check_cn_phone, check_email};
use common::{already_exists, infra, internal, invalid_argument, not_found};
use diesel::prelude::*;
use diesel::result::Error;
use hmac::{Hmac, Mac};
use migration::t_users;
use proto::pb::auth::token::v1::token_service_client::TokenServiceClient;
use proto::pb::auth::token::v1::{GenerateTokenReq, Payload};
use proto::pb::user::sys::v1::LoginRes;
use std::fmt::Display;
use tonic::transport::Channel;
use tonic::Status;

pub type UserId = infra::Id<User>;

#[derive(Queryable)]
pub struct User {
    id: i64,
    oauth_id: Option<i64>,
    username: String,
    phone: Option<String>,
    email: Option<String>,
    nickname: String,
    hashed_password: String,
    role_group: String,
    creat_at: NaiveDateTime,
    update_at: NaiveDateTime,
    delete_at: Option<NaiveDateTime>,
    last_login: Option<NaiveDateTime>,
}

#[derive(AsChangeset)]
#[diesel(table_name = t_users)]
struct PutUser<'a> {
    phone: Option<&'a str>,
    email: Option<&'a str>,
    nickname: Option<&'a str>,
    hashed_password: Option<&'a str>,
    update_at: Option<NaiveDateTime>,
    last_login: Option<NaiveDateTime>,
}

impl<'a> Default for PutUser<'a> {
    fn default() -> Self {
        Self {
            phone: None,
            email: None,
            nickname: None,
            hashed_password: None,
            update_at: Some(NaiveDateTime::default()),
            last_login: None,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = t_users)]
struct NewUser<'a> {
    username: &'a str,
    nickname: &'a str,
    role_group: &'a str,
    hashed_password: String,
}

fn map_not_found<'a>(
    scope: &'a str,
    identifier: impl Display + 'a,
) -> impl Fn(Error) -> Status + 'a {
    move |e| {
        if e == diesel::NotFound {
            return not_found!(format!("{}({})", scope, identifier));
        }
        internal!(format!("Database connection error: {}", e))
    }
}

#[inline]
fn hash_password(secret: &str, password: &str) -> String {
    let mut mac = Hmac::<sha2::Sha256>::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(password.as_bytes());
    let output = Mac::finalize(mac).into_bytes();
    base64::prelude::BASE64_STANDARD.encode(output)
}

/// TODO cached
impl User {
    pub(in crate::user::domain) fn register(
        name: &str,
        password: &str,
        secret: &str,
        rg: RoleGroup,
        conn: &mut PgConnection,
    ) -> GrpcResult<User> {
        use migration::t_users::dsl::*;

        let new_user = NewUser {
            username: name,
            nickname: name,
            role_group: rg.name(),
            hashed_password: hash_password(secret, password),
        };
        let user: User = diesel::insert_into(t_users)
            .values(new_user)
            .get_result(conn)
            .map_err(|e| {
                if let Error::DatabaseError(_, ref info) = e {
                    if matches!(info.constraint_name(), Some("t_users_username_uindex")) {
                        return already_exists!(format!("user({})", name));
                    }
                };
                internal!(format!("Cannot create a new user, err: {}", e))
            })?;
        Ok(user)
    }

    pub(in crate::user::domain) fn query_id(
        uid: UserId,
        conn: &mut PgConnection,
    ) -> GrpcResult<User> {
        use migration::t_users::dsl::*;

        let uid = uid.as_i64().expect("Expect an i64 user id");
        let user: User = t_users
            .find(uid)
            .first(conn)
            .map_err(map_not_found("user", uid))?;
        Ok(user)
    }

    pub(in crate::user::domain) fn query_email(
        mail: &str,
        conn: &mut PgConnection,
    ) -> GrpcResult<User> {
        use migration::t_users::dsl::*;

        let user: User = t_users
            .filter(email.eq(mail))
            .first(conn)
            .map_err(map_not_found("user", mail))?;
        Ok(user)
    }

    pub(in crate::user::domain) fn query_phone(
        phone_num: &str,
        conn: &mut PgConnection,
    ) -> GrpcResult<User> {
        use migration::t_users::dsl::*;

        let user: User = t_users
            .filter(phone.eq(phone_num))
            .first(conn)
            .map_err(map_not_found("user", phone_num))?;
        Ok(user)
    }

    pub(in crate::user::domain) fn query_username(
        uname: &str,
        conn: &mut PgConnection,
    ) -> GrpcResult<User> {
        use migration::t_users::dsl::*;

        let user: User = t_users
            .filter(username.eq(uname))
            .first(conn)
            .map_err(map_not_found("user", uname))?;
        Ok(user)
    }

    pub(in crate::user::domain) fn query_identifier(
        identifier: &str,
        conn: &mut PgConnection,
    ) -> GrpcResult<User> {
        let uid = identifier.parse::<i64>();
        if let Ok(uid) = uid {
            return Self::query_id(UserId::from(uid as u64), conn);
        }
        if check_email(identifier) {
            return Self::query_email(identifier, conn);
        }
        if check_cn_phone(identifier) {
            return Self::query_phone(identifier, conn);
        }
        Self::query_username(identifier, conn)
    }

    pub(in crate::user::domain) fn login(
        identifier: &str,
        password: &str,
        secret: &str,
        conn: &mut PgConnection,
    ) -> GrpcResult<User> {
        let uid = identifier.parse::<i64>();
        if let Ok(uid) = uid {
            return Self::login_id(uid, password, secret, conn);
        }
        if check_email(identifier) {
            return Self::login_email(identifier, password, secret, conn);
        }
        if check_cn_phone(identifier) {
            return Self::login_phone(identifier, password, secret, conn);
        }
        Self::login_username(identifier, password, secret, conn)
    }

    #[inline]
    fn login_email(
        mail: &str,
        password: &str,
        secret: &str,
        conn: &mut PgConnection,
    ) -> GrpcResult<User> {
        use migration::t_users::dsl::*;

        let user: User = t_users
            .filter(email.eq(mail))
            .first(conn)
            .map_err(map_not_found("user", mail))?;
        user.check_password(password, secret)?;
        Ok(user)
    }

    #[inline]
    fn login_phone(
        phone_num: &str,
        password: &str,
        secret: &str,
        conn: &mut PgConnection,
    ) -> GrpcResult<User> {
        use migration::t_users::dsl::*;

        let user: User = t_users
            .filter(phone.eq(phone_num))
            .first(conn)
            .map_err(map_not_found("user", phone_num))?;
        user.check_password(password, secret)?;
        Ok(user)
    }

    #[inline]
    fn login_username(
        uname: &str,
        password: &str,
        secret: &str,
        conn: &mut PgConnection,
    ) -> GrpcResult<User> {
        use migration::t_users::dsl::*;

        let user: User = t_users
            .filter(username.eq(uname))
            .first(conn)
            .map_err(map_not_found("user", uname))?;
        user.check_password(password, secret)?;
        Ok(user)
    }

    #[inline]
    fn login_id(
        uid: i64,
        password: &str,
        secret: &str,
        conn: &mut PgConnection,
    ) -> GrpcResult<User> {
        use migration::t_users::dsl::*;

        let user: User = t_users
            .find(uid)
            .first(conn)
            .map_err(map_not_found("user", uid))?;
        user.check_password(password, secret)?;
        Ok(user)
    }

    pub(in crate::user::domain) fn check_password(
        &self,
        password: &str,
        secret: &str,
    ) -> GrpcResult<()> {
        let mut mac = Hmac::<sha2::Sha256>::new_from_slice(secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(password.as_bytes());
        let bytes = base64::prelude::BASE64_STANDARD
            .decode(&self.hashed_password)
            .map_err(|_| internal!("Failed base64 decode hashed password"))?;
        Mac::verify_slice(mac, bytes.as_slice())
            .map_err(|_| invalid_argument!("password", "correct password").into())
    }

    pub(in crate::user::domain) fn update_password(
        &mut self,
        new_password: &str,
        secret: &str,
        conn: &mut PgConnection,
    ) -> GrpcResult<()> {
        use migration::t_users::dsl::*;

        let hashed = hash_password(secret, new_password);
        let user = PutUser {
            hashed_password: Some(&hashed),
            ..Default::default()
        };

        diesel::update(t_users.find(self.id))
            .set(user)
            .execute(conn)
            .map_err(map_not_found("user", self.id))?;

        self.hashed_password = hashed;

        Ok(())
    }

    pub(in crate::user::domain) async fn sign_token_pair(
        &self,
        mut client: TokenServiceClient<Channel>,
    ) -> GrpcResult<LoginRes> {
        let res = client
            .generate_token(GenerateTokenReq {
                sub: self.id.to_string(), // use username if i64 id is overflow
                aud: UserResolver::DOMAIN.to_string(),
                jti: None,
                payload: Some(Payload {
                    sub: self.id.to_string(), // use username if i64 id is overflow
                    group: self.role_group.to_string(),
                    extra: "".to_string(),
                }),
            })
            .await?
            .into_inner();
        debug_assert!(res.access.is_some(), "GenerateTokenRes is empty!");
        debug_assert!(res.refresh.is_some(), "GenerateTokenRes is empty!");
        Ok(LoginRes {
            access: res.access,
            refresh: res.refresh,
        })
    }

    pub(in crate::user::domain) fn bind(
        &self,
        mail: Option<String>,
        phone_num: Option<String>,
        conn: &mut PgConnection,
    ) -> GrpcResult<()> {
        use migration::t_users::dsl::*;

        diesel::update(t_users.find(self.id))
            .set(PutUser {
                phone: phone_num.as_deref(),
                email: mail.as_deref(),
                ..Default::default()
            })
            .execute(conn)
            .map_err(map_not_found("user", self.id))?;
        Ok(())
    }

    pub(in crate::user::domain) fn bind_github(
        &mut self,
        github_id: GithubId,
        conn: &mut PgConnection,
    ) -> GrpcResult<()> {
        use migration::t_oauth::dsl::*;

        match self.oauth_id {
            None => {
                conn.transaction::<(), GrpcStatus, _>(|conn| {
                    let new = NewOAuth {
                        github: Some(github_id.as_i64().expect("Expect an i64 github id")),
                    };
                    let oid: i64 = diesel::insert_into(t_oauth)
                        .values(new)
                        .returning(id)
                        .get_result(conn)
                        .map_err(|e| internal!(format!("Cannot create a new oauth, err: {}", e)))?;
                    self.oauth_id = Some(oid);
                    diesel::update(t_users::table.find(self.id))
                        .set((t_users::oauth_id.eq(oid), t_users::update_at.eq(NaiveDateTime::default())))
                        .execute(conn)
                        .map_err(|e| internal!(format!("Cannot create a new oauth, err: {}", e)))?;
                    Ok(())
                })?;
            }
            Some(oid) => {
                let put = PutOAuth {
                    github: Some(github_id.as_i64().expect("Expect an i64 github id")),
                };
                diesel::update(t_oauth.find(oid))
                    .set(put)
                    .execute(conn)
                    .map_err(map_not_found("oauth", self.id))?;
            }
        }

        Ok(())
    }
}
