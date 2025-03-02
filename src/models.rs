pub struct AuthorStats {
    pub commits: usize,
    pub lines_added: usize,
    pub lines_deleted: usize,
}

pub struct CommitInfo {
    pub hash: String,
    pub author: String,
    pub date: String,
    pub message: String,
}