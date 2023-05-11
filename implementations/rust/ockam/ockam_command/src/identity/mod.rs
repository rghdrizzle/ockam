mod create;
mod default;
mod delete;
mod list;
mod show;

use colorful::Colorful;
pub(crate) use create::CreateCommand;
pub(crate) use delete::DeleteCommand;
pub(crate) use list::ListCommand;
use ockam_api::cli_state::CliState;
pub(crate) use show::ShowCommand;

use crate::terminal::OckamColor;
use crate::util::OckamConfig;
use crate::{docs, fmt_log, fmt_ok, CommandGlobalOpts, GlobalArgs, Result, PARSER_LOGS};
use crate::{error::Error, identity::default::DefaultCommand};
use clap::{Args, Subcommand};
use ockam_api::cli_state::traits::StateDirTrait;

const LONG_ABOUT: &str = include_str!("./static/long_about.txt");
const AFTER_LONG_HELP: &str = include_str!("./static/after_long_help.txt");

/// Manage identities
#[derive(Clone, Debug, Args)]
#[command(
arg_required_else_help = true,
subcommand_required = true,
long_about = docs::about(LONG_ABOUT),
after_long_help = docs::after_help(AFTER_LONG_HELP)
)]
pub struct IdentityCommand {
    #[command(subcommand)]
    subcommand: IdentitySubcommand,
}

#[derive(Clone, Debug, Subcommand)]
pub enum IdentitySubcommand {
    Create(CreateCommand),
    Show(ShowCommand),
    List(ListCommand),
    Default(DefaultCommand),
    Delete(DeleteCommand),
}

impl IdentityCommand {
    pub fn run(self, options: CommandGlobalOpts) {
        match self.subcommand {
            IdentitySubcommand::Create(c) => c.run(options),
            IdentitySubcommand::Show(c) => c.run(options),
            IdentitySubcommand::List(c) => c.run(options),
            IdentitySubcommand::Delete(c) => c.run(options),
            IdentitySubcommand::Default(c) => c.run(options),
        }
    }
}

pub fn default_identity_name() -> String {
    let res_cli = CliState::try_default();

    let cli_state = match res_cli {
        Ok(cli_state) => cli_state,
        Err(err) => {
            eprintln!("Error initializing command state. \n\n {err:?}");
            let command_err: Error = err.into();
            std::process::exit(command_err.code());
        }
    };

    cli_state
        .identities
        .default()
        .map_or("default".to_string(), |i| i.name().to_string())
}

pub fn identity_name_parser(identity_name: &str) -> Result<String> {
    if identity_name == "default"
        && CliState::try_default()
            .unwrap()
            .identities
            .default()
            .is_err()
    {
        return Ok(create_default_identity(identity_name));
    }

    Ok(identity_name.to_string())
}

pub fn create_default_identity(identity_name: &str) -> String {
    let config = OckamConfig::load().expect("Failed to load config");
    let quiet_opts = CommandGlobalOpts::new(
        GlobalArgs {
            quiet: true,
            ..Default::default()
        },
        config,
    );

    let create_command = CreateCommand::new(identity_name.into(), None);
    create_command.run(quiet_opts);

    if let Ok(mut logs) = PARSER_LOGS.lock() {
        logs.push(fmt_log!("No default identity was found."));
        logs.push(fmt_ok!(
            "Creating default identity {}",
            identity_name
                .to_string()
                .color(OckamColor::PrimaryResource.color())
        ));
        logs.push(fmt_log!(
            "Setting identity {} as default for local operations...\n",
            identity_name
                .to_string()
                .color(OckamColor::PrimaryResource.color())
        ));
    }

    identity_name.to_string()
}
