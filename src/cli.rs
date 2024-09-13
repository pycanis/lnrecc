use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long)]
    pub config_path: Option<String>,

    #[arg(short, long)]
    pub log_path: Option<String>,
    // #[command(subcommand)]
    // command: Commands,
}

// #[derive(Subcommand, Debug)]
// enum Commands {
//     New {
//         #[arg(short, long)]
//         name: Option<String>,

//         #[arg(short, long)]
//         schedule: String,
//     },
// }
