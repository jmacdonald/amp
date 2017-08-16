# Configuration

Amp uses a YAML file to define preferences that sit in a platform-dependent configuration folder. The easiest way to edit these is to use the built-in `preferences::edit` command, which can be run in command mode. There's a corresponding `reload` command, too, if you persist any changes.

!!! tip
    If you want to version this file, the aforementioned `edit` command will
    display the full path at the bottom of the screen once the preferences have
    been loaded into a new buffer for editing.

## General Options

### Theme

```yaml
theme: solarized_dark
```

Used to specify the default theme. Values can be located through Amp's theme mode.

!!! tip
    You can configure the current theme without making a permanent configuration
    change. Hit `t` to pick a theme that'll only last until you close the editor.
    It's handy for temporarily changing to a lighter theme when working outdoors,
    or vice-versa.

### Tab Width

```yaml
tab_width: 2
```

Determines the visual width of tab characters, and when `soft_tabs` is `true`, determines the number of spaces to insert when a soft tab is inserted.

### Soft Tabs

```yaml
soft_tabs: true
```

This setting configures the type of tabs used in insert mode.
See: the infamous tabs vs. spaces debate.

### Line Length Guide

```yaml
line_length_guide: 80
```

When set to a positive integer, this renders a background vertical line at the specified offset, to guide line length. When set to `false`, the guide is hidden.


### Line Wrapping

```yaml
line_wrapping: true
```

When set to `true`, lines extending beyond the visible region are wrapped to the line below.

## File Format-Specific Options

The `tab_width` and `soft_tabs` options can be configured on a per-extension basis:

```yaml
types:
  rs:
    tab_width: 4
    soft_tabs: true
  go:
    tab_width: 8
    soft_tabs: false
```

## Key Bindings

In Amp, key bindings are simple key/command associations, targeted for a specific mode. You can define custom key bindings by defining a keymap in your preferences file:

```yaml
keymap:
  normal:
    j: "cursor::move_down"
```

Amp has quite a few built-in commands.

### Modifiers

```yaml
keymap:
  normal:
    ctrl-s: "buffer::save"
```

Amp supports qualifying key bindings with a `ctrl` modifier.

### Wildcards

```yaml
keymap:
  normal:
    _: "buffer::insert_char"
```

You can also use wildcards in key bindings. More specific key bindings will override wildcard values, making them useful as a fallback value:

```
   ...
    _: "buffer::insert_char"
    s: "buffer::save"
```

## Format/Language Support

Most popular formats/languages have syntax highlighting and symbol support out of the box. If you're editing a file that doesn't, you can extend the built-in set with a custom syntax definition. Amp uses Sublime Text's [`.sublime-syntax`](https://www.sublimetext.com/docs/3/syntax.html) files, which can be placed in Amp's `syntaxes` configuration subdirectory.

!!! tip
    If you're not sure where to look, run the `preferences::edit` command.
    The preferences will load into a new buffer for editing, and its path
    will be shown at the bottom of the screen; the `syntaxes` subdirectory is in
    the same directory as that file.
