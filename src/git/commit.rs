use std::process::Command;

pub fn commit(commit_msg: String, body: Option<String>) -> anyhow::Result<()> {
    if let Some(body) = body {
        Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(commit_msg)
            .arg("-m")
            .arg(body)
            .arg("-e")
            .output()?;
    } else {
        Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(commit_msg)
            .arg("-e")
            .output()?;
    }
    Ok(())
}
