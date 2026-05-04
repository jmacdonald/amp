use super::super::{LexemeMapper, MappedLexeme};
use super::test_harness::{theme_with_backgrounds, RendererHarness};
use super::RenderState;
use crate::view::{Colors, RGBColor, Style};
use scribe::buffer::{Position, Range};
use scribe::Buffer;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

#[test]
fn tabs_beyond_terminal_width_dont_panic() {
    let mut buffer = Buffer::new();
    buffer.insert("\t\t\t");
    let mut harness = RendererHarness::with_options(buffer, Some("tab_width: 100"), None, None);

    harness.render(0, None);
}

#[test]
fn aligned_tabs_expand_to_correct_number_of_spaces() {
    let mut buffer = Buffer::new();
    buffer.insert("\t\txy");
    let mut harness = RendererHarness::with_options(buffer, Some("tab_width: 2"), None, None);
    let (content, _) = harness.render(0, None);

    let expected_content = " 1      xy";
    assert_eq!(&content[0..expected_content.len()], expected_content);
}

#[test]
fn unaligned_tabs_expand_to_correct_number_of_spaces() {
    let mut buffer = Buffer::new();
    buffer.insert("\t \txy");
    let mut harness = RendererHarness::with_options(buffer, Some("tab_width: 2"), None, None);
    let (content, _) = harness.render(0, None);

    let expected_content = " 1      xy";
    assert_eq!(&content[0..expected_content.len()], expected_content);
}

#[test]
fn render_wraps_lines_correctly() {
    let mut buffer = Buffer::new();
    buffer.insert("amp editor\nsecond line\n");
    let mut harness = RendererHarness::new(buffer);
    let (content, _) = harness.render(0, None);

    let expected_content = " 1  amp ed\n    itor  \n 2  second\n     line \n 3        ";
    assert_eq!(&content[0..expected_content.len()], expected_content);
}

struct TestMapper {}

