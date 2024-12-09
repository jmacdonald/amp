mod clipboard;
mod event;
pub mod modes;
mod preferences;

// Published API
pub use self::clipboard::ClipboardContent;
pub use self::event::Event;
pub use self::modes::{Mode, ModeKey};
pub use self::preferences::Preferences;

use self::clipboard::Clipboard;
use self::modes::*;
use crate::commands;
use crate::errors::*;
use crate::presenters;
use crate::view::View;
use git2::Repository;
use scribe::buffer::Position;
use scribe::{Buffer, Workspace};
use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::mem;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver, Sender};

pub struct Application {
    pub mode: Mode,
    pub workspace: Workspace,
    pub view: View,
    pub clipboard: Clipboard,
    pub repository: Option<Repository>,
    pub error: Option<Error>,
    pub preferences: Rc<RefCell<Preferences>>,
    pub event_channel: Sender<Event>,
    events: Receiver<Event>,
    current_mode: ModeKey,
    previous_mode: ModeKey,
    modes: HashMap<ModeKey, Mode>,
}

impl Application {
    pub fn new(args: &[String]) -> Result<Application> {
        let preferences = initialize_preferences();

        let (event_channel, events) = mpsc::channel();
        let mut view = View::new(preferences.clone(), event_channel.clone())?;
        let clipboard = Clipboard::new();

        // Set up a workspace in the current directory.
        let workspace = create_workspace(&mut view, &preferences.borrow(), args)?;

        let mut app = Application {
            current_mode: ModeKey::Normal,
            previous_mode: ModeKey::Normal,
            mode: Mode::Normal,
            modes: HashMap::new(),
            workspace,
            view,
            clipboard,
            repository: Repository::discover(env::current_dir()?).ok(),
            error: None,
            preferences,
            event_channel,
            events,
        };

        app.create_modes()?;

        Ok(app)
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            self.render()?;
            self.wait_for_event()?;

            if let Mode::Exit = self.mode {
                debug_log!("[application] breaking main run loop");

                break;
            }
        }

