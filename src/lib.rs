mod load;
mod prelude;

// Exported
pub use crate::load::download;

use clap::{Parser, arg};

#[derive(Parser)]
pub struct Args {
    /// Channel version to build
    #[arg(short, long)]
    pub ver: String,

    /// Source directory
    #[arg(short, long)]
    pub src: String,
}
