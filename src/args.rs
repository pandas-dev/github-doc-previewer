use clap::Parser;

/// GitHub documentation previewer for pull requests
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    // path to the configuration file
    #[arg(short, long, default_value = "/etc/doc-previewer/config.toml")]
    pub config_file: String
}