impl LexemeMapper for TestMapper {
    fn map<'a, 'b>(&'a mut self, _: &str, _: Position) -> Vec<MappedLexeme<'a>> {
        vec![MappedLexeme::Focused("mapped")]
    }
}

#[test]
fn render_uses_lexeme_mapper() {
    let mut buffer = Buffer::new();
    buffer.insert("original");
    let mut harness = RendererHarness::new(buffer);
    let (content, _) = harness.render_with_mapper(0, None, Some(&mut TestMapper {}));

    let expected_content = " 1  mapped";
    assert_eq!(&content[0..expected_content.len()], expected_content);
}

#[test]
fn render_returns_cursor_position_when_at_the_start_of_an_empty_line() {
    let mut buffer = Buffer::new();
    buffer.insert("\n");
    let mut harness = RendererHarness::new(buffer);
    let (_, cursor_position) = harness.render(0, None);

    assert_eq!(cursor_position, Some(Position { line: 0, offset: 4 }));
}

#[test]
fn render_caches_state_using_correct_frequency_excluding_first_line() {
    let mut buffer = Buffer::new();
    for _ in 0..500 {
        buffer.insert("line\n");
    }
    let render_cache = Rc::new(RefCell::new(HashMap::<usize, RenderState>::new()));
    let mut harness = RendererHarness::with_options(buffer, None, None, Some(render_cache));

    harness.render(495, None);

    assert_eq!(harness.render_cache.borrow().keys().count(), 5);
}

#[test]
fn render_uses_cached_state() {
    let mut buffer = Buffer::new();
    buffer.path = Some(PathBuf::from("test.rs"));
    for _ in 0..500 {
        buffer.insert("line\n");
    }
    let render_cache = Rc::new(RefCell::new(HashMap::<usize, RenderState>::new()));
    let mut harness = RendererHarness::with_options(buffer, None, None, Some(render_cache));

    harness.render(95, None);

    assert_eq!(harness.render_cache.borrow().keys().count(), 1);
    let initial_cache = harness
        .render_cache
        .borrow()
        .values()
        .nth(0)
        .unwrap()
        .clone();

    harness.current_buffer_mut().insert("\"");
    harness.render(495, None);

    assert_eq!(harness.render_cache.borrow().keys().count(), 5);
    for value in harness.render_cache.borrow().values() {
        assert_eq!(value, &initial_cache);
    }
}

#[test]
fn render_skips_lines_correctly_when_using_cached_state() {
    let mut buffer = Buffer::new();
    buffer.path = Some(PathBuf::from("test.rs"));
    for _ in 0..203 {
        buffer.insert("line\n");
    }
    let render_cache = Rc::new(RefCell::new(HashMap::<usize, RenderState>::new()));
    let mut harness = RendererHarness::with_options(buffer, None, None, Some(render_cache));

    harness.render(95, None);

    assert_eq!(harness.render_cache.borrow().keys().count(), 1);
    harness.clear_terminal();
    harness.current_buffer_mut().insert("\"");
    let (content, _) = harness.render(200, None);

    let expected_content = " 201  line\n 202  line\n 203  line\n 204      ";
    assert_eq!(&content[0..expected_content.len()], expected_content);
}

#[test]
fn current_char_style_uses_line_highlight_when_token_background_matches_theme_on_cursor_line() {
    let mut buffer = Buffer::new();
    buffer.insert("x");
    let mut harness =
        RendererHarness::with_options(buffer, None, Some(theme_with_backgrounds()), None);
    let rendered = harness.render_styled_char(RGBColor(1, 2, 3), RGBColor(0x22, 0x33, 0x44), None);

    assert_eq!(
        rendered,
        (
            Style::Default,
            Colors::Custom(RGBColor(1, 2, 3), RGBColor(0x33, 0x44, 0x55))
        )
    );
}

#[test]
fn current_char_style_preserves_explicit_token_background_on_cursor_line() {
    let mut buffer = Buffer::new();
    buffer.insert("x");
    let mut harness =
        RendererHarness::with_options(buffer, None, Some(theme_with_backgrounds()), None);
    let rendered = harness.render_styled_char(RGBColor(1, 2, 3), RGBColor(9, 8, 7), None);

    assert_eq!(
        rendered,
        (
            Style::Default,
            Colors::Custom(RGBColor(1, 2, 3), RGBColor(9, 8, 7))
        )
    );
}

#[test]
fn current_char_style_preserves_theme_background_off_cursor_line() {
    let mut buffer = Buffer::new();
    buffer.insert("x");
    *buffer.cursor = Position { line: 1, offset: 0 };
    let mut harness =
        RendererHarness::with_options(buffer, None, Some(theme_with_backgrounds()), None);
    let rendered = harness.render_styled_char(RGBColor(1, 2, 3), RGBColor(0x22, 0x33, 0x44), None);

    assert_eq!(
        rendered,
        (
            Style::Default,
            Colors::Custom(RGBColor(1, 2, 3), RGBColor(0x22, 0x33, 0x44))
        )
    );
}

#[test]
fn current_char_style_selection_overrides_token_backgrounds() {
    let mut buffer = Buffer::new();
    buffer.insert("x");
    let highlights = vec![Range::new(
        Position { line: 0, offset: 0 },
        Position { line: 0, offset: 1 },
    )];
    let mut harness =
        RendererHarness::with_options(buffer, None, Some(theme_with_backgrounds()), None);
    let rendered =
        harness.render_styled_char(RGBColor(1, 2, 3), RGBColor(9, 8, 7), Some(&highlights));

    assert_eq!(rendered, (Style::Bold, Colors::SelectMode));
}

#[test]
fn current_char_style_uses_terminal_background_when_theme_background_matches_off_cursor_line() {
    let mut buffer = Buffer::new();
    buffer.insert("x");
    *buffer.cursor = Position { line: 1, offset: 0 };
    let mut harness = RendererHarness::with_options(
        buffer,
        Some("transparent_background: true"),
        Some(theme_with_backgrounds()),
        None,
    );
    let rendered = harness.render_styled_char(RGBColor(1, 2, 3), RGBColor(0x22, 0x33, 0x44), None);

    assert_eq!(
        rendered,
        (Style::Default, Colors::CustomForeground(RGBColor(1, 2, 3)))
    );
}

#[test]
fn current_char_style_uses_line_highlight_when_theme_background_matches_on_cursor_line_and_transparency_enabled(
) {
    let mut buffer = Buffer::new();
    buffer.insert("x");
    let mut harness = RendererHarness::with_options(
        buffer,
        Some("transparent_background: true"),
        Some(theme_with_backgrounds()),
        None,
    );
    let rendered = harness.render_styled_char(RGBColor(1, 2, 3), RGBColor(0x22, 0x33, 0x44), None);

    assert_eq!(
        rendered,
        (
            Style::Default,
            Colors::CustomFocusedForeground(RGBColor(1, 2, 3))
        )
    );
}
