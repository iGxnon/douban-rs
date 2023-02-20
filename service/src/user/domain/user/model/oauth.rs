use common::infra;
use diesel::prelude::*;
use migration::t_oauth;

pub type GithubId = infra::Id<Github>;
pub struct Github(i64);

#[derive(Queryable, Debug)]
pub(super) struct OAuth {
    pub(super) id: i64,
    pub(super) github: Option<i64>,
}

#[derive(Insertable, Default)]
#[diesel(table_name = t_oauth)]
pub(super) struct NewOAuth {
    pub(super) github: Option<i64>,
}

#[derive(AsChangeset)]
#[diesel(table_name = t_oauth)]
pub(super) struct PutOAuth {
    pub(super) github: Option<i64>,
}
