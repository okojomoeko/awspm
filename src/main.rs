use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;

#[derive(Parser)]
#[command(name = "awspm", version)]
#[command(about = "AWS Profile Manager", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize the metadata file
    Init,
    /// Sync AWS profiles to metadata file
    Sync {
        /// Check only, do not modify metadata
        #[arg(long)]
        check: bool,
        /// Automatically yes to prompts
        #[arg(short = 'y', long = "yes")]
        yes: bool,
    },
    /// List profiles
    List {
        /// Filter query. Supports 'tag:TAG', 'alias:ALIAS', 'name:NAME' or simple text matching.
        #[arg(short, long)]
        query: Option<String>,
        /// Compact mode (show only name and aliases)
        #[arg(short, long)]
        short: bool,
    },
    /// Interactive fuzzy search
    Search {
        /// Target field to search: all, tags, aliases, name
        #[arg(long, default_value = "all")]
        target: String,
        /// Initial search query
        #[arg(value_name = "QUERY")]
        query: Option<String>,
    },
    /// Select profile (alias for search)
    Select {
        /// Target field to search: all, tags, aliases, name
        #[arg(long, default_value = "all")]
        target: String,
        /// Initial search query
        #[arg(value_name = "QUERY")]
        query: Option<String>,
    },
    /// Switch to a profile in a new subshell
    Switch {
        /// Profile name (optional). If missing, interactive search is used.
        name: Option<String>,
        /// Automatically yes to prompts
        #[arg(short = 'y', long = "yes")]
        yes: bool,
    },

    /// Pin a profile to the current directory (creates/updates .envrc)
    Pin {
        /// Profile name (optional). If missing, interactive search is used.
        name: Option<String>,
        /// Automatically yes to prompts
        #[arg(short = 'y', long = "yes")]
        yes: bool,
    },
    /// Execute a command with AWS_PROFILE set
    Exec {
        /// Profile name (optional). If missing, interactive search is used.
        name: Option<String>,
        /// Command to execute
        #[arg(last = true)]
        command: Vec<String>,
        /// Automatically yes to prompts
        #[arg(short = 'y', long = "yes")]
        yes: bool,
        /// Preserve current environment variables
        #[arg(long = "preserve-env")]
        preserve_env: bool,
    },
    /// Output environment variables for a profile (e.g. for eval)
    Env {
        /// Profile name (optional). If missing, interactive search is used.
        name: Option<String>,
    },
    /// Get current profile name
    Current,
    /// Update metadata for a profile
    Update {
        /// Profile name
        name: String,
        /// Add a tag
        #[arg(long, value_delimiter = ',')]
        add_tag: Vec<String>,
        /// Remove a tag
        #[arg(long, value_delimiter = ',')]
        remove_tag: Vec<String>,
        /// Add an alias
        #[arg(long, value_delimiter = ',')]
        add_alias: Vec<String>,
        /// Remove an alias
        #[arg(long, value_delimiter = ',')]
        remove_alias: Vec<String>,
        /// Set a note
        #[arg(long)]
        set_note: Option<String>,
        /// Unset the note
        #[arg(long)]
        unset_note: bool,
        /// Set region override
        #[arg(long)]
        set_region: Option<String>,
        /// Unset region override
        #[arg(long)]
        unset_region: bool,
    },
    /// Generate shell completion script
    Completion {
        /// Shell to generate script for
        #[arg(value_enum)]
        shell: Shell,
    },
}

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let level = if cli.verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::WARN
    };

    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_writer(std::io::stderr)
        .init();

    // Check sync status for commands that rely on profile consistency
    match &cli.command {
        Commands::List { .. }
        | Commands::Search { .. }
        | Commands::Select { .. }
        | Commands::Switch { .. }
        | Commands::Pin { .. }
        | Commands::Exec { .. }
        | Commands::Current
        | Commands::Update { .. } => {
            if let Ok(store) = awspm::core::config::Store::new()
                && let Ok(status) = store.check_sync_status()
                && (!status.untracked.is_empty() || !status.orphaned.is_empty())
            {
                eprintln!(
                    "Warning: Metadata is out of sync with AWS sources ({} untracked, {} orphaned).",
                    status.untracked.len(),
                    status.orphaned.len()
                );
                eprintln!("Run `awspm sync` to fix.\n");
            }
        }
        _ => {}
    }

    match cli.command {
        Commands::Init => {
            use awspm::features::init::InitCommand;
            InitCommand::execute()?;
        }
        Commands::Sync { check, yes } => {
            use awspm::features::sync::SyncCommand;
            SyncCommand::execute(check, yes)?;
        }
        Commands::List { query, short } => {
            use awspm::features::list::ListCommand;
            ListCommand::execute(query, short)?;
        }
        Commands::Search { target, query } | Commands::Select { target, query } => {
            use awspm::features::search::{SearchCommand, SearchTarget};
            let search_target = match target.as_str() {
                "tags" => SearchTarget::Tags,
                "aliases" => SearchTarget::Aliases,
                "name" => SearchTarget::Name,
                _ => SearchTarget::All,
            };

            if let Some(selected) = SearchCommand::execute(search_target, query)? {
                println!("{}", selected);
            }
        }
        Commands::Switch { name, yes } => {
            use awspm::features::switch::SwitchCommand;
            SwitchCommand::execute(name, yes)?;
        }
        Commands::Pin { name, yes } => {
            use awspm::features::pin::PinCommand;
            PinCommand::execute(name, yes)?;
        }
        Commands::Exec {
            name,
            command,
            yes,
            preserve_env,
        } => {
            use awspm::features::exec::ExecCommand;
            ExecCommand::execute(name, command, yes, preserve_env)?;
        }
        Commands::Env { name } => {
            use awspm::features::env::EnvCommand;
            EnvCommand::execute(name)?;
        }
        Commands::Current => {
            use awspm::features::current::CurrentCommand;
            CurrentCommand::execute()?;
        }
        Commands::Update {
            name,
            add_tag,
            remove_tag,
            add_alias,
            remove_alias,
            set_note,
            unset_note,
            set_region,
            unset_region,
        } => {
            use awspm::features::update::{UpdateArgs, UpdateCommand};
            UpdateCommand::execute(UpdateArgs {
                profile_name: name,
                add_tags: add_tag,
                remove_tags: remove_tag,
                add_aliases: add_alias,
                remove_aliases: remove_alias, // Corresponds to remove_alias in args
                set_note,
                unset_note,
                set_region,
                unset_region,
            })?;
        }
        Commands::Completion { shell } => {
            use awspm::features::completion::CompletionCommand;
            let mut cmd = Cli::command();
            CompletionCommand::execute(&mut cmd, shell);
        }
    }
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
