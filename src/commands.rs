use std::process;

use tokio::runtime::Builder;
use tracing::instrument;

use crate::{
    config::read_config,
    git::{commit, GitRepo},
};

pub fn configure() {
    println!("Configuring stuff");
}

#[instrument(level = "info")]
pub fn generate() {
    let config = read_config();
    let repo = GitRepo::new();
    let diffs = repo.get_staged_diff().expect("Unable to read git diff");

    if diffs.is_empty() {
        eprintln!("Diff is empty. Have you staged your changes?");
        process::exit(1);
    }

    let runtime = Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();

    let handle = runtime.spawn(crate::generators::remote::generate_commit_message(
        diffs.clone(),
        None,
        config,
    ));

    let inference_result = match runtime.block_on(handle) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Error generating commit message: {:?}", e);
            process::exit(1);
        }
    };

    let commit_message = match inference_result {
        Ok(message) => message,
        Err(e) => {
            eprintln!("Error generating commit message: {:?}", e);
            process::exit(1);
        }
    };

    match commit(commit_message, None) {
        Ok(_) => {
            println!("Commit successful");
        }
        Err(e) => {
            eprintln!("Error committing changes: {:?}", e);
            process::exit(1);
        }
    }
}
