mod cli;

use cli::{DoubanWeb, Parser};

fn main() {
    match DoubanWeb::parse() {
        DoubanWeb::Deploy(args) => {
            println!("{:?}", args.service)
        }
        DoubanWeb::Update(args) => {
            println!("{:?}", args)
        }
    }
}
