use git2::{Commit, Oid};
use std::fs;
use std::io::{Read, Write};
use std::process::Command;
use tempfile::NamedTempFile;

use super::GitRepo;

pub fn commit(
    repo: &GitRepo,
    commit_msg: String,
    diffs: String,
) -> Result<Oid, Box<dyn std::error::Error>> {
    let repo = &repo.repo;
    let mut index = repo.index()?;

    // Write the index to the tree
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    // Prepare parent commits
    let mut parents = Vec::new();
    if let Ok(head) = repo.head() {
        if let Some(target) = head.target() {
            if let Ok(commit) = repo.find_commit(target) {
                parents.push(commit);
            }
        }
    }

    // Get the signature
    let signature = repo.signature()?;

    // Create a temporary file for the commit message
    let mut temp_file = NamedTempFile::new()?;

    // Determine the initial message content
    let mut message = String::new();
    message.push_str(&commit_msg);

    // Add commit information to the message
    message.push_str("\n\n# Please enter the commit message for your changes. Lines starting\n");
    message.push_str("# with '#' will be ignored, and an empty message aborts the commit.\n#\n");

    // Add a summary of the changes
    message.push_str("# Changes to be committed:\n");
    for line in diffs.lines() {
        message.push_str(&format!("# \t{}\n", line));
    }

    // Write the message to the temporary file
    write!(temp_file, "{}", message)?;

    // Determine which editor to use
    let editor = repo
        .config()?
        .get_string("core.editor")
        .or_else(|_| std::env::var("GIT_EDITOR"))
        .or_else(|_| std::env::var("VISUAL"))
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| String::from("vi"));

    // Launch the editor
    let status = Command::new(editor).arg(temp_file.path()).status()?;

    if !status.success() {
        return Err("Editor returned non-zero exit code".into());
    }

    // Read the edited message
    let mut edited_message = String::new();
    temp_file.reopen()?.read_to_string(&mut edited_message)?;

    // Remove comment lines and get trimmed message
    let commit_message = edited_message
        .lines()
        .filter(|line| !line.starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string();

    // Check for empty message
    if commit_message.is_empty() {
        return Err("Aborting commit due to empty commit message".into());
    }

    // Run commit-msg hook
    let msg_file = NamedTempFile::new()?;
    fs::write(msg_file.path(), &commit_message)?;

    // Convert parents to references
    let parent_refs: Vec<&Commit> = parents.iter().collect();

    // Create the commit
    let commit_id = repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        &commit_message,
        &tree,
        &parent_refs,
    )?;

    // Run post-commit hook

    Ok(commit_id)
}
