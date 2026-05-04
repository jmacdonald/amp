use super::{BufferRenderer, LexemeMapper, RenderState, ThemeStyle};
use crate::models::application::Preferences;
use crate::view::terminal::{build_terminal, Terminal, TerminalBuffer};
use crate::view::{Colors, RGBColor, Style};
use scribe::buffer::{Buffer, Position, Range};
use scribe::util::LineIterator;
use scribe::Workspace;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use syntect::highlighting::{Color, FontStyle, Theme, ThemeSet, ThemeSettings};

type SharedRenderCache = Rc<RefCell<HashMap<usize, RenderState>>>;
type TerminalHandle = Arc<Box<dyn Terminal + Sync + Send + 'static>>;

pub(super) struct RendererHarness {
    pub(super) preferences: Preferences,
    pub(super) render_cache: SharedRenderCache,
    terminal: TerminalHandle,
    theme: Theme,
    workspace: Workspace,
}

impl RendererHarness {
    pub(super) fn new(buffer: Buffer) -> RendererHarness {
        RendererHarness::with_options(buffer, None, None, None)
    }

    pub(super) fn with_options(
        buffer: Buffer,
        preferences_yaml: Option<&str>,
        theme: Option<Theme>,
        render_cache: Option<SharedRenderCache>,
    ) -> RendererHarness {
        let mut workspace = Workspace::new(Path::new(".")).unwrap();
        workspace.add_buffer(buffer);

        let preferences = Preferences::new(preferences_yaml.and_then(|yaml| {
            yaml_rust::YamlLoader::load_from_str(yaml)
                .unwrap()
                .into_iter()
                .next()
        }));
        let theme =
            theme.unwrap_or_else(|| ThemeSet::load_defaults().themes["base16-ocean.dark"].clone());

        RendererHarness {
            preferences,
            render_cache: render_cache.unwrap_or_else(|| Rc::new(RefCell::new(HashMap::new()))),
            terminal: build_terminal().unwrap(),
            theme,
            workspace,
        }
    }

    pub(super) fn current_buffer_mut(&mut self) -> &mut Buffer {
        self.workspace.current_buffer.as_mut().unwrap()
    }

    fn current_buffer(&self) -> &Buffer {
        self.workspace.current_buffer.as_ref().unwrap()
    }

    pub(super) fn render(
        &mut self,
        scroll_offset: usize,
        highlights: Option<&[Range]>,
    ) -> (String, Option<Position>) {
        self.render_with_mapper(scroll_offset, highlights, None)
    }

    pub(super) fn render_with_mapper(
        &mut self,
        scroll_offset: usize,
        highlights: Option<&[Range]>,
        lexeme_mapper: Option<&mut dyn LexemeMapper>,
    ) -> (String, Option<Position>) {
        let data = self.current_buffer().data();
        let lines = LineIterator::new(&data);
        let mut terminal_buffer = TerminalBuffer::new(self.terminal.width(), self.terminal.height());

        let cursor_position = BufferRenderer::new(
            self.current_buffer(),
            highlights,
            scroll_offset,
            &**self.terminal,
            &self.theme,
            &self.preferences,
            &self.render_cache,
            &self.workspace.syntax_set,
            &mut terminal_buffer,
        )
        .render(lines, lexeme_mapper)
        .unwrap();

        (terminal_buffer.content(), cursor_position)
    }

    pub(super) fn render_styled_char(
        &mut self,
        token_fg: RGBColor,
        token_bg: RGBColor,
        highlights: Option<&[Range]>,
    ) -> (Style, Colors) {
        let mut terminal_buffer = TerminalBuffer::new(self.terminal.width(), self.terminal.height());
        let mut renderer = BufferRenderer::new(
            self.current_buffer(),
            highlights,
            0,
            &**self.terminal,
            &self.theme,
            &self.preferences,
            &self.render_cache,
            &self.workspace.syntax_set,
            &mut terminal_buffer,
        );
        renderer.current_style = ThemeStyle {
            foreground: Color {
                r: token_fg.0,
                g: token_fg.1,
                b: token_fg.2,
                a: 0xFF,
            },
            background: Color {
                r: token_bg.0,
                g: token_bg.1,
                b: token_bg.2,
                a: 0xFF,
            },
            font_style: FontStyle::empty(),
        };
        renderer.print_line_number();
        renderer.print_lexeme("x");

        terminal_buffer
            .iter()
            .find(|(_, cell)| cell.content == "x")
            .map(|(_, cell)| (cell.style, cell.colors))
            .unwrap()
    }

    pub(super) fn clear_terminal(&self) {
        self.terminal.clear();
    }
}

pub(super) fn theme_with_backgrounds() -> Theme {
    Theme {
        name: Some(String::from("Test Theme")),
        settings: ThemeSettings {
            foreground: Some(Color {
                r: 0x11,
                g: 0x22,
                b: 0x33,
                a: 0xFF,
            }),
            background: Some(Color {
                r: 0x22,
                g: 0x33,
                b: 0x44,
                a: 0xFF,
            }),
            line_highlight: Some(Color {
                r: 0x33,
                g: 0x44,
                b: 0x55,
                a: 0xFF,
            }),
            ..ThemeSettings::default()
        },
        ..Theme::default()
    }
}
