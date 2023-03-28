use std::fmt::Display;
use common::*;
use diesel::prelude::*;
use common::status::prelude::*;
use migration::t_movies;
use chrono::NaiveDateTime;

pub type MovieId = infra::Id<Movie>;

#[derive(Queryable)]
pub struct Movie {
    id: i64,
    title: String,
    pic_url: Option<String>,
    name: String,
    alias_name: Option<String>,
    language: String,
    time_length: i32,
    released_date: NaiveDateTime,
    imdb: String,
    plot: String,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(AsChangeset, Default)]
#[diesel(table_name = t_movies)]
pub struct PutMovie<'a> {
    title: &'a str,
    pic_url: Option<&'a str>,
    name: &'a str,
    alias_name: Option<&'a str>,
    language: &'a str,
    time_length: i32,
    released_date: NaiveDateTime,
    imdb: &'a str,
    plot: &'a str
}

fn map_not_found<'a>(
    scope: &'a str,
    identifier: impl Display + 'a,
) -> impl Fn(diesel::result::Error) -> tonic::Status + 'a {
    move |e| {
        if e == diesel::NotFound {
            return not_found!(format!("{}({})", scope, identifier));
        }
        internal!(format!("Database connection error: {}", e))
    }
}

impl Movie {
    pub fn put_movie(

    ) {

    }

    pub fn query_id(
        mid: MovieId,
        conn: &mut PgConnection,
    ) -> GrpcResult<Movie> {
        use migration::t_movies::dsl::*;

        let mid = mid.as_i64().expect("Expect an i64 id");
        let movie: Movie = t_movies
            .find(mid)
            .first(conn)
            .map_err(map_not_found("movie", mid))?;
        Ok(movie)
    }

    pub fn query_imdb(
        imdb_str: &str,
        conn: &mut PgConnection,
    ) -> GrpcResult<Movie> {
        use migration::t_movies::dsl::*;

        let movie: Movie = t_movies
            .filter(imdb.eq(imdb_str))
            .first(conn)
            .map_err(map_not_found("movie", imdb_str))?;
        Ok(movie)
    }
}