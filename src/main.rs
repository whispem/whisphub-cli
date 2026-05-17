use clap::{Parser, Subcommand};
use colored::Colorize;

mod api;
mod commands;
mod config;
mod project;

#[derive(Parser)]
#[command(name = "whisphub", version, about = "WhispHub command-line client")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Log in to your WhispHub account (opens browser).
    Login,
    /// Show the currently logged-in user.
    Whoami,
    /// Log out and remove local credentials.
    Logout,
    /// Initialize a new WhispHub project in the current directory.
    Init {
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        slug: Option<String>,
        #[arg(long)]
        tagline: Option<String>,
    },
    /// Upload the current directory to your WhispHub project.
    Push {
        #[arg(short, long)]
        yes: bool,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Login => commands::login::run().await,
        Commands::Whoami => commands::whoami::run().await,
        Commands::Logout => commands::logout::run().await,
        Commands::Init { title, slug, tagline } => commands::init::run(title, slug, tagline).await,
        Commands::Push { yes } => commands::push::run(yes).await,
    };

    if let Err(e) = result {
        eprintln!("{} {}", "error:".red().bold(), e);
        std::process::exit(1);
    }
}