        Ok(())
    }

    fn render(&mut self) -> Result<()> {
        if let Err(error) = self.present() {
            presenters::error::display(&mut self.workspace, &mut self.view, &error)?;
        }

        Ok(())
    }

    fn present(&mut self) -> Result<()> {
        match self.mode {
            Mode::Confirm(_) => presenters::modes::confirm::display(
                &mut self.workspace,
                &mut self.view,
                &self.error,
            ),
            Mode::Command(ref mut mode) => presenters::modes::search_select::display(
                &mut self.workspace,
                mode,
                &mut self.view,
                &self.error,
            ),
            Mode::Insert => {
                presenters::modes::insert::display(&mut self.workspace, &mut self.view, &self.error)
            }
            Mode::Open(ref mut mode) => presenters::modes::open::display(
                &mut self.workspace,
                mode,
                &mut self.view,
                &self.error,
            ),
            Mode::Search(ref mode) => presenters::modes::search::display(
                &mut self.workspace,
                mode,
                &mut self.view,
                &self.error,
            ),
            Mode::Jump(ref mut mode) => presenters::modes::jump::display(
                &mut self.workspace,
                mode,
                &mut self.view,
                &self.error,
            ),
            Mode::LineJump(ref mode) => presenters::modes::line_jump::display(
                &mut self.workspace,
                mode,
                &mut self.view,
                &self.error,
            ),
            Mode::Paste => {
                presenters::modes::paste::display(&mut self.workspace, &mut self.view, &self.error)
            }
            Mode::Path(ref mode) => presenters::modes::path::display(
                &mut self.workspace,
                mode,
                &mut self.view,
                &self.error,
            ),
            Mode::SymbolJump(ref mut mode) => presenters::modes::search_select::display(
                &mut self.workspace,
                mode,
                &mut self.view,
                &self.error,
            ),
            Mode::Syntax(ref mut mode) => presenters::modes::search_select::display(
                &mut self.workspace,
                mode,
                &mut self.view,
                &self.error,
            ),
            Mode::Select(ref mode) => presenters::modes::select::display(
                &mut self.workspace,
                mode,
                &mut self.view,
                &self.error,
            ),
            Mode::SelectLine(ref mode) => presenters::modes::select_line::display(
                &mut self.workspace,
                mode,
                &mut self.view,
                &self.error,
            ),
            Mode::Normal => presenters::modes::normal::display(
                &mut self.workspace,
                &mut self.view,
                &self.repository,
                &self.error,
            ),
            Mode::Theme(ref mut mode) => presenters::modes::search_select::display(
                &mut self.workspace,
                mode,
                &mut self.view,
                &self.error,
            ),
            Mode::Exit => Ok(()),
        }
    }

    fn wait_for_event(&mut self) -> Result<()> {
        debug_log!("[application loop]: blocking on event channel");

        // Main blocking wait
        let event = self
            .events
            .recv()
            .chain_err(|| "Error receiving application event")?;

        debug_log!("[application loop]: received event: {:?}", event);

        self.handle_event(event);

        debug_log!("[application loop]: draining event channel");

        // Handle any other events included in the batch before rendering
        // and waiting again.
        loop {
            match self.events.try_recv() {
                Ok(event) => {
                    debug_log!("[application loop]: received event: {:?}", event);

                    self.handle_event(event);
                }
                _ => break,
            }
        }

        debug_log!("[application loop]: drained event channel");

        Ok(())
    }

    fn handle_event(&mut self, event: Event) {
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
    }

    pub fn mode_str(&self) -> Option<&'static str> {
        match self.mode {
            Mode::Command(ref mode) => {
                if mode.insert_mode() {
                    Some("search_select_insert")
                } else {
                    Some("search_select")
                }
            }
            Mode::SymbolJump(ref mode) => {
                if mode.insert_mode() {
                    Some("search_select_insert")
                } else {
                    Some("search_select")
                }
            }
            Mode::Open(ref mode) => {
                if mode.insert_mode() {
                    Some("search_select_insert")
                } else {
                    Some("search_select")
                }
            }
            Mode::Theme(ref mode) => {
                if mode.insert_mode() {
                    Some("search_select_insert")
                } else {
                    Some("search_select")
                }
            }
            Mode::Syntax(ref mode) => {
                if mode.insert_mode() {
                    Some("search_select_insert")
                } else {
                    Some("search_select")
                }
            }
            Mode::Normal => Some("normal"),
            Mode::Paste => Some("paste"),
            Mode::Path(_) => Some("path"),
            Mode::Confirm(_) => Some("confirm"),
            Mode::Insert => Some("insert"),
            Mode::Jump(_) => Some("jump"),
            Mode::LineJump(_) => Some("line_jump"),
            Mode::Select(_) => Some("select"),
            Mode::SelectLine(_) => Some("select_line"),
            Mode::Search(ref mode) => {
                if mode.insert_mode() {
                    Some("search_insert")
                } else {
                    Some("search")
                }
            }
            Mode::Exit => None,
        }
    }

    pub fn switch_to(&mut self, mode_key: ModeKey) {
        if self.current_mode == mode_key {
            return;
        }

        debug_log!("[application] switching to {:?}", mode_key);

        // Check out the specified mode.
        let mut mode = self.modes.remove(&mode_key).unwrap();

        // Activate the specified mode.
        mem::swap(&mut self.mode, &mut mode);

        // Check in the previous mode.
        self.modes.insert(self.current_mode, mode);

        // Track the previous mode.
        self.previous_mode = self.current_mode;

        // Track the new active mode.
        self.current_mode = mode_key;

        debug_log!("[application] switched to {:?}", mode_key);
    }

    pub fn switch_to_previous_mode(&mut self) {
        self.switch_to(self.previous_mode);
    }

    fn create_modes(&mut self) -> Result<()> {
        // Do the easy ones first.
        self.modes.insert(ModeKey::Exit, Mode::Exit);
        self.modes.insert(ModeKey::Insert, Mode::Insert);
        self.modes.insert(ModeKey::Normal, Mode::Normal);
        self.modes.insert(ModeKey::Paste, Mode::Paste);

        self.modes.insert(
            ModeKey::Command,
            Mode::Command(CommandMode::new(
                self.preferences.borrow().search_select_config(),
            )),
        );
        self.modes.insert(
            ModeKey::Confirm,
            Mode::Confirm(ConfirmMode::new(
                commands::application::switch_to_normal_mode,
            )),
        );
        self.modes
            .insert(ModeKey::Jump, Mode::Jump(JumpMode::new(0)));
        self.modes
            .insert(ModeKey::LineJump, Mode::LineJump(LineJumpMode::new()));
        self.modes
            .insert(ModeKey::LineJump, Mode::LineJump(LineJumpMode::new()));
        self.modes.insert(
            ModeKey::Open,
            Mode::Open(OpenMode::new(
                self.workspace.path.clone(),
                self.preferences.borrow().search_select_config(),
            )),
        );
        self.modes
            .insert(ModeKey::Path, Mode::Path(PathMode::new()));
        self.modes
            .insert(ModeKey::Search, Mode::Search(SearchMode::new(None)));
        self.modes.insert(
            ModeKey::Select,
            Mode::Select(SelectMode::new(Position::default())),
        );
        self.modes.insert(
            ModeKey::SelectLine,
            Mode::SelectLine(SelectLineMode::new(0)),
        );
        self.modes.insert(
            ModeKey::SymbolJump,
            Mode::SymbolJump(SymbolJumpMode::new(
                self.preferences.borrow().search_select_config(),
            )?),
        );
        self.modes.insert(
            ModeKey::Syntax,
            Mode::Syntax(SyntaxMode::new(
                self.preferences.borrow().search_select_config(),
            )),
        );
        self.modes.insert(
            ModeKey::Theme,
            Mode::Theme(ThemeMode::new(
                self.preferences.borrow().search_select_config(),
            )),
        );

        Ok(())
    }
}

