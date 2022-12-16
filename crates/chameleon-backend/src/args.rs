use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Args {
    /// Redis URL
    #[arg(
        short,
        long,
        env = "CHAMELEON_REDIS_URL",
        default_value = "redis://localhost:6379"
    )]
    pub redis_url: String,
}

impl Args {
    pub fn parse() -> Self {
        Parser::parse()
    }
}
