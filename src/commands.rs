use crate::git::GitRepo;

pub fn configure() {
    println!("Configuring stuff");
}

pub fn generate() {
    println!("Generating commit message");
    let diffs = GitRepo::new().get_staged_diff().unwrap();
    println!("{}", diffs);
}
