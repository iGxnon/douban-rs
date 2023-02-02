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
    include_proto!(auth, token, v1);
    include_proto!(user, sys, v1);
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

impl_args! {
    (auth, token, v1) {
        (ParseTokenReq, ParseTokenRes);
        (GenerateTokenReq, GenerateTokenRes);
        (RefreshTokenReq, RefreshTokenRes);
        (ClearCacheReq, ClearCacheRes);
    }
    (user, sys, v1) {
        (LoginReq, LoginRes);
        (RegisterReq, RegisterRes);
        (BindReq, BindRes);
    }
}
