use crate::git::{GitCommandResult, GitResult};
use crate::{
    commands::ripgrep::{Ripgrep, RipgrepOptions},
    git::{Git, GitCommand},
};
use crate::{commands::Commands, git::GitConfigOpts};
use anyhow::Context;
use log::trace;
use std::{
    io::{self, Write},
    process::{ChildStdout, Output},
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct ImmutableCommands();

impl ImmutableCommands {
    /// `git log --compact-summary --max-count=NUM ARGS`
    pub fn compact_summary_log(num: Option<u8>, args: &[String]) -> GitResult {
        trace!("last() called with: {:#?}, {:#?}", num, args);

        GitCommand {
            subcommand: "log",
            default_args: &[
                "--compact-summary",
                &format!("--max-count={}", num.unwrap_or(1)),
            ],
            user_args: args,
        }
        .execute_git_command()
    }

    /// list configured aliases, optionally filtering on those containing `filter`
    pub fn list_aliases(filter: Option<&str>, options: GitConfigOpts) -> GitResult {
        trace!("alias() called with: {:#?}", filter);

        let mut config_args = vec!["config"];

        Git::parse_config_options(options, &mut config_args);

        // this arg has to be last
        config_args.push("--get-regexp");
        config_args.push(r"^alias\.");

        // get Git config values that start with "alias."
        let aliases = Commands::pipe_from_command("git", &config_args)?;

        // strip out the initial "alias." from the config name
        let aliases = Commands::double_ended_pipe("sed", aliases, &[r"s/^alias\.//"])?;

        let filtered_aliases: ChildStdout = match filter {
            Some(pattern) => {
                // filter on `pattern`
                Ripgrep::double_ended_pipe(aliases, pattern, Some(&[RipgrepOptions::FixedStrings]))?
            }
            None => aliases,
        };

        // replace the first space (which separates the alias name and value) with a semicolon
        let delimited_aliases =
            Commands::double_ended_pipe("sed", filtered_aliases, &[r"s/ /\;/"])?;

        let aliases_table: Output = Commands::pipe_to_column(delimited_aliases, ';')?;

        io::stdout()
            .write_all(&aliases_table.stdout)
            .with_context(|| "Failed to write column output to stdout")?;

        Ok(GitCommandResult::Success)
    }

    /// list configuration settings (excluding aliases), optionally filtering on those containing `filter`
    pub fn list_configuration_settings(filter: Option<&str>, options: GitConfigOpts) -> GitResult {
        trace!("conf() called with: {:#?}", filter);

        let mut config_args = vec!["config", "--list"];

        Git::parse_config_options(options, &mut config_args);

        // get Git config values that start with "alias."
        let configs = Commands::pipe_from_command("git", &config_args)?;

        // filter out config entries that start with "alias."
        // `rg -v ^alias\.`
        let configs_no_aliases =
            Ripgrep::double_ended_pipe(configs, r"^alias\.", Some(&[RipgrepOptions::InvertMatch]))?;

        let filtered_configs: ChildStdout = match filter {
            Some(pattern) => {
                // filter on `pattern`
                Ripgrep::double_ended_pipe(
                    configs_no_aliases,
                    pattern,
                    Some(&[RipgrepOptions::FixedStrings]),
                )?
            }
            None => configs_no_aliases,
        };

        let config_table: Output = Commands::pipe_to_column(filtered_configs, '=')?;

        io::stdout()
            .write_all(&config_table.stdout)
            .with_context(|| "Failed to write column output to stdout")?;

        Ok(GitCommandResult::Success)
    }

    /// `git log --pretty='%C(yellow)%h %C(magenta)%as %C(blue)%aL %C(cyan)%s%C(reset)' --max-count=NUM ARGS`
    pub fn one_line_log(num: Option<u8>, args: &[String]) -> GitResult {
        trace!("log_oneline() called with: {:#?}", num);

        GitCommand {
            subcommand: "log",
            default_args: &[
                "--pretty='%C(yellow)%h %C(magenta)%as %C(blue)%aL %C(cyan)%s%C(reset)'",
                &format!("--max-count={}", num.unwrap_or(25)),
            ],
            user_args: args,
        }
        .execute_git_command()
    }

    /// `git show --expand-tabs=4 --max-count=NUM ARGS`
    pub fn show(num: Option<u8>, args: &[String]) -> GitResult {
        trace!("show() called with: {:#?}", num);

        GitCommand {
            subcommand: "show",
            default_args: &[
                "--expand-tabs=4",
                &format!("--max-count={}", num.unwrap_or(1)),
            ],
            user_args: args,
        }
        .execute_git_command()
    }

    /// `git show --pretty='' --name-only --max-count=NUM`
    pub fn show_files(num: Option<u8>) -> GitResult {
        trace!("show_files() called with: {:#?}", num);

        GitCommand {
            subcommand: "show",
            default_args: &[
                "--pretty=",
                "--name-only",
                &format!("--max-count={}", num.unwrap_or(1)),
            ],
            user_args: &[],
        }
        .execute_git_command()
    }
}
