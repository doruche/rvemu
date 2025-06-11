#![allow(unused)]

use clap::{Args, Parser, Subcommand, ValueEnum};
use rvemu_core::{elf, emulator::Emulator, syscall, Error, InsnSet, Result};
use std::{collections::HashSet, hash::Hash, io::Read, path::PathBuf};

#[derive(Parser, Debug)]
#[command(name = "rvemu", author = "doruche", version = "0.1.0",
    about = "A userland RISC-V emulator", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(short, long)]
    verbose : bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run a RISC-V elf
    Run(RunArgs),
}

#[derive(Args, Debug)]
pub struct RunArgs {
    /// Path to the RISC-V elf file
    path: PathBuf,
   /// ISA to use (I, M, A, F, D, C)
    #[arg(short, long, default_value = "I")]
    isa: Option<String>,
    /// Syscall implementation to use
    #[arg(short, long, value_enum, default_value = "glibc")]
    syscall: Syscall,    
    /// Stack size in kb (default: 8 MiB)
    #[arg(long, default_value = "8192")]
    stack_size: usize, 
    /// Arguments to pass to the program
    args: Option<Vec<String>>,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Syscall {
    Glibc,
    Newlib,
    Minilib,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    rvemu_core::log::log_init(rvemu_core::log::Level::Debug);

    match cli.command {
        Commands::Run(args) => cmd_run(args),
    }
}

fn cmd_run(args: RunArgs) -> Result<()> {
    let path = args.path;
    let path_str = path.to_string_lossy().to_string();
    
    let insn_sets: HashSet<InsnSet> = match args.isa {
        Some(isas) => {
            let mut sets = HashSet::new();
            for isa in isas.chars() {
                match InsnSet::from_str(&isa.to_string()) {
                    Some(set) => {
                        sets.insert(set);
                    },
                    None => return Err(Error::Unimplemented),
                }
            }
            sets
        },
        None => {
            vec![InsnSet::I].into_iter().collect()
        }
    };

    let stack_size = args.stack_size * 1024;
    let syscall = match args.syscall {
        Syscall::Glibc => return Err(Error::Unimplemented),
        Syscall::Newlib => return Err(Error::Unimplemented),
        Syscall::Minilib => Box::new(syscall::Minilib),
    };
    let args = args.args.unwrap_or_default();

    let mut builder = Emulator::new();

    for isa in insn_sets {
        builder = builder.decoder(isa);
    }
    builder = builder.syscall(syscall).stack_size(stack_size);
    let mut emulator = builder.build()?;
    
    let mut file = std::fs::File::open(path)
        .map_err(|e| Error::IoError(e))?;

    let mut elf_data = Vec::new();
    file.read_to_end(&mut elf_data)
        .map_err(|e| Error::IoError(e))?;

    emulator.load_elf(&elf_data)?;

    match emulator.run() {
        Ok(exit_code) => {
            println!("[rvemu] program exited with code {}", exit_code);
            Ok(())
        }
        Err(e) => {
            eprintln!("[rvemu] program exited with error: {}", e);
            Err(e)
        }
    }
}