use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(name = "geo-loc")]
#[command(about = "Print the host's current geographic location in a pipe-friendly format")]
#[command(version = "geo-loc 0.1.0\nProviders: corelocation (macOS), ip (portable fallback)")]
pub struct Args {
    #[arg(long, value_enum, default_value = "plain")]
    pub format: Format,

    #[arg(long, value_enum, default_value = "auto")]
    pub provider: Provider,

    #[arg(long)]
    pub accuracy: Option<String>,

    #[arg(long, default_value = "5")]
    pub timeout: u64,

    #[arg(long)]
    pub watch: Option<u64>,

    #[arg(long)]
    pub no_cache: bool,

    #[arg(long)]
    pub verbose: bool,
}

#[derive(Clone, ValueEnum)]
pub enum Format {
    Json,
    Csv,
    Env,
    Plain,
}

#[derive(Clone, ValueEnum)]
pub enum Provider {
    Auto,
    Corelocation,
    Geoclue,
    Ip,
}
