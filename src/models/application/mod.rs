pub mod modes;
mod clipboard;
mod preferences;

// Published API
pub use self::clipboard::ClipboardContent;
pub use self::preferences::Preferences;

use errors::*;
use std::env;
use std::path::Path;
use std::cell::RefCell;
use std::rc::Rc;
use input;
use presenters;
use self::modes::*;
use scribe::{Buffer, Workspace};
use view::{self, StatusLineData, View};
use self::clipboard::Clipboard;
use git2::Repository;

pub enum Mode {
    Confirm(ConfirmMode),
    Command(CommandMode),
    Exit,
    Insert,
    Jump(JumpMode),
    LineJump(LineJumpMode),
    Normal,
    Open(OpenMode),
    Select(SelectMode),
    SelectLine(SelectLineMode),
    SearchInsert(SearchInsertMode),
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
}

impl Application {
    pub fn new() -> Result<Application> {
        let current_dir = env::current_dir()?;

        // TODO: Log errors to disk.
        let preferences =
            Rc::new(RefCell::new(Preferences::load().unwrap_or_else(|_| Preferences::new(None))));

        // Set up a workspace in the current directory.
        let mut workspace = Workspace::new(&current_dir)?;

        // Try to open the specified file.
        // TODO: Handle non-existent files as new empty buffers.
        for path_arg in env::args().skip(1) {
            let path = Path::new(&path_arg);

            let argument_buffer = if path.exists() {
                // Load the buffer from disk.
                Buffer::from_file(path)?
            } else {
                // Build an empty buffer.
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
        }

        let view = View::new(preferences.clone())?;
        let clipboard = Clipboard::new();

        Ok(Application {
               mode: Mode::Normal,
               workspace: workspace,
               search_query: None,
               view: view,
               clipboard: clipboard,
               repository: Repository::discover(&current_dir).ok(),
               error: None,
               preferences: preferences,
           })
    }

    pub fn run() -> Result<()> {
        let mut application = Application::new()?;

        loop {
            // Present the application state to the view.
            match application.mode {
                Mode::Confirm(_) => {
                    presenters::modes::confirm::display(&mut application.workspace,
                                                        &mut application.view)
                },
                Mode::Command(ref mut mode) => {
                    presenters::modes::search_select::display(&mut application.workspace,
                                                              mode,
                                                              &mut application.view)
                }
                Mode::Insert => {
                    presenters::modes::insert::display(&mut application.workspace,
                                                       &mut application.view)
                }
                Mode::Open(ref mut mode) => {
                    presenters::modes::search_select::display(&mut application.workspace,
                                                              mode,
                                                              &mut application.view)
                }
                Mode::SearchInsert(ref mode) => {
                    presenters::modes::search_insert::display(&mut application.workspace,
                                                              mode,
                                                              &mut application.view)
                }
                Mode::Jump(ref mut mode) => {
                    presenters::modes::jump::display(&mut application.workspace,
                                                     mode,
                                                     &mut application.view)
                }
                Mode::LineJump(ref mode) => {
                    presenters::modes::line_jump::display(&mut application.workspace,
                                                          mode,
                                                          &mut application.view)
                }
                Mode::SymbolJump(ref mut mode) => {
                    presenters::modes::search_select::display(&mut application.workspace,
                                                              mode,
                                                              &mut application.view)
                }
                Mode::Select(ref mode) => {
                    presenters::modes::select::display(&mut application.workspace,
                                                       mode,
                                                       &mut application.view)
                }
                Mode::SelectLine(ref mode) => {
                    presenters::modes::select_line::display(&mut application.workspace,
                                                            mode,
                                                            &mut application.view)
                }
                Mode::Normal => {
                    presenters::modes::normal::display(&mut application.workspace,
                                                       &mut application.view,
                                                       &application.repository)
                }
                Mode::Theme(ref mut mode) => {
                    presenters::modes::search_select::display(&mut application.workspace,
                                                              mode,
                                                              &mut application.view)
                }
                Mode::Exit => ()
            }

            // Display an error from previous command invocation, if one exists.
            if let Some(ref error) = application.error {
                application
                    .view
                    .draw_status_line(
                        &vec![StatusLineData{
                            content: error.description().to_string(),
                            style: view::Style::Bold,
                            colors: view::Colors::Warning,
                        }]
                    );
                application.view.present();
            }

            // Listen for and respond to user input.
            let command = application.view.listen().and_then(|key| {
                // Pass the input to the current mode.
                match application.mode {
                    Mode::Command(ref mode) => if mode.insert_mode() {
                        input::modes::search_select_insert::handle(key)
                    } else {
                        input::modes::search_select::handle(key)
                    },
                    Mode::SymbolJump(ref mode) => if mode.insert_mode() {
                        input::modes::search_select_insert::handle(key)
                    } else {
                        input::modes::search_select::handle(key)
                    },
                    Mode::Open(ref mode) => if mode.insert_mode() {
                        input::modes::search_select_insert::handle(key)
                    } else {
                        input::modes::search_select::handle(key)
                    },
                    Mode::Theme(ref mode) => if mode.insert_mode() {
                        input::modes::search_select_insert::handle(key)
                    } else {
                        input::modes::search_select::handle(key)
                    },
                    Mode::Normal => input::modes::normal::handle(key),
                    Mode::Confirm(_) => input::modes::confirm::handle(key),
                    Mode::Insert => input::modes::insert::handle(key),
                    Mode::Jump(_) => input::modes::jump::handle(key),
                    Mode::LineJump(_) => input::modes::line_jump::handle(key),
                    Mode::Select(_) => input::modes::select::handle(key),
                    Mode::SelectLine(_) => input::modes::select_line::handle(key),
                    Mode::SearchInsert(_) => input::modes::search_insert::handle(key),
                    Mode::Exit => None,
                }
            });

            if let Some(com) = command {
                // Run the command and store its error output.
                application.error = com(&mut application).err();
            }

            // Check if the command resulted in an exit, before
            // looping again and asking for input we won't use.
            if let Mode::Exit = application.mode {
                application.view.clear();
                break
            }
        }

        Ok(())
    }
}