fn initialize_preferences() -> Rc<RefCell<Preferences>> {
    Rc::new(RefCell::new(
        Preferences::load().unwrap_or_else(|_| Preferences::new(None)),
    ))
}

fn create_workspace(
    view: &mut View,
    preferences: &Preferences,
    args: &[String],
) -> Result<Workspace> {
    // Discard the executable portion of the argument list.
    let mut path_args = args.iter().skip(1).peekable();

    // Move into an argument-specified directory, if present.
    let initial_dir = env::current_dir()?;
    if let Some(arg) = path_args.peek() {
        let path = Path::new(&arg);

        if path.is_dir() {
            env::set_current_dir(path.canonicalize()?)?;
        }
    }

    let workspace_dir = env::current_dir()?;
    let syntax_path = user_syntax_path()?;
    let mut workspace = Workspace::new(&workspace_dir, syntax_path.as_deref())
        .chain_err(|| WORKSPACE_INIT_FAILED)?;

    // If the first argument was a directory, we've navigated into
    // it; skip it before evaluating file args, lest we interpret
    // it again as a non-existent file and create a buffer for it.
    if workspace_dir != initial_dir {
        path_args.next();
    }

    // Try to open specified files.
    for path_arg in path_args {
        let path = Path::new(&path_arg);

        if path.is_dir() {
            continue;
        }

        // Check if the user has provided any syntax preference for this file.
        // If not, a default one will be applied on calling workspace.add_buffer()
        let syntax_definition = preferences
            .syntax_definition_name(path)
            .and_then(|name| workspace.syntax_set.find_syntax_by_name(&name).cloned());

        // Open the specified path if it exists, or
        // create a new buffer pointing to it if it doesn't.
        let argument_buffer = if path.exists() {
            let mut buffer = Buffer::from_file(path)?;
            buffer.syntax_definition = syntax_definition;

            buffer
        } else {
            let mut buffer = Buffer::new();
            buffer.syntax_definition = syntax_definition;

            // Point the buffer to the path, ensuring that it's absolute.
            if path.is_absolute() {
                buffer.path = Some(path.to_path_buf());
            } else {
                buffer.path = Some(workspace.path.join(path));
            }

            buffer
        };

        workspace.add_buffer(argument_buffer);
        view.initialize_buffer(workspace.current_buffer.as_mut().unwrap())?;
    }

    Ok(workspace)
}

#[cfg(not(test))]
fn user_syntax_path() -> Result<Option<PathBuf>> {
    Preferences::syntax_path().map(Some)
}

