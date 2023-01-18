CQRS 是一种思想，主要是把请求分为 `Command` 和 `Query` 两种，并针对这两种请求采取不同的策略。

例如：https://github.com/banq/jivejdon

![image-20230112022148619](https://img.skygard.cn/image-20230112022148619.png)

参考：https://github.com/KodrAus/rust-web-app#commands-and-queries

实现了一个简单的 CQRS 模型：

```rust
pub trait Args {     
  type Output;
}
   
#[async_trait]
pub trait Command<TArgs: Args> {
  async fn execute(self, input: TArgs) -> TArgs::Output;
}
   
#[async_trait] 
pub trait Query<TArgs: Args> {     
  async fn execute(&self, input: TArgs) -> TArgs::Output;
}
```

它本质上就是将 `Command` 和 `Query` 绑定给两种相似的函数签名

- `fn execute(self, input: TArgs) -> TArgs::Output`  ——  Command
- `fn execute(&self, input: TArgs) -> TArgs::Output`  ——  Query

它们唯一的区别就是 `self` 与 `&self`

对于一个数据模型 model，为他实现 `Command` 和 `Query` 两种 trait，只有 `Command` 的实现可以修改 model 本身。`Query` 因持有不可变引用所以不能修改 model。

并且，`self` 可以取引用得到 `&self`，而反过来不行，也就是说，可以在 `Command` 中使用 `Query` ，而反过来不行。

**它本身并不提供任何 CQRS 模型的实现，这种设计模型只提供一个框架便于实现各种 CQRS，以及为了便于下一步的依赖注入**