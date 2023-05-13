pub mod pb {
    pub mod auth {
        pub mod token {
            pub mod v1 {
                include!("./gen/auth.token.v1.rs");
            }
        }
    }
    pub mod user {
        pub mod sys {
            pub mod v1 {
                include!("./gen/user.sys.v1.rs");
            }
        }
    }
    pub mod movie {
        pub mod movie {
            pub mod v1 {
                include!("./gen/movie.movie.v1.rs");
            }
        }
        pub mod celebrity {
            pub mod v1 {
                include!("./gen/movie.celebrity.v1.rs");
            }
        }
        pub mod score {
            pub mod v1 {
                include!("./gen/movie.score.v1.rs");
            }
        }
    }
    pub mod common {
        pub mod v1 {
            include!("./gen/common.v1.rs");
        }
    }
}

macro_rules! impl_args {
    (
        $(
            $ty:ident($proj:ident, $domain:ident, $ver:ident) {
                $(
                    ($req:ident, $res:ident);
                )+
            }
        )+
    ) => {
        $(
            $(
                impl common::infra::$ty for pb::$proj::$domain::$ver::$req {
                    type Output = Result<pb::$proj::$domain::$ver::$res, common::status::ext::GrpcStatus>;
                }
            )+
        )+
    };
}

/// empty response must be a command
macro_rules! impl_empty_res {
    (
        $(
            ($proj:ident, $domain:ident, $ver:ident) {
                $(
                    $req:ident,
                )+
            }
        )+
    ) => {
        $(
            $(
                impl common::infra::CommandArgs for pb::$proj::$domain::$ver::$req {
                    type Output = Result<pb::common::v1::EmptyRes, common::status::ext::GrpcStatus>;
                }
            )+
        )+
    };
}

impl_args! {
    CommandArgs(auth, token, v1) {
        (GenerateTokenReq, GenerateTokenRes);
        (RefreshTokenReq, RefreshTokenRes);
    }
    QueryArgs(user, sys, v1) {
        (LoginReq, LoginRes);
    }
    QueryArgs(auth, token, v1) {
        (ParseTokenReq, ParseTokenRes);
    }
    QueryArgs(movie, movie, v1) {
        (GetReq, GetRes);
        (ListReq, ListRes);
    }
    QueryArgs(movie, celebrity, v1) {
        (GetReq, GetRes);
        (ListReq, ListRes);
    }
    QueryArgs(movie, score, v1) {
        (GetReq, GetRes);
    }
}

// empty response must be a command
impl_empty_res! {
    (auth, token, v1) {
        ClearCacheReq,
    }
    (user, sys, v1) {
        RegisterReq,
        BindReq,
    }
    (movie, movie, v1) {
        PutReq,
        DelReq,
    }
    (movie, celebrity, v1) {
        PutReq,
        DelReq,
    }
    (movie, score, v1) {
        ScoreReq,
    }
}
