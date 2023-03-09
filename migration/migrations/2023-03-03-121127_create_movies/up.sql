-- Your SQL goes here

-- auto-generated definition
create table t_movies
(
    id            bigserial
        constraint t_movies_pk
            primary key,
    title         varchar(512)            not null,
    pic_url       varchar(512),
    name          varchar(512)            not null,
    alias_name    varchar(512),
    language      varchar(128)            not null,
    time_length   integer                 not null,
    released_date date                    not null,
    imdb          varchar(64)             not null,
    plot          text      default ''    not null,
    created_at    timestamp default now() not null,
    updated_at    timestamp default now() not null
);

select diesel_manage_updated_at('t_movies');

comment on table t_movies is 'movies table, it prefers to be immutable';

comment on column t_movies.id is 'pk of movies table';

comment on constraint t_movies_pk on t_movies is 'pk of movies table';

comment on column t_movies.title is 'the movie''s title on the web page';

comment on column t_movies.pic_url is 'picture url of the movie';

comment on column t_movies.name is 'movie name';

comment on column t_movies.alias_name is 'alias name of this movie';

comment on column t_movies.language is 'the language of the movie';

comment on column t_movies.time_length is 'the length of the movie, minutes';

comment on column t_movies.released_date is 'released date of this movie';

comment on column t_movies.imdb is 'IMDb number of this movie';

comment on column t_movies.plot is 'the plot of this movie';

create index t_movies_title_index
    on t_movies (title);

create index t_movies_imdb_index
    on t_movies (imdb);

comment on index t_movies_title_index is 'movie title index';

comment on index t_movies_imdb_index is 'movie imdb index';

create table t_celebrities
(
    id         bigserial
        constraint t_celebrities_pk
            primary key,
    name       varchar(128)            not null,
    name_en    varchar(128)            not null,
    pic_url    varchar(512)            not null,
    gender     varchar(64)             not null,
    imdb       varchar(64)             not null,
    info       text                    not null,
    created_at timestamp default now() not null,
    updated_at timestamp default now() not null
);

select diesel_manage_updated_at('t_celebrities');

comment on table t_celebrities is 'celebrities in the film industry, most field are immutable';

comment on column t_celebrities.id is 'pk of the celebrity';

comment on constraint t_celebrities_pk on t_celebrities is 'pk of the table';

comment on column t_celebrities.name is 'the name of the celebrity';

comment on column t_celebrities.name_en is 'English name of the celebrity';

comment on column t_celebrities.gender is 'gender of the celebrity';

comment on column t_celebrities.imdb is 'the imdb number of this celebrity';

comment on column t_celebrities.info is 'the description of the celebrity';

comment on column t_celebrities.pic_url is 'pic url of this celebrity';

create index t_celebrities_name_index
    on t_celebrities (name);

create index t_celebrities_imdb_index
    on t_celebrities (imdb);

comment on index t_celebrities_name_index is 'name index';

comment on index t_celebrities_imdb_index is 'celebrity imdb index';

create table t_movies_directors
(
    id  bigserial
        constraint t_movies_directors_pk
            primary key,
    mid bigint not null
        constraint t_movies_directors_t_movies_id_fk
            references t_movies,
    cid bigint not null
        constraint t_movies_directors_t_celebrities_id_fk
            references t_celebrities
);

comment on table t_movies_directors is 'movie director MvM pair';

comment on column t_movies_directors.mid is 'movie id';

comment on column t_movies_directors.cid is 'director id(celebrity id)';

create table t_movies_writers
(
    id  bigserial
        constraint t_movies_writers_pk
            primary key,
    mid bigint not null
        constraint t_movies_writers_t_movies_id_fk
            references t_movies,
    cid bigint not null
        constraint t_movies_writers_t_celebrities_id_fk
            references t_celebrities
);

comment on table t_movies_writers is 'movies writers MvM pair';

comment on column t_movies_writers.mid is 'fk of movies';

comment on column t_movies_writers.cid is 'fk of celebrities';

create table t_movies_actors
(
    id  bigserial
        constraint t_movies_actors_pk
            primary key,
    mid bigint not null
        constraint t_movies_actors_t_movies_id_fk
            references t_movies,
    cid bigint not null
        constraint t_movies_actors_t_celebrities_id_fk
            references t_celebrities
);

comment on table t_movies_actors is 'movies actors MvM pair';

comment on column t_movies_actors.mid is 'fk of movies';

comment on column t_movies_actors.cid is 'fk of celebrities';

create table t_movies_categories
(
    id       bigserial
        constraint t_movies_categories_pk
            primary key,
    mid      bigint       not null
        constraint t_movies_categories_t_movies_id_fk
            references t_movies,
    category varchar(128) not null
);

comment on table t_movies_categories is 'movies categories MvM pair';

comment on column t_movies_categories.mid is 'fk of movies';

comment on column t_movies_categories.category is 'category of the movie';

create index t_movies_category_index
    on t_movies_categories (category);

create table t_movies_country
(
    id      bigserial
        constraint t_movies_countries_pk
            primary key,
    mid     bigint       not null
        constraint t_movies_countries_t_movies_id_fk
            references t_movies,
    country varchar(128) not null
);

comment on table t_movies_country is 'movies countries MvM pair';

comment on column t_movies_country.mid is 'fk of movies';

comment on column t_movies_country.country is 'country of the movie';

create index t_movies_country_index
    on t_movies_country (country);

create table t_movies_scores
(
    id         bigserial
        constraint t_movies_scores_pk
            primary key,
    mid        bigint                  not null
        constraint t_movies_scores_t_movies_id_fk
            references t_movies,
    created_at timestamp default now() not null,
    updated_at timestamp default now() not null,
    score_avg  float     default 0     not null,
    cnt_1      bigint    default 0     not null,
    cnt_2      bigint    default 0     not null,
    cnt_3      bigint    default 0     not null,
    cnt_4      bigint    default 0     not null,
    cnt_5      bigint    default 0     not null
);

select diesel_manage_updated_at('t_movies_scores');

select diesel_manage_set_score_avg('t_movies_scores');

comment on table t_movies_scores is 'scores of a movie';

comment on column t_movies_scores.id is 'pk of the table';

comment on column t_movies_scores.mid is 'movie id';

comment on column t_movies_scores.cnt_1 is '1 star count';

comment on column t_movies_scores.cnt_2 is '2 start count';

comment on column t_movies_scores.cnt_3 is '3 star count';

comment on column t_movies_scores.cnt_4 is '4 star count';

comment on column t_movies_scores.cnt_5 is '5 star count';