use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Args {
    /// Postgres URL
    #[arg(
        short,
        long,
        env = "CHAMELEON_POSTGRES_URL",
        default_value = "postgres://postgres:password@localhost/chameleon"
    )]
    pub postgres_url: String,
}

impl Args {
    pub fn parse() -> Self {
        Parser::parse()
    }
}