// Building/linking user syntaxes is expensive, which is most obvious in the
// test suite, as it creates application instances in rapid succession. Bypass
// these in test and benchmark environments.
#[cfg(test)]
fn user_syntax_path() -> Result<Option<PathBuf>> {
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::preferences::Preferences;
    use super::{Application, Mode, ModeKey};
    use crate::view::View;

    use scribe::Buffer;
    use std::cell::RefCell;
    use std::env;
    use std::path::Path;
    use std::rc::Rc;
    use std::sync::mpsc;
    use yaml_rust::YamlLoader;

    #[test]
    fn application_uses_file_arguments_to_load_contents_into_buffers_when_files_exist() {
        let application =
            Application::new(&vec![String::new(), String::from("Cargo.lock")]).unwrap();
        let buffer = Buffer::from_file(Path::new("Cargo.lock")).unwrap();

        assert_eq!(
            application.workspace.current_buffer.as_ref().unwrap().path,
            buffer.path
        );
        assert_eq!(
            application
                .workspace
                .current_buffer
                .as_ref()
                .unwrap()
                .data(),
            buffer.data()
        );
    }

    #[test]
    fn application_uses_file_arguments_to_create_new_buffers_when_files_do_not_exist() {
        let application =
            Application::new(&vec![String::new(), String::from("non_existent_file")]).unwrap();

        assert_eq!(
            application.workspace.current_buffer.as_ref().unwrap().path,
            Some(env::current_dir().unwrap().join("non_existent_file"))
        );
        assert_eq!(
            application
                .workspace
                .current_buffer
                .as_ref()
                .unwrap()
                .data(),
            ""
        );
    }

    #[test]
    fn create_workspace_correctly_applies_user_defined_syntax_when_opening_buffer_from_command_line(
    ) {
        let data = YamlLoader::load_from_str("types:\n  xyz:\n    syntax: Rust").unwrap();
        let preferences = Rc::new(RefCell::new(Preferences::new(data.into_iter().nth(0))));
        let (event_channel, _) = mpsc::channel();
        let mut view = View::new(preferences.clone(), event_channel.clone()).unwrap();

        let args = vec![String::new(), String::from("src/test.xyz")];
        let workspace = super::create_workspace(&mut view, &preferences.borrow(), &args).unwrap();

        assert_eq!(
            workspace
                .current_buffer
                .as_ref()
                .unwrap()
                .syntax_definition
                .as_ref()
                .unwrap()
                .name,
            "Rust"
        );
    }

    #[test]
    fn switch_to_activates_the_specified_mode() {
        let mut app = Application::new(&Vec::new()).unwrap();

        assert_eq!(app.current_mode, ModeKey::Normal);
        assert!(matches!(app.mode, Mode::Normal));

        app.switch_to(ModeKey::Exit);

        assert_eq!(app.current_mode, ModeKey::Exit);
        assert!(matches!(app.mode, Mode::Exit));
    }

    #[test]
    fn switch_to_retains_state_from_previous_modes() {
        let mut app = Application::new(&Vec::new()).unwrap();

        app.switch_to(ModeKey::Search);
        match app.mode {
            Mode::Search(ref mut s) => s.input = Some(String::from("state")),
            _ => panic!("switch_to didn't change app mode"),
        }

        app.switch_to(ModeKey::Normal);
        app.switch_to(ModeKey::Search);
        match app.mode {
            Mode::Search(ref s) => assert_eq!(s.input, Some(String::from("state"))),
            _ => panic!("switch_to didn't change app mode"),
        }
    }

    #[test]
    fn switch_to_previous_mode_works() {
        let mut app = Application::new(&Vec::new()).unwrap();

        app.switch_to(ModeKey::Insert);
        app.switch_to(ModeKey::Exit);

        assert_eq!(app.current_mode, ModeKey::Exit);
        assert!(matches!(app.mode, Mode::Exit));

        app.switch_to_previous_mode();

        assert_eq!(app.current_mode, ModeKey::Insert);
        assert!(matches!(app.mode, Mode::Insert));
    }

    #[test]
    fn switch_to_handles_switching_to_current_mode() {
        let mut app = Application::new(&Vec::new()).unwrap();

        app.switch_to(ModeKey::Insert);

        assert_eq!(app.current_mode, ModeKey::Insert);
        assert!(matches!(app.mode, Mode::Insert));

        app.switch_to(ModeKey::Insert);

        assert_eq!(app.current_mode, ModeKey::Insert);
        assert!(matches!(app.mode, Mode::Insert));
    }
}
