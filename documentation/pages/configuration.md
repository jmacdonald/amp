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

For setting options for common files _without_ extensions, use a file name:

```yaml
types:
  Makefile:
    tab_width: 4
    soft_tabs: false
```

## Key Bindings

In Amp, key bindings are simple key/command associations, scoped to a specific mode. You can define custom key bindings by defining a keymap in your preferences file:

```yaml
keymap:
  normal:
    j: "cursor::move_down"
```

!!! tip
    Wondering where to find command names? You can view the full list in a new buffer by running `application::display_available_commands` using [command mode](usage.md#running-commands). You can also view Amp's default key bindings by running `application::display_default_keymap`.

### Modifiers

Amp supports qualifying key bindings with a `ctrl` modifier:

```yaml
keymap:
  normal:
    ctrl-s: "buffer::save"
```

### Wildcards

You can also use wildcards in key bindings:

```yaml
keymap:
  normal:
    _: "buffer::insert_char"
```

More specific key bindings will override wildcard values, making them useful as a fallback value:

```
   ...
    _: "buffer::insert_char"
    s: "buffer::save"
```

### Multiple Commands

You can also pass a collection of commands to run. Amp will run all of the commands in order, stopping if/when any errors occur:

```yaml
keymap:
  normal:
    v:
      - "application::switch_to_select_mode"
      - "application::switch_to_jump_mode"
```

!!! tip
    It may not be readily apparent, but chaining commands like this is powerful. A significant portion of Amp's functionality is
    built by composing multiple commands into larger, more complex ones.

## Format/Language Support

Most popular formats/languages have syntax highlighting and symbol support out of the box. If you're editing a file that doesn't, you can extend the built-in set with a custom syntax definition. Amp uses Sublime Text's [`.sublime-syntax`](https://www.sublimetext.com/docs/3/syntax.html) files, which can be placed in Amp's `syntaxes` configuration subdirectory.

!!! tip
    If you're not sure where to look, run the `preferences::edit` command.
    The preferences will load into a new buffer for editing, and its path
    will be shown at the bottom of the screen; the `syntaxes` subdirectory is in
    the same directory as that file.

## Themes

Amp includes [Solarized](http://ethanschoonover.com/solarized) dark and light themes by default. You can extend the built-in set with custom themes of your own. Amp uses Text Mate's `.tmTheme` format, many of which can be found [here](http://wiki.macromates.com/Themes/UserSubmittedThemes). They should be placed in Amp's `themes` configuration subdirectory.

!!! tip
    If you're not sure where to look, run the `preferences::edit` command.
    The preferences will load into a new buffer for editing, and its path
    will be shown at the bottom of the screen; the `themes` subdirectory is in
    the same directory as that file.

## Open Mode

### Excluding Files/Directories

Using Unix shell-style glob patterns, Amp's file finder can be configured to exclude files and directories:

```yaml
open_mode:
  exclusions:
    - "**/.git"
    - "**/.svn"
```

You can also opt out of exclusions altogether by setting the value to `false`:

```yaml
open_mode:
  exclusions: false
```
