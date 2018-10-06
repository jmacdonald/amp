mod clipboard;
mod event;
pub mod modes;
mod preferences;

// Published API
pub use self::clipboard::ClipboardContent;
pub use self::event::Event;
pub use self::preferences::Preferences;

use self::clipboard::Clipboard;
use self::modes::*;
use commands;
use errors::*;
use git2::Repository;
use presenters;
use scribe::{Buffer, Workspace};
use std::cell::RefCell;
use std::env;
use std::ops::Drop;
use std::path::Path;
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use view::terminal::*;
use view::{self, StatusLineData, View};

pub enum Mode {
    Confirm(ConfirmMode),
    Command(CommandMode),
    Exit,
    Insert,
    Jump(JumpMode),
    LineJump(LineJumpMode),
    Path(PathMode),
    Normal,
    Open(OpenMode),
    Select(SelectMode),
    SelectLine(SelectLineMode),
    Search(SearchMode),
    SymbolJump(SymbolJumpMode),
    Theme(ThemeMode),
}

pub struct Application {
    pub mode: Mode,
    pub workspace: Workspace,
    pub search_query: Option<String>,
    pub view: View,
    pub clipboard: Clipboard,
    pub repository: Option<Repository>,
    pub error: Option<Error>,
    pub preferences: Rc<RefCell<Preferences>>,
    pub event_channel: Sender<Event>,
    events: Receiver<Event>,
}

impl Application {
    pub fn new() -> Result<Application> {
        let preferences = initialize_preferences();

        let (event_channel, events) = mpsc::channel();
        let mut view = View::new(build_terminal(), preferences.clone(), event_channel.clone())?;
        let clipboard = Clipboard::new();

        // Set up a workspace in the current directory.
        let workspace = create_workspace(&mut view)?;

        Ok(Application {
            mode: Mode::Normal,
            workspace,
            search_query: None,
            view,
            clipboard,
            repository: Repository::discover(&env::current_dir()?).ok(),
            error: None,
            preferences,
            event_channel,
            events,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            self.render();
            self.wait_for_event()?;

            if let Mode::Exit = self.mode {
                break;
            }
        }

        Ok(())
    }

    fn render(&mut self) {
        if let Err(error) = self.present() {
            render_error(&mut self.view, &error);
        } else if let Some(ref error) = self.error {
            // Display an error from previous command invocation, if one exists.
            render_error(&mut self.view, error);
        }
    }

    fn present(&mut self) -> Result<()> {
        match self.mode {
            Mode::Confirm(_) => {
                presenters::modes::confirm::display(&mut self.workspace, &mut self.view)
            }
            Mode::Command(ref mut mode) => {
                presenters::modes::search_select::display(&mut self.workspace, mode, &mut self.view)
            }
            Mode::Insert => presenters::modes::insert::display(&mut self.workspace, &mut self.view),
            Mode::Open(ref mut mode) => {
                presenters::modes::search_select::display(&mut self.workspace, mode, &mut self.view)
            }
            Mode::Search(ref mode) => {
                presenters::modes::search::display(&mut self.workspace, mode, &mut self.view)
            }
            Mode::Jump(ref mut mode) => {
                presenters::modes::jump::display(&mut self.workspace, mode, &mut self.view)
            }
            Mode::LineJump(ref mode) => {
                presenters::modes::line_jump::display(&mut self.workspace, mode, &mut self.view)
            }
            Mode::Path(ref mode) => {
                presenters::modes::path::display(&mut self.workspace, mode, &mut self.view)
            }
            Mode::SymbolJump(ref mut mode) => {
                presenters::modes::search_select::display(&mut self.workspace, mode, &mut self.view)
            }
            Mode::Select(ref mode) => {
                presenters::modes::select::display(&mut self.workspace, mode, &mut self.view)
            }
            Mode::SelectLine(ref mode) => {
                presenters::modes::select_line::display(&mut self.workspace, mode, &mut self.view)
            }
            Mode::Normal => presenters::modes::normal::display(
                &mut self.workspace,
                &mut self.view,
                &self.repository,
            ),
            Mode::Theme(ref mut mode) => {
                presenters::modes::search_select::display(&mut self.workspace, mode, &mut self.view)
            }
            Mode::Exit => Ok(()),
        }
    }

    fn wait_for_event(&mut self) -> Result<()> {
        let event = self
            .events
            .recv()
            .chain_err(|| "Error receiving application event")?;
        match event {
            Event::Key(key) => {
                self.view.last_key = Some(key);
                self.error = commands::application::handle_input(self).err();
            }
            Event::Resize => {}
            Event::OpenModeIndexComplete(index) => {
                if let Mode::Open(ref mut open_mode) = self.mode {
                    open_mode.set_index(index);

                    // Trigger a search, in case a query was
                    // entered while we were indexing.
                    open_mode.search();
                }
            }
        }

        Ok(())
    }

    pub fn mode_str(&self) -> Option<&'static str> {
        match self.mode {
            Mode::Command(ref mode) => if mode.insert_mode() {
                Some("search_select_insert")
            } else {
                Some("search_select")
            },
            Mode::SymbolJump(ref mode) => if mode.insert_mode() {
                Some("search_select_insert")
            } else {
                Some("search_select")
            },
            Mode::Open(ref mode) => if mode.insert_mode() {
                Some("search_select_insert")
            } else {
                Some("search_select")
            },
            Mode::Theme(ref mode) => if mode.insert_mode() {
                Some("search_select_insert")
            } else {
                Some("search_select")
            },
            Mode::Normal => Some("normal"),
            Mode::Path(_) => Some("path"),
            Mode::Confirm(_) => Some("confirm"),
            Mode::Insert => Some("insert"),
            Mode::Jump(_) => Some("jump"),
            Mode::LineJump(_) => Some("line_jump"),
            Mode::Select(_) => Some("select"),
            Mode::SelectLine(_) => Some("select_line"),
            Mode::Search(ref mode) => if mode.insert_mode() {
                Some("search_insert")
            } else {
                Some("search")
            },
            Mode::Exit => None,
        }
    }
}

