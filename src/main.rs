mod models;
mod repo;
mod analyzer;
mod changelog;
mod utils;

use std::path::Path;
use colored::*;
use git2::Repository;
use clap::{Parser, Subcommand};

/// Program to analyze the contribution statistics of each author in a Git repository
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the Git repository
    #[arg(short, long, default_value = ".")]
    path: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate a changelog between two Git tags
    Changelog {
        /// Starting tag
        #[arg(short, long)]
        from_tag: String,
        
        /// Ending tag
        #[arg(short, long)]
        to_tag: String,
        
        /// Output file (optional)
        #[arg(short, long)]
        output: Option<String>,
    }
}

fn main() {
    let args: Args = Args::parse();
    let repo_path: &Path = Path::new(&args.path);

    match Repository::open(repo_path) {
        Ok(repo) => {
            match &args.command {
                Some(Commands::Changelog { from_tag, to_tag, output }) => {
                    changelog::generate_changelog(&repo, from_tag, to_tag, output);
                },
                None => {
                    if repo.is_empty().unwrap_or(true) {
                        println!("{}", "Repository is empty. No commits to analyze.".yellow());
                    } else {
                        let dirname: &str = repo_path.file_name().and_then(|name| name.to_str()).unwrap_or("");
                        println!("✅ Analyzing repository: {}", dirname.green());
                        analyzer::analyze_repo(&repo);
                    }
                }
            }
        },
        Err(_) => println!("{}", format!("❌ '{}' is NOT a git repo! Check your path.", args.path).red()),
    }
}