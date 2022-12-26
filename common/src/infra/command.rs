// 规范化读写时的函数签名
// 读：使用 &self 不夺取 Read 所有权，可以多次读取
// 写：使用 self 夺取 Write 所有权，只能一次写入

use async_trait::async_trait;
use std::future::Future;

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

#[async_trait]
impl<TArgs, TWrite, TFuture> Command<TArgs> for TWrite
where
    TArgs: Args + Send + 'static,
    TWrite: (FnOnce(TArgs) -> TFuture) + Send,
    TFuture: Future<Output = TArgs::Output> + Send,
{
    async fn execute(self, input: TArgs) -> TArgs::Output {
        self(input).await
    }
}

#[async_trait]
impl<TArgs, TRead, TFuture> Query<TArgs> for TRead
where
    TArgs: Args + Send + 'static,
    TRead: (Fn(TArgs) -> TFuture) + Sync,
    TFuture: Future<Output = TArgs::Output> + Send,
{
    async fn execute(&self, input: TArgs) -> TArgs::Output {
        self(input).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn write_read_test() {
        struct AddValue(i32);
        struct GetLen;

        impl Args for GetLen {
            type Output = usize;
        }

        impl Args for AddValue {
            type Output = ();
        }

        let mut data = vec![];

        fn add_value(data: &mut Vec<i32>) -> impl Command<AddValue> + '_ {
            move |value: AddValue| async move {
                data.push(value.0);
            }
        }

        fn get_len(data: &[i32]) -> impl Query<GetLen> + '_ {
            move |_| async move { data.len() }
        }

        let write = add_value(&mut data);

        // panic because it borrowed as mutable before
        // let query = get_len(&data);

        write.execute(AddValue(1)).await;
        assert_eq!(data, vec![1]);

        let read = get_len(&data);
        assert_eq!(1, read.execute(GetLen).await);
        assert_eq!(1, read.execute(GetLen).await);
    }
}
