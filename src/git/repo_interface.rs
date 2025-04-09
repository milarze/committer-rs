use git2::{DiffOptions, Repository, Tree};
use tracing::instrument;
pub struct GitRepo {
    pub repo: Repository,
}

impl GitRepo {
    #[instrument(level = "info")]
    pub fn new() -> Self {
        let repo = Repository::open_from_env().expect("Not a git repository");
        GitRepo { repo }
    }

    #[instrument(level = "info", skip(self))]
    pub fn get_staged_diff(&self) -> Result<String, git2::Error> {
        let index = self.repo.index()?;

        let head_tree: Option<Tree> = match self.repo.head() {
            Ok(head) => {
                let head_commit = head.peel_to_commit()?;
                Some(head_commit.tree()?)
            }
            Err(e) => {
                // Check if the error is due to an unborn branch (no commits yet)
                if e.code() == git2::ErrorCode::UnbornBranch
                    || e.code() == git2::ErrorCode::NotFound
                {
                    None
                } else {
                    return Err(e);
                }
            }
        };

        let mut diff_options = DiffOptions::new();
        let mut diffs = String::new();
        self.repo
            .diff_tree_to_index(head_tree.as_ref(), Some(&index), Some(&mut diff_options))?
            .print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
                diffs.push_str(&format!("{}\n", String::from_utf8_lossy(line.content())));
                true
            })?;
        Ok(diffs)
    }
}

impl std::fmt::Debug for GitRepo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GitRepo {{ repo: {} }}", self.repo.path().display())
    }
}
