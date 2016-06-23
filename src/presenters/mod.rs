extern crate git2;
extern crate scribe;
extern crate rustbox;

pub mod modes;

use std::path::PathBuf;
use scribe::Buffer;
use view::StatusLineData;
use git2::{Repository, Status};

fn path_as_title(path: Option<PathBuf>) -> String {
    format!(" {}", path.map(|path| path.to_string_lossy().into_owned()).unwrap_or("".to_string()))
}

fn buffer_status_line_data(buffer: &Buffer) -> StatusLineData {
    // Determine buffer title styles based on its modification status.
    let (buffer_title, buffer_title_style) = if buffer.modified() {
        // Use an emboldened path with an asterisk.
        let mut title = path_as_title(buffer.path.clone());
        title.push('*');

        (title, Some(rustbox::RB_BOLD))
    } else {
        (path_as_title(buffer.path.clone()), None)
    };

    StatusLineData {
        content: buffer_title,
        style: buffer_title_style,
        background_color: None,
        foreground_color: None,
    }
}

fn git_status_line_data(repo: &Option<Repository>, path: &Option<PathBuf>) -> StatusLineData {
    // Build a display value for the current buffer's git status.
    let mut content = String::new();
    if let &Some(ref repo) = repo {
        if let &Some(ref path) = path {
            if let Ok(status) = repo.status_file(path) {
                content = presentable_status(&status).to_string();
            }
        }
    }

    StatusLineData {
        content: content,
        style: None,
        background_color: None,
        foreground_color: None,
    }
}
fn presentable_status(status: &Status) -> &str {
    if status.contains(git2::STATUS_WT_NEW) {
        if status.contains(git2::STATUS_INDEX_NEW) {
            // Parts of the file are staged as new in the index.
            "[partially staged]"
        } else {
            // The file has never been added to the repository.
            "[untracked]"
        }
    } else if status.contains(git2::STATUS_INDEX_NEW) {
        // The complete file is staged as new in the index.
        "[staged]"
    } else {
        if status.contains(git2::STATUS_WT_MODIFIED) {
            if status.contains(git2::STATUS_INDEX_MODIFIED) {
                // The file has both staged and unstaged modifications.
                "[partially staged]"
            } else {
                // The file has unstaged modifications.
                "[modified]"
            }
        } else if status.contains(git2::STATUS_INDEX_MODIFIED) {
            // The file has staged modifications.
            "[staged]"
        } else {
            // The file is tracked, but has no modifications.
            "[ok]"
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate git2;
    extern crate scribe;

    use super::{presentable_status, visible_tokens};
    use scribe::buffer::{Buffer, LineRange, Token, Category};

    #[test]
    fn visible_tokens_returns_tokens_in_the_specified_range() {
        let mut buffer = Buffer::new();
        buffer.insert("first\nsecond\nthird\nfourth");

        let tokens = visible_tokens(&buffer.tokens(), LineRange::new(1, 3));
        assert_eq!(tokens,
                   vec![
                Token{ lexeme: "second".to_string(), category: Category::Text },
                Token{ lexeme: "\n".to_string(), category: Category::Whitespace },
                Token{ lexeme: "third".to_string(), category: Category::Text },
                Token{ lexeme: "\n".to_string(), category: Category::Whitespace },
            ]);
    }

    #[test]
    pub fn presentable_status_returns_untracked_when_status_is_locally_new() {
        let status = git2::STATUS_WT_NEW;
        assert_eq!(presentable_status(&status), "[untracked]".to_string());
    }

    #[test]
    pub fn presentable_status_returns_ok_when_status_unmodified() {
        let status = git2::STATUS_CURRENT;
        assert_eq!(presentable_status(&status), "[ok]".to_string());
    }

    #[test]
    pub fn presentable_status_returns_staged_when_only_modified_in_index() {
        let status = git2::STATUS_INDEX_MODIFIED;
        assert_eq!(presentable_status(&status), "[staged]".to_string());
    }

    #[test]
    pub fn presentable_status_returns_staged_when_new_in_index() {
        let status = git2::STATUS_INDEX_NEW;
        assert_eq!(presentable_status(&status), "[staged]".to_string());
    }

    #[test]
    pub fn presentable_status_returns_partially_staged_when_modified_locally_and_in_index() {
        let status = git2::STATUS_WT_MODIFIED | git2::STATUS_INDEX_MODIFIED;
        assert_eq!(presentable_status(&status),
                   "[partially staged]".to_string());
    }

    #[test]
    pub fn presentable_status_returns_partially_staged_when_new_locally_and_in_index() {
        let status = git2::STATUS_WT_NEW | git2::STATUS_INDEX_NEW;
        assert_eq!(presentable_status(&status),
                   "[partially staged]".to_string());
    }
}
