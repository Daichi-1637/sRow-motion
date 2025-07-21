use crate::config::Config;
use shared::error::AppResult;

pub mod arg_config_builder;
pub mod json_config_builder;

pub trait ConfigBuilder {
    fn build(&self) -> AppResult<Config>;
}
