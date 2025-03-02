use std::fs;
use colored::*;
use git2::{Repository, DiffOptions};
use crate::models::CommitInfo;
use crate::repo::{tag_exists, get_commit_from_tag};
use crate::utils::format_time;


/// Generate a changelog between two tags
pub fn generate_changelog(repo: &Repository, from_tag: &str, to_tag: &str, output: &Option<String>) {
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

/// Format the changelog into a readable string
pub fn format_changelog(title: String,commits: &[CommitInfo], files_changed: usize, insertions: usize, deletions: usize) -> String {
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
        if msg.starts_with("Merged PR") || msg.starts_with("feature") || msg.starts_with("task") {
            features.push(commit);
        } else if msg.starts_with("fix") || msg.starts_with("bug") {
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