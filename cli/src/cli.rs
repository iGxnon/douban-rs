pub use clap::Parser;

#[derive(Parser)] // requires `derive` feature
#[command(name = "douban-web")]
#[command(bin_name = "douban-rs")]
pub enum DoubanWeb {
    Deploy(Deploy),
    Update(Update),
}

#[derive(clap::Args)]
#[command(about = "Deploy douban services locally", long_about = None)]
pub struct Deploy {
    #[arg(long, short = 'p', value_name = "DEPLOY_DIR")]
    pub path: Option<std::path::PathBuf>,

    #[arg(required = true, value_name = "SERVICE_NAME")]
    pub service: Vec<String>,
}

#[derive(clap::Args, Debug)]
pub struct Update {}

pub mod handle {}
