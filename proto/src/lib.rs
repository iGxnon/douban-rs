pub mod pb {
    pub mod auth {
        pub mod token {
            pub mod v1 {
                include!("./gen/auth.token.v1.rs");
            }
        }
    }
}

#[macro_export]
macro_rules! impl_args {
    (
        $(
            ($req:ty, $res:ty)
        )+
    ) => {
        $(
            impl common::infra::Args for $req {
                type Output = Result<$res, common::status::ext::GrpcStatus>;
            }
        )+
    };
}

impl_args!((
    pb::auth::token::v1::ParseTokenReq,
    pb::auth::token::v1::ParseTokenRes
)(
    pb::auth::token::v1::GenerateTokenReq,
    pb::auth::token::v1::GenerateTokenRes
)(
    pb::auth::token::v1::RefreshTokenReq,
    pb::auth::token::v1::RefreshTokenRes
)(
    pb::auth::token::v1::ClearCacheReq,
    pb::auth::token::v1::ClearCacheRes
));
