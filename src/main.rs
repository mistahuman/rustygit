use clap::{Parser, Subcommand};
use git2::{Repository, DiffOptions, Oid, ObjectType};
use std::collections::HashMap;
use std::path::Path;
use std::fs;
use colored::*;
use prettytable::{Table, row};

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

struct AuthorStats {
    commits: usize,
    lines_added: usize,
    lines_deleted: usize,
}

/// Struct that represents a commit for the changelog
struct CommitInfo {
    hash: String,
    author: String,
    date: String,
    message: String,
}

fn main() {
    let args: Args = Args::parse();
    let repo_path: &Path = Path::new(&args.path);

    match Repository::open(repo_path) {
        Ok(repo) => {
            match &args.command {
                Some(Commands::Changelog { from_tag, to_tag, output }) => {
                    generate_changelog(&repo, from_tag, to_tag, output);
                },
                None => {
                    if repo.is_empty().unwrap_or(true) {
                        println!("{}", "Repository is empty. No commits to analyze.".yellow());
                    } else {
                        let dirname: &str = repo_path.file_name().and_then(|name| name.to_str()).unwrap_or("");
                        println!("✅ Analyzing repository: {}", dirname.green());
                        analyze_repo(&repo);
                    }
                }
            }
        },
        Err(_) => println!("{}", format!("❌ '{}' is NOT a git repo! Check your path.", args.path).red()),
    }
}

fn analyze_repo(repo: &Repository) {
    let mut author_stats: HashMap<String, AuthorStats> = HashMap::new();
    let mut revwalk: git2::Revwalk<'_> = repo.revwalk().expect("Failed to get revwalk");
    revwalk.push_head().expect("Failed to push head");
    
    for commit_id in revwalk {
        if let Ok(oid) = commit_id {
            if let Ok(commit) = repo.find_commit(oid) {
                let author: String = commit.author().name().unwrap_or("Unknown").to_string();
                let parent: Option<git2::Commit<'_>> = commit.parent(0).ok();

                let mut diff_opts: DiffOptions = DiffOptions::new();
                let diff: git2::Diff<'_> = if let Some(parent) = parent {
                    repo.diff_tree_to_tree(Some(&parent.tree().unwrap()), Some(&commit.tree().unwrap()), Some(&mut diff_opts)).unwrap()
                } else {
                    repo.diff_tree_to_tree(None, Some(&commit.tree().unwrap()), Some(&mut diff_opts)).unwrap()
                };
                
                let mut lines_added: usize = 0;
                let mut lines_deleted: usize = 0;
                diff.stats().map(|stats: git2::DiffStats| {
                    lines_added = stats.insertions();
                    lines_deleted = stats.deletions();
                }).ok();
                
                let entry: &mut AuthorStats = author_stats.entry(author).or_insert(AuthorStats { commits: 0, lines_added: 0, lines_deleted: 0 });
                entry.commits += 1;
                entry.lines_added += lines_added;
                entry.lines_deleted += lines_deleted;
            }
        }
    }

    let total_contributions: usize = author_stats.values().map(|s| s.lines_added + s.lines_deleted).sum();
    let mut stats_vec: Vec<_> = author_stats.into_iter().collect();
    stats_vec.sort_by(|a: &(String, AuthorStats), b: &(String, AuthorStats)| b.1.commits.cmp(&a.1.commits));
    
    let mut table = Table::new();
    table.add_row(row!["Author", "Commits", "Lines Added", "Lines Deleted", "Contribution %"]);
    
    for (author, stats) in stats_vec {
        let contribution: f64 = if total_contributions > 0 {
            (stats.lines_added + stats.lines_deleted) as f64 / total_contributions as f64 * 100.0
        } else {
            0.0
        };
        
        table.add_row(row![
            author, 
            stats.commits.to_string(), 
            stats.lines_added.to_string(), 
            stats.lines_deleted.to_string(), 
            format!("{:.2}%", contribution)
        ]);
    }
    
    table.printstd();
}

