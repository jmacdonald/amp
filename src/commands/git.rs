use commands;
use git2;
use git2::{BranchType, Oid, Repository};
use std::path::Path;
use models::application::{Application, ClipboardContent, Mode};
use regex::Regex;

pub fn add(app: &mut Application) {
    if let Some(ref mut repo) = app.repository {
        if let Some(buf) = app.workspace.current_buffer() {
            if let Ok(ref mut index) = repo.index() {
                if let Some(ref buffer_path) = buf.path {
                    if let Some(repo_path) = repo.workdir() {
                        if let Ok(path) = buffer_path.strip_prefix(repo_path) {
                            index.add_path(path);
                            index.write();
                        }
                    }
                }
            }
        }
    }
}

pub fn copy_remote_url(app: &mut Application) {
    if_let_chain! {
        [
            let Some(ref mut repo) = app.repository,
            let Some(buf) = app.workspace.current_buffer(),
            let Ok(remote) = repo.find_remote("origin"),
            let Some(url) = remote.url(),
            let Ok(regex) = Regex::new(r"^git@github.com:(.*).git$"),
            let Some(captures) = regex.captures(url),
            let Some(gh_path) = captures.at(1),
            let Some(ref buffer_path) = buf.path,
            let Some(repo_path) = repo.workdir(),
            let Ok(relative_path) = buffer_path.strip_prefix(repo_path),
            exists_in_repo(&repo, relative_path),
            let Ok(branches) = repo.branches(Some(BranchType::Local))
        ],
        {
            // We want to build URLs that point to an object ID, so that they'll
            // refer to a snapshot of the file as it looks at this very moment.
            let last_oid = repo
              .revwalk()
              .ok()
              .and_then(|mut walker| {
                  // We need to set a starting point for the commit graph we'll
                  // traverse. We want the most recent commit, so start at HEAD.
                  walker.push_head();

                  // Pull the first revision (HEAD).
                  walker.next().and_then(|revision| revision.ok())
              });

            if let Some(oid) = last_oid {
                let line_range = match app.mode {
                    Mode::SelectLine(ref s) => {
                        // Avoid zero-based line numbers.
                        let line_1 = buf.cursor.line + 1;
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
                    oid,
                    relative_path.to_string_lossy(),
                    line_range
                );
                app.clipboard.set_content(
                    ClipboardContent::Inline(gh_url)
                );
            }
        }
    }
    commands::application::switch_to_normal_mode(app);
}

fn exists_in_repo(repo: &Repository, path: &Path) -> bool {
    if let Ok(status) = repo.status_file(path) {
        !status.contains(git2::STATUS_WT_NEW) &&
            !status.contains(git2::STATUS_INDEX_NEW)
    } else {
        false
    }
}
