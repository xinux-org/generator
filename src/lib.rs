mod load;
mod prelude;

// Exported
pub use crate::load::download;

use clap::{arg, Parser};

#[derive(Parser)]
pub struct Args {
    /// Channel version to build
    #[arg(short, long)]
    pub ver: String,

    /// Source directory
    #[arg(short, long)]
    pub src: String,
}
