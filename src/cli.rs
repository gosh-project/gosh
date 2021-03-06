// imports

// [[file:~/Workspace/Programming/gosh-rs/gosh/gosh.note::*imports][imports:1]]
use crate::cmd_utils::*;
use crate::core::*;

use gchemol::prelude::*;
use gchemol::{io, Molecule};

use std::path::PathBuf;
use std::process::Command;
// imports:1 ends here

// base

// [[file:~/Workspace/Programming/gosh-rs/gosh/gosh.note::*base][base:1]]
/// A commander for interactive interpreter
pub struct Commander {
    /// active molecules
    pub molecules: Vec<Molecule>,
    /// input file containg molecules
    pub filename: Option<PathBuf>,
}

#[derive(StructOpt, Debug)]
#[structopt(setting = structopt::clap::AppSettings::VersionlessSubcommands)]
pub enum GoshCmd {
    /// Quit go shell.
    #[structopt(name = "quit", alias = "q", alias = "exit")]
    Quit {},

    /// Show available commands.
    #[structopt(name = "help", alias = "h", alias = "?")]
    Help {},

    /// Write molecule(s) into file.
    #[structopt(name = "write", alias = "save")]
    Write {
        /// The filename to write.
        #[structopt(name = "FILE_NAME", parse(from_os_str))]
        filename: Option<PathBuf>,
    },

    /// Load molecule(s) from file.
    #[structopt(name = "load")]
    Load {
        /// The filename containing one or more molecules.
        #[structopt(name = "MOLECULE_NAME", parse(from_os_str))]
        filename: PathBuf,
    },

    /// Rebuild bonds based on atom distances.
    #[structopt(name = "rebond")]
    Rebond {},

    /// Clean up bad molecular geometry.
    #[structopt(name = "clean")]
    Clean {},

    /// Convert molecule formats in batch. e.g.: convert *.xyz .mol2
    #[structopt(name = "convert")]
    Convert {
        /// input files: e.g.: *
        files: Vec<PathBuf>,
        /// target format: e.g.: .mol2
        format_to: String,
    },

    /// Format molecule using template file.
    #[structopt(name = "format")]
    Format {
        /// Path to template file.
        #[structopt(name = "TEMPLATE_NAME", parse(from_os_str))]
        filename: PathBuf,
    },

    /// Break molecule into smaller fragments based on connectivity.
    #[structopt(name = "fragment")]
    Fragment {},

    /// Create supercell for all loaded molecules.
    #[structopt(name = "supercell")]
    Supercell {
        /// range a
        range_a: isize,
        /// range b
        range_b: isize,
        /// range c
        range_c: isize,
    },

    /// Superimpose current molecule onto reference molecule by translating and
    /// rotating target molecule
    #[structopt(name = "superimpose")]
    Superimpose {
        /// Path to reference molecule file.
        #[structopt(name = "REFERENCE_MOLECULE", parse(from_os_str))]
        filename: PathBuf,
    },

    /// Show supported file formats.
    #[structopt(name = "avail")]
    Avail {},

    /// List files under current directory.
    #[structopt(name = "ls", alias = "l", alias = "ll")]
    List {},

    /// Print path to current directory.
    #[structopt(name = "pwd")]
    Pwd {},
}
// base:1 ends here

// core

// [[file:~/Workspace/Programming/gosh-rs/gosh/gosh.note::*core][core:1]]
impl Commander {
    pub fn new() -> Self {
        Commander {
            filename: None,
            molecules: vec![],
        }
    }

    pub fn action(&mut self, cmd: &GoshCmd) -> CliResult {
        match cmd {
            GoshCmd::Quit {} | GoshCmd::Help {} => {
                //
            }
            GoshCmd::List {} => {
                if let Err(ref e) = run_cmd("ls") {
                    eprintln!("{:?}", e);
                }
            }
            GoshCmd::Pwd {} => {
                if let Err(ref e) = run_cmd("pwd") {
                    eprintln!("{:?}", e);
                }
            }
            GoshCmd::Load { filename } => {
                self.molecules = io::read(filename)?;
                self.filename = Some(filename.to_owned());

                println!("Loaded {} molecule(s).", self.molecules.len());
            }

            GoshCmd::Clean {} => {
                self.check()?;
                self.molecules[0].clean()?;
            }

            GoshCmd::Avail {} => {
                gchemol::io::describe_backends();
            }

            GoshCmd::Write { filename } => {
                self.check()?;

                if let Some(filename) = filename.as_ref().or(self.filename.as_ref()) {
                    io::write(&filename, &self.molecules)?;
                    println!(
                        "Wrote {} molecules in {}",
                        self.molecules.len(),
                        filename.display()
                    );
                } else {
                    eprintln!("No filename.");
                }
            }

            GoshCmd::Fragment {} => {
                self.check()?;
                let mols = self.molecules[0].fragment();
                self.molecules.clear();
                self.molecules.extend(mols);
            }
            GoshCmd::Rebond {} => {
                self.check()?;
                for mol in self.molecules.iter_mut() {
                    mol.rebond();
                }
            }
            GoshCmd::Supercell {
                range_a,
                range_b,
                range_c,
            } => {
                use gchemol::Supercell;

                self.check()?;

                let mut mols = vec![];
                for mol in self.molecules.iter() {
                    if mol.lattice.is_some() {
                        let mol = Supercell::new()
                            .with_range_a(0, *range_a)
                            .with_range_b(0, *range_b)
                            .with_range_c(0, *range_c)
                            .build(&mol);
                        mols.push(mol);
                    } else {
                        eprintln!("No lattice data.");
                    }
                }
                self.molecules = mols;
            }
            GoshCmd::Superimpose { filename } => {
                self.check()?;
            }
            GoshCmd::Format { filename } => {
                self.check()?;

                for mol in &self.molecules {
                    let template = io::read_file(&filename).map_err(|e| {
                        error!("failed to load template");
                        e
                    })?;
                    let s = mol.render_with(&template).map_err(|e| {
                        error!("failed to render molecule");
                        e
                    })?;
                    println!("{:}", s);
                }
            }
            o => {
                eprintln!("{:?}: not implemented yet!", o);
            }
        }

        Ok(())
    }

    /// basic sanity check
    fn check(&self) -> Result<()> {
        if self.molecules.is_empty() {
            bail!("No active molecule available.")
        }

        Ok(())
    }
}

fn run_cmd(cmdline: &str) -> CliResult {
    let output = std::process::Command::new(cmdline)
        .output()
        .map_err(|_| format_err!("external cmdline failed: {}", cmdline))?;

    if output.status.success() {
        println!("{}", String::from_utf8_lossy(&output.stdout));
    } else {
        println!("{}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}
// core:1 ends here
