-- Your SQL goes here

create table t_oauth
(
    id     serial8
        constraint t_oauth_pk
            primary key,
    github bigint default null
);

comment on table t_oauth is 'oauth table for user, used for scalable oauth apps';

comment on column t_oauth.id is 'id of this oauth';

comment on constraint t_oauth_pk on t_oauth is 'pk of oauth';

comment on column t_oauth.github is 'github oauth id';

create index t_oauth_github_index
    on t_oauth (github);

comment on index t_oauth_github_index is 'index for github used to locate oauth id of a user';

create table t_users
(
    id              serial8
        constraint t_users_pk
            primary key,
    oauth_id        bigint       default null
        constraint t_users_oauth_fk
            references t_oauth (id),
    username        varchar(64)                not null,
    phone           varchar(128) default null,
    email           varchar(256) default null,
    nickname        varchar(64)                not null,
    hashed_password varchar(64)                not null,
    create_at       timestamp    default now() not null,
    update_at       timestamp    default now() not null,
    delete_at       timestamp    default null,
    last_login      timestamp    default null
);

comment on table t_users is 'douban_rs users table';

comment on column t_users.id is 'id of user';

comment on constraint t_users_pk on t_users is 'pk of users table';

comment on column t_users.oauth_id is 'oauth_id of user';

comment on constraint t_users_oauth_fk on t_users is 'oauth table key';

comment on column t_users.username is 'username of user';

comment on column t_users.phone is 'phone number of user';

comment on column t_users.email is 'email of user';

comment on column t_users.nickname is 'nickname of user, default is username, cannot used to login';

comment on column t_users.hashed_password is 'password which is hashed';

comment on column t_users.create_at is 'user create timestamp';

comment on column t_users.update_at is 'user info update timestamp';

comment on column t_users.delete_at is 'soft delete timestamp';

comment on column t_users.last_login is 'last login timestamp';

create index t_users_email_index
    on t_users (email);

comment on index t_users_email_index is 'email index used to locate user';

create index t_users_phone_index
    on t_users (phone);

comment on index t_users_phone_index is 'phone number used to locate user';

create unique index t_users_username_uindex
    on t_users (username);

