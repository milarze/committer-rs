use tokio::runtime::Builder;

use crate::{
    config::read_config,
    git::{commit, GitRepo},
};

pub fn configure() {
    println!("Configuring stuff");
}

pub fn generate() {
    println!("Generating commit message");
    let config = read_config();
    let repo = GitRepo::new();
    let diffs = repo.get_staged_diff().expect("Unable to read git diff");
    if diffs.is_empty() {
        panic!("Diff is empty. Have you staged your changes?");
    }

    let runtime = Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();
    let handle = runtime.spawn(crate::commit_generator::generate_commit_message(
        diffs.clone(),
        None,
        config,
    ));
    let inference_result = runtime.block_on(handle).expect("Unable to make API call");
    if inference_result.is_err() {
        panic!("Error generating commit message: {:?}", inference_result);
    }
    let commit_message = inference_result.unwrap();
    if commit_message.is_empty() {
        panic!("Generated commit message is empty");
    }
    commit(commit_message, None).expect("Unable to commit");
}
