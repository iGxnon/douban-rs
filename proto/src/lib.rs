macro_rules! include_proto {
    ($proj:ident, $domain:ident, $ver:ident) => {
        pub mod $proj {
            pub mod $domain {
                pub mod $ver {
                    include!(concat!(
                        "./gen/",
                        stringify!($proj),
                        ".",
                        stringify!($domain),
                        ".",
                        stringify!($ver),
                        ".rs"
                    ));
                }
            }
        }
    };
}

pub mod pb {
    // include_proto!(auth, token, v1);
    pub mod auth {
        pub mod token {
            pub mod v1 {
                include!("./gen/auth.token.v1.rs");
            }
        }
    }
    // include_proto!(user, sys, v1);
    pub mod user {
        pub mod sys {
            pub mod v1 {
                include!("./gen/user.sys.v1.rs");
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
            ($proj:ident, $domain:ident, $ver:ident) {
                $(
                    ($req:ident, $res:ident);
                )+
            }
        )+
    ) => {
        $(
            $(
                impl common::infra::Args for pb::$proj::$domain::$ver::$req {
                    type Output = Result<pb::$proj::$domain::$ver::$res, common::status::ext::GrpcStatus>;
                }
            )+
        )+
    };
}

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
                impl common::infra::Args for pb::$proj::$domain::$ver::$req {
                    type Output = Result<pb::common::v1::EmptyRes, common::status::ext::GrpcStatus>;
                }
            )+
        )+
    };
}

impl_args! {
    (auth, token, v1) {
        (ParseTokenReq, ParseTokenRes);
        (GenerateTokenReq, GenerateTokenRes);
        (RefreshTokenReq, RefreshTokenRes);

    }
    (user, sys, v1) {
        (LoginReq, LoginRes);
    }
}

impl_empty_res! {
    (auth, token, v1) {
        ClearCacheReq,
    }
    (user, sys, v1) {
        RegisterReq,
        BindReq,
    }
}
