use crate::errors::*;
use crate::errors;
use crate::commands::{self, Result};
use crate::models::application::{Application, ClipboardContent, Mode};
use git2;
use regex::Regex;

pub fn add(app: &mut Application) -> Result {
    let repo = app.repository.as_ref().ok_or("No repository available")?;
    let buffer = app.workspace.current_buffer.as_ref().ok_or(BUFFER_MISSING)?;
    let mut index = repo.index().chain_err(|| "Couldn't get the repository index")?;
    let buffer_path = buffer.path.as_ref().ok_or(BUFFER_PATH_MISSING)?;
    let repo_path = repo.workdir().ok_or("No path found for the repository")?;
    let relative_path = buffer_path.strip_prefix(repo_path).chain_err(|| {
        "Failed to build a relative buffer path"
    })?;

    index.add_path(relative_path).chain_err(|| "Failed to add path to index.")?;
    index.write().chain_err(|| "Failed to write index.")
}

pub fn copy_remote_url(app: &mut Application) -> Result {
    if let Some(ref mut repo) = app.repository {
        let buffer = app.workspace.current_buffer.as_ref().ok_or(BUFFER_MISSING)?;
        let buffer_path = buffer.path.as_ref().ok_or(BUFFER_PATH_MISSING)?;
        let remote = repo.find_remote("origin").chain_err(|| {
            "Couldn't find a remote \"origin\""
        })?;
        let url = remote.url().ok_or("No URL for remote/origin")?;

        let gh_path = get_gh_path(url)?;

        let repo_path = repo.workdir().ok_or("No path found for the repository")?;
        let relative_path = buffer_path.strip_prefix(repo_path).chain_err(|| {
            "Failed to build a relative buffer path"
        })?;

        let status = repo.status_file(relative_path).chain_err(|| {
            "Couldn't get status info for the specified path"
        })?;
        if status.contains(git2::Status::WT_NEW) || status.contains(git2::Status::INDEX_NEW) {
            bail!("The provided path doesn't exist in the repository");
        }

        // We want to build URLs that point to an object ID, so that they'll
        // refer to a snapshot of the file as it looks at this very moment.
        let mut revisions = repo.revwalk().chain_err(|| {
            "Couldn't build a list of revisions for the repository"
        })?;

        // We need to set a starting point for the commit graph we'll
        // traverse. We want the most recent commit, so start at HEAD.
        revisions.push_head().chain_err(|| "Failed to push HEAD to commit graph.")?;

        // Pull the first revision (HEAD).
        let last_oid = revisions.next().and_then(|revision| revision.ok()).ok_or(
            "Couldn't find a git object ID for this file"
        )?;

        let line_range = match app.mode {
            Mode::SelectLine(ref s) => {
                // Avoid zero-based line numbers.
                let line_1 = buffer.cursor.line + 1;
                let line_2 = s.anchor + 1;

                if line_1 < line_2 {
                    format!("#L{}-L{}", line_1, line_2)
                } else if line_2 < line_1 {
                    format!("#L{}-L{}", line_2, line_1)
                } else {
                    format!("#L{}", line_1)
                }
            },
            _ => String::new(),
        };

        let gh_url = format!(
            "https://github.com/{}/blob/{:?}/{}{}",
            gh_path,
            last_oid,
            relative_path.to_string_lossy(),
            line_range
        );

        app.clipboard.set_content(
            ClipboardContent::Inline(gh_url)
        )?;
    } else {
        bail!("No repository available");
    }

    commands::application::switch_to_normal_mode(app)?;

    Ok(())
}

fn get_gh_path(url: &str) -> errors::Result<&str> {
    lazy_static! {
        static ref REGEX: Regex =
            Regex::new(r"^(?:https://|git@)github.com(?::|/)(.*?)(?:.git)?$").unwrap();
    }
    REGEX.captures(url).and_then(|c| c.at(1)).chain_err(|| {
        "Failed to capture remote repo path"
    })
}

#[test]
fn test_get_gh_path() {
    let cases = [
        ("git@github.com:jmacdonald/amp.git", "jmacdonald/amp"),
        ("https://github.com/jmacdonald/amp.git", "jmacdonald/amp"),
        ("https://github.com/jmacdonald/amp", "jmacdonald/amp"),
    ];

    cases.iter().for_each(|(url, expected_gh_path)| {
        assert_eq!(&get_gh_path(url).unwrap(), expected_gh_path)
    })
}
