use git2::{Cred, Error, PushOptions, RemoteCallbacks, Repository};
use thiserror::Error;

pub struct GitRepository {
    inner: Repository,
}

#[derive(Debug, Error)]
pub enum GitError {
    #[error("Cannot open repository")]
    CannotOpenRepository,
    #[error("Branch {0} not found")]
    BranchNotFound(String),
    #[error("Commit {0} not found")]
    CommitNotFound(String),
    #[error("Cannot create branch {0}")]
    CannotCreateBranch(String),
    #[error("Cannot checkout branch {0}: {1}")]
    CannotCheckoutBranch(String, Error),
    #[error("Cannot get HEAD")]
    CannotGetHead,
    #[error("You are not in a branch, please checkout a branch first")]
    NotInABranch,
    #[error("Repository origin not found")]
    OriginNotFound,
    #[error("Cannot push to origin: {0}")]
    CannotPushToOrigin(Error),
}

impl GitRepository {
    pub fn discover() -> Result<Self, GitError> {
        let inner = Repository::discover(".").map_err(|_| GitError::CannotOpenRepository)?;

        Ok(Self { inner })
    }

    pub fn create_and_checkout_branch(
        &self,
        new_branch: &str,
        from_branch: &str,
    ) -> Result<(), GitError> {
        let target_branch = self
            .inner
            .find_branch(from_branch, git2::BranchType::Local)
            .map_err(|_| GitError::BranchNotFound(from_branch.to_string()))?;

        let commit = target_branch
            .get()
            .peel_to_commit()
            .map_err(|_| GitError::CommitNotFound(from_branch.to_string()))?;

        let mut branch = self
            .inner
            .branch(new_branch, &commit, false)
            .map_err(|_| GitError::CannotCreateBranch(new_branch.to_string()))?;

        branch
            .set_upstream(Some(new_branch))
            .map_err(|_| GitError::CannotCreateBranch(new_branch.to_string()))?;

        let tree = branch
            .get()
            .peel(git2::ObjectType::Tree)
            .map_err(|e| GitError::CannotCheckoutBranch(new_branch.to_string(), e))?;

        self.inner
            .checkout_tree(&tree, None)
            .map_err(|e| GitError::CannotCheckoutBranch(new_branch.to_string(), e))?;

        let ref_name = match branch.get().name() {
            Some(name) => name,
            None => {
                return Err(GitError::BranchNotFound(new_branch.to_string()));
            }
        };

        self.inner
            .set_head(ref_name)
            .map_err(|e| GitError::CannotCheckoutBranch(new_branch.to_string(), e))?;

        Ok(())
    }

    pub fn push(&self) -> Result<(), GitError> {
        let reference = self.inner.head().map_err(|_| GitError::CannotGetHead)?;
        if !reference.is_branch() {
            return Err(GitError::NotInABranch);
        }

        let mut remote = self
            .inner
            .find_remote("origin")
            .map_err(|_| GitError::OriginNotFound)?;

        let ref_name = match reference.name() {
            Some(name) => name,
            None => {
                return Err(GitError::OriginNotFound);
            }
        };

        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            Cred::ssh_key_from_agent(username_from_url.unwrap())
        });

        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(callbacks);

        let refspec = format!("{}:{}", ref_name, ref_name);
        remote
            .push(&[refspec], Some(&mut push_options))
            .map_err(GitError::CannotPushToOrigin)
    }
}

pub fn to_branch_name(str: &str) -> String {
    str.chars()
        .map(|c: char| -> char {
            match c.to_ascii_lowercase() {
                m if m.is_ascii_lowercase() || m.is_ascii_digit() => m,
                _ => '-',
            }
        })
        .fold(String::default(), |acc, c| -> String {
            if c == '-' && acc.ends_with('-') {
                acc
            } else {
                format!("{}{}", acc, c)
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_name_convertion() {
        let input = "This is a string";
        let expected = "this-is-a-string";
        assert_eq!(to_branch_name(input), expected);
    }

    #[test]
    fn test_branch_name_remove_non_letter() {
        let input = "This is a string on f🔥re";
        let expected = "this-is-a-string-on-f-re";
        assert_eq!(to_branch_name(input), expected);
    }

    #[test]
    fn test_branch_name_keep_number() {
        let input = "This is a string with a 9";
        let expected = "this-is-a-string-with-a-9";
        assert_eq!(to_branch_name(input), expected);
    }

    #[test]
    fn test_branch_name_strip_dash() {
        let input = "This is a string with a     big space";
        let expected = "this-is-a-string-with-a-big-space";
        assert_eq!(to_branch_name(input), expected);
    }
}
