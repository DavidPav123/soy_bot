use std::{error::Error, fmt};

pub use sc2_proc_macro::{FromStr, bot, bot_new, variant_checkers};

#[derive(Debug, PartialEq, Eq)]
pub struct ParseEnumError;

impl fmt::Display for ParseEnumError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to parse enum")
    }
}

impl Error for ParseEnumError {}
