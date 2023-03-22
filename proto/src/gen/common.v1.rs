#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EmptyRes {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Slice {
    /// slice option 1
    #[prost(message, optional, tag = "1")]
    pub limit: ::core::option::Option<ByLimit>,
    /// slice option 2
    #[prost(message, optional, tag = "2")]
    pub page: ::core::option::Option<ByPage>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ByLimit {
    #[prost(int32, tag = "1")]
    pub limit: i32,
    #[prost(int32, tag = "2")]
    pub offset: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ByPage {
    #[prost(int32, tag = "1")]
    pub page: i32,
    #[prost(int32, tag = "4")]
    pub per_page: i32,
}
