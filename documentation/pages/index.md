# Overview

Amp is inspired by [Vim](https://vim.sourceforge.io)'s modal approach to
text editing, which is reflected in several of its default key bindings.
That similarity aside, there are several key differences.

Above all else, Amp aims to _keep things as simple as possible_. There are
already plenty of highly-configurable editors available. At its core, Amp aims
to minimize configuration and provide a great out-of-the-box experience. The
following sections describe some of the ideas central to the design of Amp that
differentiate it from other options.

### UX

Like Vim, Amp is a modal editor: keystrokes perform different functions based
on the current mode. Many familiar modes (insert, normal, select, etc.) are
available, as well as several new ones providing additional functionality.

### Essential Features

Amp's primary audience is _developers_.

Syntax highlighting, a fuzzy file finder, local symbol jump, and basic Git
integration are available without additional configuration or external
dependencies (e.g. plug-ins, ctags, external indexing binaries).

### Configuration

Amp shouldn't require any initial configuration. User preferences live in a
single YAML file and have sensible defaults. There's also a built-in command to
easily edit this file without having to leave the application.

### Considerations

Although still in its infancy, Amp is suitable for day-to-day use, with a few
exceptions. There are features not yet in place; some are planned, others are not.

##### Encoding

Amp only supports UTF-8 (and by proxy, ASCII). Supporting other encoding types
is not planned. Windows line endings (CRLF) are also currently unsupported.

##### Split Panes

Unlike Vim, Amp doesn't provide split panes, and support isn't planned. It's
recommended to use [tmux](https://github.com/tmux/tmux/wiki) instead, which
provides this (and much more) for your shell, text editor, and any other
terminal-based applications you may use.

##### Plug-ins

Many editors allow users to extend and change much of their behaviour through
the use of plug-ins. This is not a goal for Amp. However, in spite of its focus
on being "complete" from the start, an avenue for extended language, framework,
and workflow support is necessary. Features like "go to definition" require
non-trivial language support, and are great candidates for plug-ins.

As a result, _Amp will eventually include a runtime and plug-in API_, providing
the ability to define new commands.
