#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetReq {
    #[prost(int64, optional, tag = "1")]
    pub id: ::core::option::Option<i64>,
    #[prost(string, optional, tag = "2")]
    pub imdb: ::core::option::Option<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListReq {
    /// search keyword
    #[prost(string, optional, tag = "1")]
    pub keyword: ::core::option::Option<::prost::alloc::string::String>,
    /// restrict to a movie
    #[prost(int64, optional, tag = "2")]
    pub movie_id: ::core::option::Option<i64>,
    #[prost(message, optional, tag = "3")]
    pub slice: ::core::option::Option<super::super::super::common::v1::Slice>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PutReq {
    #[prost(message, optional, tag = "1")]
    pub payload: ::core::option::Option<CelebrityPayload>,
    #[prost(int64, repeated, tag = "2")]
    pub as_actor_movies_id: ::prost::alloc::vec::Vec<i64>,
    #[prost(int64, repeated, tag = "5")]
    pub as_director_movies_id: ::prost::alloc::vec::Vec<i64>,
    #[prost(int64, repeated, tag = "6")]
    pub as_writer_movies_id: ::prost::alloc::vec::Vec<i64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DelReq {
    #[prost(int64, tag = "1")]
    pub id: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CelebrityPayload {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "2")]
    pub name_en: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub pic_url: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, tag = "4")]
    pub gender: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub imdb: ::prost::alloc::string::String,
    #[prost(string, tag = "6")]
    pub info: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetRes {
    #[prost(int64, tag = "1")]
    pub id: i64,
    #[prost(message, optional, tag = "2")]
    pub payload: ::core::option::Option<CelebrityPayload>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListRes {
    #[prost(message, repeated, tag = "1")]
    pub gets: ::prost::alloc::vec::Vec<GetRes>,
}
/// Generated client implementations.
pub mod celebrity_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// CRUD
    #[derive(Debug, Clone)]
    pub struct CelebrityServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl CelebrityServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> CelebrityServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> CelebrityServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            CelebrityServiceClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        pub async fn get_celebrity(
            &mut self,
            request: impl tonic::IntoRequest<super::GetReq>,
        ) -> Result<tonic::Response<super::GetRes>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/movie.celebrity.v1.CelebrityService/GetCelebrity",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn list_celebrities(
            &mut self,
            request: impl tonic::IntoRequest<super::ListReq>,
        ) -> Result<tonic::Response<super::ListRes>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/movie.celebrity.v1.CelebrityService/ListCelebrities",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn list_actors(
            &mut self,
            request: impl tonic::IntoRequest<super::ListReq>,
        ) -> Result<tonic::Response<super::ListRes>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/movie.celebrity.v1.CelebrityService/ListActors",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn list_directors(
            &mut self,
            request: impl tonic::IntoRequest<super::ListReq>,
        ) -> Result<tonic::Response<super::ListRes>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/movie.celebrity.v1.CelebrityService/ListDirectors",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn list_writers(
            &mut self,
            request: impl tonic::IntoRequest<super::ListReq>,
        ) -> Result<tonic::Response<super::ListRes>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/movie.celebrity.v1.CelebrityService/ListWriters",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn put_celebrity(
            &mut self,
            request: impl tonic::IntoRequest<super::PutReq>,
        ) -> Result<
            tonic::Response<super::super::super::super::common::v1::EmptyRes>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/movie.celebrity.v1.CelebrityService/PutCelebrity",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn del_celebrity(
            &mut self,
            request: impl tonic::IntoRequest<super::DelReq>,
        ) -> Result<
            tonic::Response<super::super::super::super::common::v1::EmptyRes>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/movie.celebrity.v1.CelebrityService/DelCelebrity",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod celebrity_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with CelebrityServiceServer.
    #[async_trait]
    pub trait CelebrityService: Send + Sync + 'static {
        async fn get_celebrity(
            &self,
            request: tonic::Request<super::GetReq>,
        ) -> Result<tonic::Response<super::GetRes>, tonic::Status>;
        async fn list_celebrities(
            &self,
            request: tonic::Request<super::ListReq>,
        ) -> Result<tonic::Response<super::ListRes>, tonic::Status>;
        async fn list_actors(
            &self,
            request: tonic::Request<super::ListReq>,
        ) -> Result<tonic::Response<super::ListRes>, tonic::Status>;
        async fn list_directors(
            &self,
            request: tonic::Request<super::ListReq>,
        ) -> Result<tonic::Response<super::ListRes>, tonic::Status>;
        async fn list_writers(
            &self,
            request: tonic::Request<super::ListReq>,
        ) -> Result<tonic::Response<super::ListRes>, tonic::Status>;
        async fn put_celebrity(
            &self,
            request: tonic::Request<super::PutReq>,
        ) -> Result<
            tonic::Response<super::super::super::super::common::v1::EmptyRes>,
            tonic::Status,
        >;
        async fn del_celebrity(
            &self,
            request: tonic::Request<super::DelReq>,
        ) -> Result<
            tonic::Response<super::super::super::super::common::v1::EmptyRes>,
            tonic::Status,
        >;
    }
    /// CRUD
    #[derive(Debug)]
    pub struct CelebrityServiceServer<T: CelebrityService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: CelebrityService> CelebrityServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for CelebrityServiceServer<T>
    where
        T: CelebrityService,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/movie.celebrity.v1.CelebrityService/GetCelebrity" => {
                    #[allow(non_camel_case_types)]
                    struct GetCelebritySvc<T: CelebrityService>(pub Arc<T>);
                    impl<T: CelebrityService> tonic::server::UnaryService<super::GetReq>
                    for GetCelebritySvc<T> {
                        type Response = super::GetRes;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetReq>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).get_celebrity(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetCelebritySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/movie.celebrity.v1.CelebrityService/ListCelebrities" => {
                    #[allow(non_camel_case_types)]
                    struct ListCelebritiesSvc<T: CelebrityService>(pub Arc<T>);
                    impl<T: CelebrityService> tonic::server::UnaryService<super::ListReq>
                    for ListCelebritiesSvc<T> {
                        type Response = super::ListRes;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListReq>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).list_celebrities(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ListCelebritiesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/movie.celebrity.v1.CelebrityService/ListActors" => {
                    #[allow(non_camel_case_types)]
                    struct ListActorsSvc<T: CelebrityService>(pub Arc<T>);
                    impl<T: CelebrityService> tonic::server::UnaryService<super::ListReq>
                    for ListActorsSvc<T> {
                        type Response = super::ListRes;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListReq>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).list_actors(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ListActorsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/movie.celebrity.v1.CelebrityService/ListDirectors" => {
                    #[allow(non_camel_case_types)]
                    struct ListDirectorsSvc<T: CelebrityService>(pub Arc<T>);
                    impl<T: CelebrityService> tonic::server::UnaryService<super::ListReq>
                    for ListDirectorsSvc<T> {
                        type Response = super::ListRes;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListReq>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).list_directors(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ListDirectorsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/movie.celebrity.v1.CelebrityService/ListWriters" => {
                    #[allow(non_camel_case_types)]
                    struct ListWritersSvc<T: CelebrityService>(pub Arc<T>);
                    impl<T: CelebrityService> tonic::server::UnaryService<super::ListReq>
                    for ListWritersSvc<T> {
                        type Response = super::ListRes;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListReq>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).list_writers(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ListWritersSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/movie.celebrity.v1.CelebrityService/PutCelebrity" => {
                    #[allow(non_camel_case_types)]
                    struct PutCelebritySvc<T: CelebrityService>(pub Arc<T>);
                    impl<T: CelebrityService> tonic::server::UnaryService<super::PutReq>
                    for PutCelebritySvc<T> {
                        type Response = super::super::super::super::common::v1::EmptyRes;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::PutReq>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).put_celebrity(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = PutCelebritySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/movie.celebrity.v1.CelebrityService/DelCelebrity" => {
                    #[allow(non_camel_case_types)]
                    struct DelCelebritySvc<T: CelebrityService>(pub Arc<T>);
                    impl<T: CelebrityService> tonic::server::UnaryService<super::DelReq>
                    for DelCelebritySvc<T> {
                        type Response = super::super::super::super::common::v1::EmptyRes;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DelReq>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).del_celebrity(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DelCelebritySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: CelebrityService> Clone for CelebrityServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: CelebrityService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: CelebrityService> tonic::server::NamedService for CelebrityServiceServer<T> {
        const NAME: &'static str = "movie.celebrity.v1.CelebrityService";
    }
}
