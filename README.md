## douban-rs (WIP) 

> 使用 rust 对 [douban](https://github.com/mouse-douban/douban-web) 项目的后端进行重构

使用到的 Web 开发技术栈为:

- [axum](https://github.com/tokio-rs/axum) —— HTTP server
- [tower](https://github.com/tower-rs/tower) —— Infrastructure for service building
- [tonic](https://github.com/hyperium/tonic) —— gRPC implementation

## 前言

Rust 强大的抽象能力给构建后端项目提供了很多种可能，导致我在编写时前前后后添加，删除了很多代码

基本上敲定如下的设计思路：

1. [配置](./design-pattern/配置.md)

2. [CQRS (Command Query Responsibility Segregation)](./design-pattern/CQRS.md)

3. [依赖注入](./design-pattern/依赖注入.md)

4. Isn't Resolver A God Object？

   来自 https://github.com/KodrAus/rust-web-app#isnt-resolver-a-god-object 
   
   虽然在 OOP 编程中，推崇的是让一组对象相互协作式的工作，god object 是一个 bad design，但我写的是 rust 啊，连继承的特性都没有的 OOP 你跟我说什么？（bushi
   
   参考了 `rust-web-app` 中的 Resolver，这个项目中，每一个领域 (`domain`) 都有一个 `Resolver`，在这个 domain 的各个地方都有 Resolver 的一些方法实现，所以这就让 Resolver 渐渐变成一个 god object？
   
   我这里将 Resolver 下沉到了每一个 domain，与 rust-web-app 中不一样的是，我构建的是微服务架构的程序，各个 Resolver 之间可以没有关系，这减小了单个 Resolver 的体量。对于一个 domain 来说，它最主要的部分也只是几个 execute 函数的集合，单个 Resolver 主要服务好这几个函数即可。而 domain 体量逐渐膨胀后，应该考虑划分新的 domain 了，而不会对 Resolver 的体量有影响
   
5. [错误处理](./design-pattern/错误处理.md)

   

