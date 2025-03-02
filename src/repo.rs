use git2::{Repository, ObjectType, Oid};

/// Check if a tag exists in the repository
pub fn tag_exists(repo: &Repository, tag_name: &str) -> bool {
    match repo.revparse_single(&format!("refs/tags/{}", tag_name)) {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// Get the commit object from a tag name
pub fn get_commit_from_tag(repo: &Repository, tag_name: &str) -> Option<Oid> {
    match repo.revparse_single(&format!("refs/tags/{}", tag_name)) {
        Ok(object) => {
            match object.kind() {
                Some(ObjectType::Tag) => {
                    match object.peel(ObjectType::Commit) {
                        Ok(commit_obj) => commit_obj.id().into(),
                        Err(_) => None,
                    }
                },
                Some(ObjectType::Commit) => {
                    Some(object.id())
                },
                _ => None,
            }
        },
        Err(_) => None,
    }
}