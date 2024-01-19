use std::{collections::HashMap, io, path::PathBuf};

use chrono::DateTime;
use clap::{Args, Parser, Subcommand};
use cmd::{check, run, user};
use enum_as_inner::EnumAsInner;
use handler_impl::{
    group::DefaultGroupHandler, notifications::DefaultNotificationHandler,
    permission::DefaultPermHandler, setting::DefaultSettingHandler, user::DefaultUserHandler,
};
use parking_lot::RwLock;
use sea_orm::DatabaseConnection;
use skynet::{config::Config, logger::Logger, plugin::PluginManager};

mod api;
mod cmd;
mod db;
mod handler_impl;

#[allow(clippy::struct_excessive_bools)]
#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Config file
    #[arg(
        short,
        long,
        global = true,
        value_name = "FILE",
        default_value = "conf.yml"
    )]
    config: PathBuf,

    /// Plugin folder path
    #[arg(
        short,
        long,
        global = true,
        value_name = "PATH",
        default_value = "plugin"
    )]
    plugin: PathBuf,

    /// Show verbose/debug log
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Do not print any log
    #[arg(short, long, global = true)]
    quiet: bool,

    /// Persist previous session when initialized
    #[arg(long, global = true)]
    persist_session: bool,

    /// Use JSON to format log
    #[arg(long, global = true)]
    log_json: bool,
}

#[derive(Subcommand, EnumAsInner, Clone)]
enum Commands {
    /// Run skynet
    Run {
        /// Disable CSRF protection, for debugging purpose only.
        #[arg(long)]
        disable_csrf: bool,
    },
    /// User management
    User(UserCli),
    /// Check config file
    Check,
}

#[derive(Args, Clone)]
struct UserCli {
    #[command(subcommand)]
    command: UserCommands,
}

#[derive(Subcommand, EnumAsInner, Clone)]
enum UserCommands {
    /// Add new user
    Add {
        /// User avatar
        #[arg(short, long, value_name = "FILE")]
        avatar: Option<PathBuf>,

        /// New username
        username: String,
    },

    /// Init root user
    Init {
        /// User avatar
        #[arg(short, long, value_name = "FILE")]
        avatar: Option<PathBuf>,
    },

    /// Reset user
    Reset {
        /// Reset username
        username: String,
    },
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let mut skynet = skynet::Skynet {
        logger: Logger::new(),
        user: Box::new(DefaultUserHandler::new()),
        group: Box::new(DefaultGroupHandler::new()),
        perm: Box::new(DefaultPermHandler::new()),
        notification: Box::new(DefaultNotificationHandler::new()),
        setting: Box::new(DefaultSettingHandler::new()),

        config: Config::new(),
        locale: HashMap::new(),

        db: DatabaseConnection::default(),
        redis: None,

        plugin: PluginManager::new(),
        menu: Vec::new(),

        running: RwLock::new(false),
        start_time: DateTime::default(),
    };
    // init logger first
    skynet
        .logger
        .init(!cli.quiet, cli.log_json, cli.verbose)
        .unwrap();

    match &cli.command {
        Commands::Run { disable_csrf } => Box::pin(run::command(&cli, skynet, *disable_csrf)).await,
        Commands::User(user_cli) => Box::pin(user::command(&cli, skynet, user_cli)).await,
        Commands::Check => check::command(&cli),
    }
    Ok(())
}