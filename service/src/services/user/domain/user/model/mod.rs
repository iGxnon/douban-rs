pub mod store;

use crate::user::domain::user::model::store::{PgGet, PgSet};
use base64::Engine;
use chrono::NaiveDateTime;
use common::status::ext::GrpcResult;
use common::*;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use hmac::{Hmac, Mac};
use migration::*;

pub type UserId = infra::Id<User>;
pub type OAuthId = infra::Id<OAuth>;
pub type GithubId = infra::Id<Github>;

#[derive(Queryable)]
pub struct User {
    id: i64,
    oauth_id: Option<i64>,
    username: String,
    phone: Option<String>,
    email: Option<String>,
    nickname: String,
    hashed_password: String,
    creat_at: NaiveDateTime,
    update_at: NaiveDateTime,
    delete_at: Option<NaiveDateTime>,
    last_login: Option<NaiveDateTime>,
}

#[derive(Insertable)]
#[diesel(table_name = t_users)]
pub struct NewUser<'a> {
    username: &'a str,
    nickname: &'a str,
    hashed_password: String,
}

#[derive(AsChangeset)]
#[diesel(table_name = t_users)]
pub struct PutUser<'a> {
    id: Option<i64>, // used to find user, cannot update
    phone: Option<&'a str>,
    email: Option<&'a str>,
    nickname: Option<&'a str>,
    hashed_password: Option<&'a str>,
    update_at: Option<NaiveDateTime>, // this is set automatically by diesel if it is None
    last_login: Option<NaiveDateTime>,
}

#[derive(Queryable, Debug)]
pub struct OAuth {
    id: i64,
    github: Option<i64>,
}

#[derive(Insertable)]
#[diesel(table_name = t_oauth)]
pub struct NewOAuth {
    github: Option<i64>,
}

#[derive(AsChangeset)]
#[diesel(table_name = t_oauth)]
pub struct PutOAuth {
    id: Option<i64>,
    github: Option<i64>,
}

pub struct Github(i64);

impl<'a> NewUser<'a> {
    pub(super) fn new(name: &'a str, password: &'a str, secret: &str) -> Self {
        let mut mac = Hmac::<sha2::Sha256>::new_from_slice(secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(password.as_bytes());
        let output = Mac::finalize(mac).into_bytes();
        let hashed_password = base64::prelude::BASE64_STANDARD.encode(output);
        Self {
            username: name,
            nickname: name,
            hashed_password,
        }
    }

    pub(super) async fn register(
        self,
        pool: &'static Pool<ConnectionManager<PgConnection>>,
    ) -> GrpcResult<()> {
        infra::Command::execute(self, PgSet::new(pool)).await
    }
}

impl User {
    pub(super) fn with_id(id: UserId) -> Self {
        Self {
            id: id.as_i64().expect("Need an u64 id"),
            oauth_id: None,
            username: "".to_string(),
            phone: None,
            email: None,
            nickname: "".to_string(),
            hashed_password: "".to_string(),
            creat_at: Default::default(),
            update_at: Default::default(),
            delete_at: None,
            last_login: None,
        }
    }

    async fn check_password(
        &self,
        password: &str,
        secret: &str,
        pool: &'static Pool<ConnectionManager<PgConnection>>,
    ) -> GrpcResult<bool> {
        let mut mac = Hmac::<sha2::Sha256>::new_from_slice(secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(password.as_bytes());
        let user = infra::Query::execute(self, PgGet::new(pool)).await?;
        Ok(Mac::verify_slice(mac, user.hashed_password.as_bytes()).is_ok())
    }
}
