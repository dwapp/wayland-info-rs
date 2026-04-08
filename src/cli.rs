use argh::FromArgs;

#[derive(Debug, FromArgs)]
/// Wayland protocol information dumper
pub struct Cli {
    /// output JSON
    #[argh(switch)]
    pub json: bool,

    /// hide detailed protocol data
    #[argh(switch)]
    pub simple: bool,

    /// sort globals by interface (omit name field)
    #[argh(switch)]
    pub sort: bool,

    /// only show matching protocol
    #[argh(option, short = 'p')]
    pub protocol: Option<String>,

    /// print version information and exit
    #[argh(switch, short = 'v')]
    pub version: bool,
}

pub struct CliOptions {
    pub json_output: bool,
    pub full_output: bool,
    pub sort_output: bool,
    pub protocol_filter: Option<String>,
}

pub fn parse_args() -> CliOptions {
    let cli: Cli = argh::from_env();

    if cli.version {
        println!("wayland-info-rs {}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }

    let full_output = !cli.simple;

    CliOptions {
        json_output: cli.json,
        full_output,
        sort_output: cli.sort,
        protocol_filter: cli.protocol,
    }
}