/// Check if a tag exists in the repository
fn tag_exists(repo: &Repository, tag_name: &str) -> bool {
    match repo.revparse_single(&format!("refs/tags/{}", tag_name)) {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// Get the commit object from a tag name
fn get_commit_from_tag(repo: &Repository, tag_name: &str) -> Option<Oid> {
    match repo.revparse_single(&format!("refs/tags/{}", tag_name)) {
        Ok(object) => {
            match object.kind() {
                Some(ObjectType::Tag) => {
                    // If it's an annotated tag, we need to peel it to get the commit
                    match object.peel(ObjectType::Commit) {
                        Ok(commit_obj) => commit_obj.id().into(),
                        Err(_) => None,
                    }
                },
                Some(ObjectType::Commit) => {
                    // If it's a lightweight tag (directly points to commit)
                    Some(object.id())
                },
                _ => None,
            }
        },
        Err(_) => None,
    }
}

/// Generate a changelog between two tags
fn generate_changelog(repo: &Repository, from_tag: &str, to_tag: &str, output: &Option<String>) {
    let title: String = format!("Changelog from {} to {}", from_tag, to_tag);
    // Check if tags exist
    if !tag_exists(repo, from_tag) {
        println!("{}", format!("❌ Tag '{}' does not exist!", from_tag).red());
        return;
    }
    
    if !tag_exists(repo, to_tag) {
        println!("{}", format!("❌ Tag '{}' does not exist!", to_tag).red());
        return;
    }
    
    // Get commit hashes for the tags
    let from_commit_id = match get_commit_from_tag(repo, from_tag) {
        Some(id) => id,
        None => {
            println!("{}", format!("❌ Cannot resolve tag '{}' to a commit", from_tag).red());
            return;
        }
    };
    
    let to_commit_id = match get_commit_from_tag(repo, to_tag) {
        Some(id) => id,
        None => {
            println!("{}", format!("❌ Cannot resolve tag '{}' to a commit", to_tag).red());
            return;
        }
    };
    
    println!("Generating changelog from '{}' to '{}'...", from_tag.green(), to_tag.green());
    // Get commits between the tags
    let mut commits = Vec::new();
    let mut revwalk = repo.revwalk().expect("Failed to create revwalk");
    revwalk.push(to_commit_id).expect("Failed to push to_tag");
    revwalk.hide(from_commit_id).expect("Failed to hide from_tag");
    
    for commit_id in revwalk {
        if let Ok(oid) = commit_id {
            if let Ok(commit) = repo.find_commit(oid) {
                let commit_info = CommitInfo {
                    hash: oid.to_string(),
                    author: commit.author().name().unwrap_or("Unknown").to_string(),
                    date: format_time(&commit.time()),
                    message: commit.message().unwrap_or("").to_string(),
                };
                commits.push(commit_info);
            }
        }
    }
    
    // Calculate file changes
    let from_commit = repo.find_commit(from_commit_id).expect("Failed to find from_commit");
    let to_commit = repo.find_commit(to_commit_id).expect("Failed to find to_commit");
    
    let from_tree = from_commit.tree().expect("Failed to get from_tree");
    let to_tree = to_commit.tree().expect("Failed to get to_tree");
    
    let mut diff_opts = DiffOptions::new();
    let diff = repo.diff_tree_to_tree(Some(&from_tree), Some(&to_tree), Some(&mut diff_opts))
        .expect("Failed to diff trees");
    
    let stats = diff.stats().expect("Failed to get diff stats");
    let files_changed = stats.files_changed();
    let insertions = stats.insertions();
    let deletions = stats.deletions();
    
    // Format the changelog
    let changelog = format_changelog(title, &commits, files_changed, insertions, deletions);
    
    // Save to file or print to screen
    match output {
        Some(file_path) => {
            if let Err(e) = fs::write(file_path, &changelog) {
                println!("{}", format!("❌ Failed to write to file: {}", e).red());
            } else {
                println!("✅ Changelog saved to '{}'", file_path.green());
            }
        },
        None => {
            println!("\n{}", changelog);
        }
    }
}

/// Format a git time to a readable string
fn format_time(time: &git2::Time) -> String {
    let seconds = time.seconds();
    let offset = time.offset_minutes();
    
    // Using chrono would be better here, but keeping dependencies minimal
    // This is a simple implementation
    let timestamp = seconds + (offset as i64 * 60);
    format!("{}", timestamp)
}

/// Format the changelog into a readable string
fn format_changelog(title: String,commits: &[CommitInfo], files_changed: usize, insertions: usize, deletions: usize) -> String {
    let mut result = String::new();
    
    // Add header
    result.push_str(format!("# {}\n\n", title).as_str());
    
    // Add statistics section
    result.push_str("## Statistics\n\n");
    result.push_str(&format!("- Files changed: {}\n", files_changed));
    result.push_str(&format!("- Lines added: {}\n", insertions));
    result.push_str(&format!("- Lines deleted: {}\n", deletions));
    result.push_str(&format!("- Total commits: {}\n\n", commits.len()));
    
    // Group commits by type (assuming conventional commit format)
    let mut features = Vec::new();
    let mut fixes = Vec::new();
    let mut others = Vec::new();
    
    for commit in commits {
        let msg = &commit.message;
        if msg.starts_with("feat") || msg.starts_with("feature") {
            features.push(commit);
        } else if msg.starts_with("fix") {
            fixes.push(commit);
        } else {
            others.push(commit);
        }
    }
    
    // Add sections for each commit type
    if !features.is_empty() {
        result.push_str("## New Features\n\n");
        for commit in &features {
            result.push_str(&format!("- {} ({})\n  _by {} on {}_\n", 
                commit.message.lines().next().unwrap_or(""),
                &commit.hash[..7],
                commit.author,
                commit.date
            ));
        }
        result.push('\n');
    }
    
    if !fixes.is_empty() {
        result.push_str("## Bug Fixes\n\n");
        for commit in &fixes {
            result.push_str(&format!("- {} ({})\n  _by {} on {}_\n", 
                commit.message.lines().next().unwrap_or(""),
                &commit.hash[..7],
                commit.author,
                commit.date
            ));
        }
        result.push('\n');
    }
    
    if !others.is_empty() {
        result.push_str("## Other Changes\n\n");
        for commit in &others {
            result.push_str(&format!("- {} ({})\n  _by {} on {}_\n", 
                commit.message.lines().next().unwrap_or(""),
                &commit.hash[..7],
                commit.author,
                commit.date
            ));
        }
    }
    
    result
}