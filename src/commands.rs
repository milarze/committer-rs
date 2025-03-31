use tokio::runtime::Builder;

use crate::{config::read_config, git::GitRepo};

pub fn configure() {
    println!("Configuring stuff");
}

pub fn generate() {
    println!("Generating commit message");
    let config = read_config();
    let diffs = GitRepo::new()
        .get_staged_diff()
        .expect("Unable to read git diff");
    if diffs.is_empty() {
        panic!("Diff is empty. Have you staged your changes?");
    }

    let runtime = Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();
    let handle = runtime.spawn(crate::commit_generator::generate_commit(
        diffs, None, config,
    ));
    let result = runtime.block_on(handle).expect("Unable to make API call");
    println!("Result: {:?}", result);
}
