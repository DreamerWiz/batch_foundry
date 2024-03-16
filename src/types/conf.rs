use serde::{Serialize, Deserialize};
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct SubCommandParam{
    #[serde(rename = "NAME")]
    pub name: String,
    #[serde(rename = "SHORT_NAME")]
    pub short_name: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct SubCommand{
    #[serde(rename = "COMMAND_NAME")]
    pub command_name: String,
    #[serde[rename = "ABOUT"]]
    pub about: String,
    #[serde[rename = "ARGS"]]
    pub args: Vec<SubCommandParam>
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Conf{
    #[serde(rename = "COMMAND_NAME")]
    pub command_name: String,

    #[serde(rename = "VERSION")]
    pub version: String,

    #[serde(rename = "AUTHOR")]
    pub author: String,

    #[serde[rename = "ABOUT"]]
    pub about: String,

    #[serde[rename ="SUBCOMMAND_REQUIRED"]]
    pub subcommand_required: bool,

    #[serde[rename="ARG_REQUIRED_ELSE_HELP"]]
    pub arg_required_else_help: bool,

    #[serde[rename="SUB_COMMANDS"]]
    pub sub_commands: Vec<SubCommand>
}