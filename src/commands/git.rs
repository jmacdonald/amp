use git2;
use git2::{BranchType, Repository};
use std::path::PathBuf;
use models::application::{Application, ClipboardContent};
use regex::Regex;

pub fn add(app: &mut Application) {
    if let Some(ref mut repo) = app.repository {
        if let Some(buf) = app.workspace.current_buffer() {
            if let Ok(ref mut index) = repo.index() {
                if let Some(ref path) = buf.path {
                    index.add_path(&path);
                    index.write();
                }
            }
        }
    }
}

pub fn copy_remote_url(app: &mut Application) {
    if let Some(ref mut repo) = app.repository {
        if let Some(buf) = app.workspace.current_buffer() {
            let remote = repo.find_remote("origin").unwrap();
            let url = remote.url().unwrap();
            if let Ok(regex) = Regex::new(r"^git@github.com:(.*).git$") {
                if let Some(captures) = regex.captures(url) {
                    if let Some(gh_path) = captures.at(1) {
                        if let Some(ref path) = buf.path {
                            if exists_in_repo(repo, path) {
                                if let Ok(branches) = repo.branches(Some(BranchType::Local)) {
                                    for (branch, _) in branches {
                                        if branch.is_head() {
                                            if let Ok(Some(branch_name)) = branch.name() {
                                                let gh_url = format!(
                                                    "https://github.com/{}/tree/{}/{}",
                                                    gh_path,
                                                    branch_name,
                                                    path.to_string_lossy()
                                                );
                                                app.clipboard.set_content(
                                                    ClipboardContent::Inline(gh_url)
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            };
        }
    }
}

fn exists_in_repo(repo: &Repository, path: &PathBuf) -> bool {
    if let Ok(status) = repo.status_file(path) {
        !status.contains(git2::STATUS_WT_NEW) &&
            !status.contains(git2::STATUS_INDEX_NEW)
    } else {
        false
    }
}
