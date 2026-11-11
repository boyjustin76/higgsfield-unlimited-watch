use anyhow::Result;
use clap::{Parser, Subcommand};
use huw::{
    browser::{Browser, Config},
    transport::Process,
};
use std::path::PathBuf;
#[derive(Parser)]
#[command(
    version,
    about = "Higgsfield local browser checks; never submits generation"
)]
struct Cli {
    #[command(subcommand)]
    action: Action,
}
#[derive(Subcommand)]
enum Action {
    Inspect {
        #[arg(long)]
        config: PathBuf,
    },
    EnsureUnlimited {
        #[arg(long)]
        config: PathBuf,
    },
}
fn run(cli: Cli) -> Result<serde_json::Value> {
    let (path, ensure) = match cli.action {
        Action::Inspect { config } => (config, false),
        Action::EnsureUnlimited { config } => (config, true),
    };
    let config: Config = serde_json::from_slice(&std::fs::read(path)?)?;
    let b = Browser {
        config,
        runner: &Process,
    };
    if ensure { b.ensure() } else { b.inspect() }
}
fn main() {
    match run(Cli::parse()) {
        Ok(v) => println!("{}", serde_json::to_string_pretty(&v).unwrap()),
        Err(e) => {
            eprintln!(
                "{}",
                serde_json::json!({"error":e.to_string(),"generation_submitted":false})
            );
            std::process::exit(2);
        }
    }
}
