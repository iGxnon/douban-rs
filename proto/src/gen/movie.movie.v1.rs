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
    #[prost(string, optional, tag = "3")]
    pub keyword: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "4")]
    pub language: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, optional, tag = "5")]
    pub time_range: ::core::option::Option<TimeRange>,
    #[prost(int32, optional, tag = "6")]
    pub released_years: ::core::option::Option<i32>,
    #[prost(int64, optional, tag = "7")]
    pub actor_id: ::core::option::Option<i64>,
    #[prost(string, optional, tag = "8")]
    pub category: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "9")]
    pub country: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(int64, optional, tag = "10")]
    pub director_id: ::core::option::Option<i64>,
    #[prost(message, optional, tag = "11")]
    pub score_range: ::core::option::Option<ScoreRange>,
    #[prost(int64, optional, tag = "12")]
    pub writer_id: ::core::option::Option<i64>,
    #[prost(message, optional, tag = "13")]
    pub slice: ::core::option::Option<super::super::super::common::v1::Slice>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PutReq {
    #[prost(message, optional, tag = "1")]
    pub payload: ::core::option::Option<MoviePayload>,
    #[prost(int64, repeated, tag = "2")]
    pub actors_id: ::prost::alloc::vec::Vec<i64>,
    #[prost(string, repeated, tag = "3")]
    pub categories: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(string, repeated, tag = "4")]
    pub countries: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(int64, repeated, tag = "5")]
    pub directors_id: ::prost::alloc::vec::Vec<i64>,
    #[prost(int64, repeated, tag = "6")]
    pub writers_id: ::prost::alloc::vec::Vec<i64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DelReq {
    #[prost(int64, tag = "1")]
    pub id: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TimeRange {
    /// unit (hour)
    #[prost(int32, tag = "1")]
    pub start: i32,
    #[prost(int32, tag = "2")]
    pub end: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ScoreRange {
    /// unit (1-5)
    #[prost(int32, tag = "1")]
    pub start: i32,
    #[prost(int32, tag = "2")]
    pub end: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MoviePayload {
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "2")]
    pub pic_url: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, tag = "3")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "4")]
    pub alias_name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, tag = "5")]
    pub language: ::prost::alloc::string::String,
    #[prost(int32, tag = "6")]
    pub time_length: i32,
    #[prost(message, optional, tag = "7")]
    pub release_date: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(string, tag = "8")]
    pub imdb: ::prost::alloc::string::String,
    #[prost(string, tag = "9")]
    pub plot: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetRes {
    #[prost(int64, tag = "1")]
    pub id: i64,
    #[prost(message, optional, tag = "2")]
    pub payload: ::core::option::Option<MoviePayload>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListRes {
    #[prost(message, repeated, tag = "1")]
    pub gets: ::prost::alloc::vec::Vec<GetRes>,
}
/// Generated client implementations.
pub mod movie_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// CRUD
    #[derive(Debug, Clone)]
    pub struct MovieServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl MovieServiceClient<tonic::transport::Channel> {
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
    impl<T> MovieServiceClient<T>
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
        ) -> MovieServiceClient<InterceptedService<T, F>>
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
            MovieServiceClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn get_movie(
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
                "/movie.movie.v1.MovieService/GetMovie",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn list_movies(
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
                "/movie.movie.v1.MovieService/ListMovies",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn put_movie(
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
                "/movie.movie.v1.MovieService/PutMovie",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn del_movie(
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
                "/movie.movie.v1.MovieService/DelMovie",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod movie_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with MovieServiceServer.
    #[async_trait]
    pub trait MovieService: Send + Sync + 'static {
        async fn get_movie(
            &self,
            request: tonic::Request<super::GetReq>,
        ) -> Result<tonic::Response<super::GetRes>, tonic::Status>;
        async fn list_movies(
            &self,
            request: tonic::Request<super::ListReq>,
        ) -> Result<tonic::Response<super::ListRes>, tonic::Status>;
        async fn put_movie(
            &self,
            request: tonic::Request<super::PutReq>,
        ) -> Result<
            tonic::Response<super::super::super::super::common::v1::EmptyRes>,
            tonic::Status,
        >;
        async fn del_movie(
            &self,
            request: tonic::Request<super::DelReq>,
        ) -> Result<
            tonic::Response<super::super::super::super::common::v1::EmptyRes>,
            tonic::Status,
        >;
    }
    /// CRUD
    #[derive(Debug)]
    pub struct MovieServiceServer<T: MovieService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: MovieService> MovieServiceServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for MovieServiceServer<T>
    where
        T: MovieService,
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
                "/movie.movie.v1.MovieService/GetMovie" => {
                    #[allow(non_camel_case_types)]
                    struct GetMovieSvc<T: MovieService>(pub Arc<T>);
                    impl<T: MovieService> tonic::server::UnaryService<super::GetReq>
                    for GetMovieSvc<T> {
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
                            let fut = async move { (*inner).get_movie(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetMovieSvc(inner);
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
                "/movie.movie.v1.MovieService/ListMovies" => {
                    #[allow(non_camel_case_types)]
                    struct ListMoviesSvc<T: MovieService>(pub Arc<T>);
                    impl<T: MovieService> tonic::server::UnaryService<super::ListReq>
                    for ListMoviesSvc<T> {
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
                            let fut = async move { (*inner).list_movies(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ListMoviesSvc(inner);
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
                "/movie.movie.v1.MovieService/PutMovie" => {
                    #[allow(non_camel_case_types)]
                    struct PutMovieSvc<T: MovieService>(pub Arc<T>);
                    impl<T: MovieService> tonic::server::UnaryService<super::PutReq>
                    for PutMovieSvc<T> {
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
                            let fut = async move { (*inner).put_movie(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = PutMovieSvc(inner);
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
                "/movie.movie.v1.MovieService/DelMovie" => {
                    #[allow(non_camel_case_types)]
                    struct DelMovieSvc<T: MovieService>(pub Arc<T>);
                    impl<T: MovieService> tonic::server::UnaryService<super::DelReq>
                    for DelMovieSvc<T> {
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
                            let fut = async move { (*inner).del_movie(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DelMovieSvc(inner);
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
    impl<T: MovieService> Clone for MovieServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: MovieService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: MovieService> tonic::server::NamedService for MovieServiceServer<T> {
        const NAME: &'static str = "movie.movie.v1.MovieService";
    }
}
