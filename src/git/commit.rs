use std::process::Command;

use tracing::instrument;

#[instrument(level = "info", skip(commit_msg, body))]
pub fn commit(commit_msg: String, body: Option<String>) -> anyhow::Result<()> {
    if let Some(body) = body {
        let mut child = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(commit_msg)
            .arg("-m")
            .arg(body)
            .arg("-e")
            .spawn()?;
        child.wait()?;
    } else {
        let mut child = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(commit_msg)
            .arg("-e")
            .spawn()?;
        child.wait()?;
    }
    
    Ok(())
}
