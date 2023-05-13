## douban-rs (WIP) 

> 这是一个实验性项目，旨在探究 rust web 开发的最佳实践
> 
> 架构不能脱离实际，这里我选取了 `豆瓣` 这个实际应用来实践这个架构
>

使用到的 Web 开发技术栈为:

- [axum](https://github.com/tokio-rs/axum) —— HTTP server
- [tower](https://github.com/tower-rs/tower) —— Infrastructure for service building
- [tonic](https://github.com/hyperium/tonic) —— gRPC implementation

## 前言

Rust 强大的抽象能力给构建后端项目提供了很多种可能，导致我在编写时前前后后添加，删除了很多代码

基本上敲定如下的设计思路：

1. [混合架构](./design-pattern/混合架构.md)
1. [配置](./design-pattern/配置.md)
2. [CQRS (Command Query Responsibility Segregation)](./design-pattern/CQRS.md)
3. [依赖注入](./design-pattern/依赖注入.md)
4. [错误处理](./design-pattern/错误处理.md)

以上是最基本的设计思路

## 细节

总体上，整个项目分为 4 个 crate 

- cli           ———— 便于快速部署的脚本(WIP)
- common-rs     ———— 通用包/基础脚手架
- migration     ———— 关系数据库ORM对象声明/表结构迁移
- proto         ———— 服务间通信协议(IDL)定义
- service       ———— 服务

对于 service crate，分成各个"广域"，例如 `auth`，`user`，每个广域下可以有多个"辖域"，这样可以一定程度来控制不同域之间的访问权限。例如可以控制 `user` 广域下的"用户基础信息资源"公开到全域中，各个广域均可访问，而一些隐私信息可以只公开到 `user` 广域下的辖域访问。

每个广域都可以暴露 API，各类 API 会以不同的风格划分到对应的包下，例如 `restful` 风格的 API 会被划分到广域下的 `rest` 包下，`protobuf`、 `thrift ` 等服务间通信的 API 将会被划分到 `rpc` 包下，`graphql` 风格的 API 会被划分到 `graphql` 文件夹下等等。至于每类 API 可以暴露的数量、可以暴露多少种 API、每类 API 是直接对接辖域还是翻译自其他 API，这取决于实际需求，例如 `auth` 广域我只暴露了一个 `rpc` 形式的 `api`，因为我不需要客户端直接访问 `auth`，而是让服务去访问；而对于 `user` 广域的 `rest` API，它的实现只是翻译 gRPC 的 API，因为我不仅仅需要客户端访问 `user` 下的辖域，也需要其他的服务也可以访问。

每个辖域可分为三个子包

- command    ———— 将 command 请求对接到 model 中
- query    ———— 将 query 请求对接到 model 中
- model    ———— 领域模型

`command` 和 `query` 的任务比较简单，主要为：`参数检查`，`依赖注入` 和 `对接领域模型`，**我希望所有的逻辑都最好只在领域模型中**，这样做的好处是避免逻辑分散到各层之中，难以把握全局逻辑关系。缺点就是领域模型可能会变得异常庞大，需要经常重构，需要编码者有一定的抽象和解耦能力。

这也称为 `DDD(Domain-Driven Design)`