impl Drop for Application {
    fn drop(&mut self) {
        self.view.clear();
    }
}

fn render_error(view: &mut View, error: &Error) {
    view.draw_status_line(&[StatusLineData {
        content: error.description().to_string(),
        style: view::Style::Bold,
        colors: view::Colors::Warning,
    }]);
    view.present();
}

fn initialize_preferences() -> Rc<RefCell<Preferences>> {
    Rc::new(RefCell::new(
        Preferences::load().unwrap_or_else(|_| Preferences::new(None)),
    ))
}

fn create_workspace(view: &mut View) -> Result<Workspace> {
    // Move into an argument-specified directory, if present.
    if let Some(arg) = env::args().nth(1) {
        let path = Path::new(&arg).canonicalize()?;

        if path.is_dir() {
            env::set_current_dir(&path)?;
        }
    }

    let workspace_dir = env::current_dir()?;
    let mut workspace = Workspace::new(&workspace_dir)?;

    // Try to open specified files.
    for path_arg in env::args().skip(1) {
        let path = Path::new(&path_arg);

        // We're only interested in files.
        if !path.is_file() {
            continue;
        }

        // Open the specified path if it exists, or
        // create a new buffer pointing to it if it doesn't.
        let mut argument_buffer = if path.exists() {
            Buffer::from_file(path)?
        } else {
            let mut buffer = Buffer::new();

            // Point the buffer to the path, ensuring that it's absolute.
            if path.is_absolute() {
                buffer.path = Some(path.to_path_buf());
            } else {
                buffer.path = Some(workspace.path.join(path));
            }

            buffer
        };
        workspace.add_buffer(argument_buffer);
        view.initialize_buffer(workspace.current_buffer().unwrap())?;
    }

    // Add user syntax definitions.
    let syntax_path = Preferences::syntax_path()?;
    if let Err(e) = workspace.syntax_set.load_syntaxes(syntax_path, true) {
        bail!("Failed to load user syntaxes: {:?}", e);
    }
    workspace.syntax_set.link_syntaxes();

    Ok(workspace)
}

#[cfg(not(any(test, feature = "bench")))]
fn build_terminal() -> Arc<Terminal + Sync + Send> {
    Arc::new(RustboxTerminal::new())
}

#[cfg(any(test, feature = "bench"))]
fn build_terminal() -> Arc<Terminal + Sync + Send> {
    // Use a headless terminal if we're in test mode.
    Arc::new(TestTerminal::new())
}
