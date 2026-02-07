use clap::{ArgGroup, Parser};

#[derive(Debug, Parser)]
#[command(name = "wayland-info-rs")]
#[command(about = "Wayland protocol information dumper", long_about = None)]
#[command(group = ArgGroup::new("detail").args(["full", "simple"]).multiple(false))]
pub struct Cli {
    /// Output JSON
    #[arg(long)]
    pub json: bool,

    /// Include detailed protocol data (default)
    #[arg(long)]
    pub full: bool,

    /// Hide detailed protocol data
    #[arg(long)]
    pub simple: bool,

    /// Sort globals by interface (omit name field)
    #[arg(long)]
    pub sort: bool,

    /// Only show matching protocol
    #[arg(short = 'p', long = "protocol")]
    pub protocol: Option<String>,
}

pub struct CliOptions {
    pub json_output: bool,
    pub full_output: bool,
    pub sort_output: bool,
    pub protocol_filter: Option<String>,
}

pub fn parse_args() -> CliOptions {
    let cli = Cli::parse();
    let full_output = !cli.simple;

    CliOptions {
        json_output: cli.json,
        full_output,
        sort_output: cli.sort,
        protocol_filter: cli.protocol,
    }
}
