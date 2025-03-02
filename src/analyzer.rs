use std::collections::HashMap;
use git2::{Repository, DiffOptions};
use prettytable::{Table, row};
use crate::models::AuthorStats;

pub fn analyze_repo(repo: &Repository) {
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