use super::GitConfigOpts;
use crate::git::{
    commands::{immutable::ImmutableCommands, mutable::MutableCommands},
    hooks::pre_commit::PreCommitHook,
    GitCommandResult, GitResult,
};
use clap::Subcommand;

#[derive(Subcommand, Debug, Clone, Copy)]
pub(crate) enum HookSubcommands {
    /// `pre-commit` hook
    PreCommit {},
}

/// Specify which files to operate a command against
#[derive(Subcommand, Debug, Clone, Copy)]
pub(crate) enum WhichFiles {
    All,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Subcommands {
    /// Wrapper around `git-add`.
    ///
    /// - If called without `args`, it adds all unstaged files IFF the staging area is empty.
    /// - If called with args, it passes through to `git-add`.
    #[command(allow_hyphen_values = true)]
    A {
        /// Command arguments
        args: Option<Vec<String>>,
    },
    /// Add updated and untracked files.
    ///
    /// Fails if the staging area is not empty before attempting to add files.
    #[command(allow_hyphen_values = true)]
    Aa {},
    /// Add updated and untracked files and then commit.
    ///
    /// Fails if the staging area is not empty before attempting to add files.
    #[command(allow_hyphen_values = true)]
    Aac {
        /// Command arguments
        args: Vec<String>,
    },
    /// Stage updated and untracked files and amend the previous commit.
    ///
    /// Fails if the staging area is not empty when subcommand is run.
    #[clap(alias = "aam")]
    Aamend {},
    /// List configured aliases
    Alias {
        /// text to filter on
        filter: Option<String>,

        #[clap(flatten)]
        options: GitConfigOpts,
    },
    /// Commit updated files.
    ///
    /// Fails if the staging area is not empty when subcommand is run.
    #[clap(alias = "ac")]
    #[command(allow_hyphen_values = true)]
    Auc {
        /// Command arguments
        args: Vec<String>,
    },
    /// Stage updated files and amend the previous commit.
    ///
    /// Fails if the staging area is not empty when subcommand is run.
    #[clap(alias = "aum")]
    Aumend {},
    /// Reset author to current value of `user.author` and `user.email` for the last n commits.
    Author {
        /// Number of commits to reset (else defaults to 1)
        num: Option<u16>,
    },
    /// List config settings (excluding aliases).
    Conf {
        /// The text to filter on
        filter: Option<String>,

        #[clap(flatten)]
        options: GitConfigOpts,
    },
    /// Call a git hook.
    Hook {
        // The hook to call
        #[command(subcommand)]
        hook: HookSubcommands,
    },
    /// List the files that changed in the last n commits.
    #[clap(alias = "shf")]
    Files {
        /// The number of commits to list files for (else defaults to 1)
        num: Option<u16>,
    },
    /// Wrapper around `git-log`, formatted to 1 line per commit.
    #[command(allow_hyphen_values = true)]
    L {
        /// The number of commits to list (else defaults to 25)
        num: Option<u16>,

        /// Command arguments
        args: Vec<String>,
    },
    /// List commit message and of changed files for the last n commits; wrapper around `git-log --compact-summary`.
    #[clap(alias = "la")]
    #[command(allow_hyphen_values = true)]
    Last {
        /// The number of commits to list (else defaults to 10)
        num: Option<u16>,

        /// Command arguments
        args: Vec<String>,
    },
    /// Wrapper around `git-restore`.
    #[clap(alias = "rest")]
    #[command(allow_hyphen_values = true)]
    Restore {
        /// Which files to operate on
        #[command(subcommand)]
        which: Option<WhichFiles>,

        /// Command arguments
        args: Vec<String>,
    },
    /// Wrapper around `git-show`.
    #[command(allow_hyphen_values = true)]
    #[clap(alias = "sh")]
    Show {
        /// The number of commits to show (else defaults to 1)
        num: Option<u16>,

        /// Command arguments
        args: Vec<String>,
    },
    /// Reset the last n commits and keep the undone changes in working directory.
    Undo {
        /// The number of commits to undo (else defaults to 1)
        num: Option<u16>,
    },
    /// Move staged files back to staging area; wrapper around `git-restore --staged`.
    #[clap(alias = "u")]
    #[command(allow_hyphen_values = true)]
    Unstage {
        /// which files to operate on
        #[command(subcommand)]
        which: Option<WhichFiles>,

        /// Command arguments
        args: Vec<String>,
    },
    /// Update the specified local branch from origin without checking it out.
    #[clap(alias = "unwind")]
    #[command(allow_hyphen_values = true)]
    Update {
        /// Command arguments
        branch: String,
    },
}

impl Subcommands {
    pub(crate) fn run(&self) -> Result<GitCommandResult, anyhow::Error> {
        match self {
            Subcommands::A { args } => match args {
                Some(args) => MutableCommands::add(args),
                None => MutableCommands::add_updated(),
            },
            Subcommands::Aa {} => MutableCommands::add_all(),
            Subcommands::Aac { args } => MutableCommands::commit_all(args),
            Subcommands::Aamend {} => MutableCommands::commit_all_amended(),
            Subcommands::Alias { filter, options } => ImmutableCommands::list_aliases(
                filter.as_deref(),
                crate::git::GitConfigOpts {
                    show_origin: options.show_origin,
                    show_scope: options.show_scope,
                },
            ),
            Subcommands::Auc { args } => MutableCommands::commit_all_updated_files(args),
            Subcommands::Aumend {} => MutableCommands::commit_all_updated_files_amended(),
            Subcommands::Author { num } => MutableCommands::update_commit_author(*num),
            Subcommands::Conf { filter, options } => {
                ImmutableCommands::list_configuration_settings(
                    filter.as_deref(),
                    crate::git::GitConfigOpts {
                        show_origin: options.show_origin,
                        show_scope: options.show_scope,
                    },
                )
            }
            Subcommands::Hook { hook } => hook.run(),
            Subcommands::Files { num } => ImmutableCommands::show_files(*num),
            Subcommands::L { num, args } => ImmutableCommands::one_line_log(*num, args),
            Subcommands::Last { num, args } => ImmutableCommands::compact_summary_log(*num, args),
            Subcommands::Show { num, args } => ImmutableCommands::show(*num, args),
            Subcommands::Restore { which, args } => {
                if let Some(all) = which {
                    match all {
                        WhichFiles::All => MutableCommands::restore_all(),
                    }
                } else {
                    MutableCommands::restore(args)
                }
            }
            Subcommands::Undo { num } => MutableCommands::undo_commits(*num),
            Subcommands::Unstage { which, args } => {
                if let Some(which) = which {
                    match which {
                        WhichFiles::All => MutableCommands::unstage_all(),
                    }
                } else {
                    MutableCommands::unstage(args)
                }
            }
            Subcommands::Update { branch } => MutableCommands::update_branch_from_remote(branch),
        }
    }
}

impl HookSubcommands {
    pub(crate) fn run(&self) -> GitResult {
        match self {
            HookSubcommands::PreCommit {} => PreCommitHook::run(),
        }
    }
}
