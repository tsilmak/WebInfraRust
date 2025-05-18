use clap::Parser;

#[derive(Parser, Debug)]
pub struct Config {
    #[arg(long, default_value = "3000")]
    pub port: u16,
}

pub fn load() -> Config {
    Config::parse()
}
