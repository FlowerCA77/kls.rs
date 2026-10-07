use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use inkwell::OptimizationLevel;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Cli {
    pub files: Vec<PathBuf>,

    #[arg(short, long)]
    pub output: Option<PathBuf>,

    #[arg(long, value_enum, default_value_t = EmitKind::Executable)]
    pub emit: EmitKind,

    #[arg(short = 'O', long, default_value_t = 2)]
    pub optimize: u8,

    #[arg(long)]
    pub fast_math: bool,

    #[arg(long)]
    pub ast: bool,

    #[arg(long)]
    pub ir: bool,

    #[arg(long)]
    pub dry_run: bool,
}

impl Cli {
    pub fn optimization_level(&self) -> OptimizationLevel {
        match self.optimize {
            0 => OptimizationLevel::None,
            1 => OptimizationLevel::Less,
            2 => OptimizationLevel::Default,
            3 => OptimizationLevel::Aggressive,
            _ => panic!("optimization level must be 0-3"),
        }
    }

    pub fn optimization_level_string(opt_level: OptimizationLevel) -> String {
        match opt_level {
            OptimizationLevel::None => "default<O0>",
            OptimizationLevel::Less => "default<O1>",
            OptimizationLevel::Default => "default<O2>",
            OptimizationLevel::Aggressive => "default<O3>",
        }
        .into()
    }
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmitKind {
    Executable,
    Object,
    Asm,
    LlvmIr,
}
