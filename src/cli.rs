use crate::{
    browser::{Browser, Config},
    monitor::{self, Clock, RealClock},
    relay,
    transport::Process,
};
use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::{Arc, atomic::AtomicBool},
    time::Instant,
};
#[derive(Parser)]
#[command(
    version,
    about = "Higgsfield local checks and monitoring; never submits generation"
)]
pub struct Cli {
    #[command(subcommand)]
    pub action: Action,
}
#[derive(Args)]
pub struct WatchArgs {
    #[arg(long, default_value_t = 0.0)]
    pub relay_wait: f64,
    #[arg(long)]
    pub config: PathBuf,
    #[arg(long)]
    pub asset_id: String,
    #[arg(long)]
    pub state_dir: PathBuf,
    #[arg(long, default_value_t = 20.0)]
    pub interval: f64,
    #[arg(long, default_value_t = 3600.0)]
    pub timeout: f64,
    #[arg(long)]
    pub resume: bool,
    #[arg(long, default_value = "hc")]
    pub hc: String,
    #[arg(long, requires = "notify_app")]
    pub notify_terminal: Option<String>,
    #[arg(long,requires="notify_terminal",value_parser=["codex","copilot"])]
    pub notify_app: Option<String>,
}
#[derive(Subcommand)]
pub enum Action {
    Supervise {
        #[command(flatten)]
        watch: WatchArgs,
        #[arg(long, default_value_t = 2)]
        max_restarts: u32,
        #[arg(long, default_value_t = 5.0)]
        backoff: f64,
    },
    Inspect {
        #[arg(long)]
        config: PathBuf,
    },
    EnsureUnlimited {
        #[arg(long)]
        config: PathBuf,
    },
    SetPrompt {
        #[arg(long)]
        config: PathBuf,
        #[arg(long)]
        file: PathBuf,
    },
    Watch(WatchArgs),
    Relay {
        #[arg(long, default_value_t = 0.0)]
        relay_wait: f64,
        #[arg(long)]
        state_dir: PathBuf,
        #[arg(long, default_value = "hc")]
        hc: String,
        #[arg(long)]
        notify_terminal: String,
        #[arg(long,value_parser=["codex","copilot"])]
        notify_app: String,
    },
    Demo {
        #[arg(long, default_value = "local/demo")]
        state_dir: PathBuf,
    },
}
fn config(path: PathBuf) -> Result<Config> {
    Ok(serde_json::from_slice(&std::fs::read(path)?)?)
}
pub fn run(cli: Cli, cancel: &AtomicBool) -> Result<Value> {
    match cli.action {
        Action::Supervise {
            watch,
            max_restarts,
            backoff,
        } => crate::supervisor::supervise(watch, max_restarts, backoff, cancel),
        Action::Inspect { config: path } => Browser {
            config: config(path)?,
            runner: &Process,
        }
        .inspect(),
        Action::EnsureUnlimited { config: path } => Browser {
            config: config(path)?,
            runner: &Process,
        }
        .ensure(),
        Action::SetPrompt { config: path, file } => Browser {
            config: config(path)?,
            runner: &Process,
        }
        .replace_prompt(&std::fs::read_to_string(file)?),
        Action::Watch(a) => {
            let b = Browser {
                config: config(a.config)?,
                runner: &Process,
            };
            let mut clock = RealClock {
                start: Instant::now(),
                cancel,
            };
            let event = monitor::watch(
                |id| b.observe(id),
                &a.asset_id,
                &a.state_dir,
                a.interval,
                a.timeout,
                a.resume,
                &mut clock,
            )?;
            if let (Some(target), Some(app)) = (a.notify_terminal, a.notify_app) {
                Ok(
                    json!({"event":event,"relay":relay::deferred(&a.state_dir,&a.hc,&target,&app,&Process,a.relay_wait)?}),
                )
            } else {
                Ok(event)
            }
        }
        Action::Relay {
            relay_wait,
            state_dir,
            hc,
            notify_terminal,
            notify_app,
        } => relay::deferred(
            &state_dir,
            &hc,
            &notify_terminal,
            &notify_app,
            &Process,
            relay_wait,
        ),
        Action::Demo { state_dir } => {
            struct Fast(f64);
            impl Clock for Fast {
                fn now(&self) -> f64 {
                    self.0
                }
                fn sleep(&mut self, s: f64) {
                    self.0 += s;
                }
            }
            let mut states = ["pending", "completed", "completed"].into_iter();
            let event = monitor::watch(
                |id| Ok(json!({"state":states.next().unwrap(),"asset_id":id})),
                "00000000-0000-4000-8000-000000000001",
                &state_dir,
                20.0,
                3600.0,
                false,
                &mut Fast(0.0),
            )?;
            Ok(json!({"synthetic":true,"event":event}))
        }
    }
}
pub fn entry() {
    let cli = Cli::parse();
    let cancel = Arc::new(AtomicBool::new(false));
    let flag = cancel.clone();
    if let Err(_) = ctrlc::set_handler(move || {
        flag.store(true, std::sync::atomic::Ordering::Relaxed);
    }) {
        eprintln!("signal handler unavailable");
        std::process::exit(2);
    }
    match run(cli, &cancel) {
        Ok(v) => println!("{}", serde_json::to_string_pretty(&v).unwrap()),
        Err(e) => {
            eprintln!(
                "{}",
                json!({"error":e.to_string(),"generation_submitted":false})
            );
            std::process::exit(2);
        }
    }
}
