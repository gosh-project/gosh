// bin/gulp-adaptor.rs
// :PROPERTIES:
// :header-args: :tangle src/bin/gulp-adaptor.rs
// :END:

// [[file:~/Workspace/Programming/gosh-rs/gosh/gosh.note::*bin/gulp-adaptor.rs][bin/gulp-adaptor.rs:1]]
#[macro_use]
extern crate duct;

extern crate gchemol;
extern crate gosh;

use std::path::PathBuf;

use gchemol::{io, prelude::*, Molecule};
use gosh::cmd_utils::*;

use gosh::adaptors::*;
use gosh::models::*;

/// Read GULP calculated results, format them as standard external model results.
#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(flatten)]
    verbosity: Verbosity,

    /// GULP generated output file
    #[structopt(parse(from_os_str))]
    outfile: PathBuf,

    /// Parse all result entries found in the output
    #[structopt(short = "a", long = "all")]
    all: bool,
}

fn main() -> Result<()> {
    let args = Cli::from_args();
    args.verbosity.setup_env_logger(&env!("CARGO_PKG_NAME"))?;

    // 1. read SIESTA output
    let outfile = &args.outfile;

    let app = Gulp();
    if args.all {
        for d in app.parse_all(&outfile)? {
            if d.is_empty() {
                bail!("No data extracted from: {:?}", outfile);
            }
            println!("{:}", d);
        }
    } else {
        let d = app.parse_last(&outfile)?;
        if d.is_empty() {
            bail!("No data extracted from: {:?}", outfile);
        }
        println!("{:}", d);
    }

    Ok(())
}
// bin/gulp-adaptor.rs:1 ends here