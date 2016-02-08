[![Build Status](https://travis-ci.org/jmacdonald/amp.svg?branch=master)](https://travis-ci.org/jmacdonald/amp)

# Amp: A text editor for your terminal.

Heavily inspired by Vi/Vim. Amp aims to take the core interaction model of Vim,
simplify it, and bundle in the essential features required for a modern text
editor.

Written with :heart: in [Rust](http://rust-lang.org).

# Features

* Fast file finder
* Syntax highlighting
* UTF-8 support
* Native clipboard integration
* Basic Git integration

# Usage

`amp [file1] [file2] ...`

# Modes

## <a id="normal_mode">Normal Mode</a>

[Key Bindings](https://github.com/jmacdonald/amp/tree/master/src/input/modes/normal.rs)

Amp starts in normal mode, which is used to switch between and move within buffers. It also acts as a jumping point for other modes, which provide more specialized actions.

## <a id="open_mode">Open Mode</a>

[Key Bindings](https://github.com/jmacdonald/amp/tree/master/src/input/modes/open.rs)

Indexes the current path (and its subdirectories) to allow opening new buffers.

### <a id="fragment_matching">Fragment Matching</a>

In place of fuzzy matching, Amp uses fragment matching. When used properly, it
returns accurate matches across large projects/codebases with minimal typing.

Instead of typing full words, use fragments of the path name, separated by
spaces. Search terms _must_ occur in the path. The order of tokens doesn't matter: you can add fragments from parent directory names after file name fragments.

#### Examples

Target path: `src/models/applications/modes/open.rs`  
Search terms: `mod app op`

Target path: `src/input/modes/jump.rs`  
Search terms: `inp j`

Target path: `src/view/scrollable_region.rs`  
Search terms: `scrol`

Hitting `backspace` will delete the entire last token. Given the size of the tokens, it's almost always easier to re-enter the last entry than to correct it.

## Insert Mode

[Key Bindings](https://github.com/jmacdonald/amp/tree/master/src/input/modes/insert.rs)

There's nothing too special about insert mode. With the exception of the arrow, page up/down, and home/end keys, which are used for navigation, all other keystrokes simply insert text at the cursor position.

## <a id="jump_mode">Jump Mode</a>

[Key Bindings](https://github.com/jmacdonald/amp/tree/master/src/input/modes/jump.rs)

If you've used Vim's [EasyMotion plug-in](https://github.com/easymotion/vim-easymotion), Amp's jump mode will be instantly familiar.

With the exception of basic left/right/up/down and word-based cursor movement, jump mode is Amp's primary means of navigation. When in [normal mode](#normal_mode), press `f` to enter jump mode. All of words on-screen will be prefixed with two-character jump tokens. Type two characters to move the cursor to the corresponding token.

![jump mode](https://raw.githubusercontent.com/jmacdonald/amp/master/doc/jump_mode.gif)

## Line Jump Mode

[Key Bindings](https://github.com/jmacdonald/amp/tree/master/src/input/modes/line_jump.rs)

Especially useful when dealing with line-specific compiler/interpreter errors. Type `g` to enter line jump mode, enter the line to which you'd like to jump, and hit `Enter`.

## Symbol Jump Mode

[Key Bindings](https://github.com/jmacdonald/amp/tree/master/src/input/modes/symbol_jump.rs)

In [supported languages](https://github.com/jmacdonald/luthor/tree/master/src/lexers), Amp will search for method and function definition tokens. Type `space` to enter symbol jump mode. Much like [open mode](#open_mode), symbol jump mode uses [fragment matching](#fragment_matching) to search the symbol list. Hitting `Enter` on any of the matches will move the cursor to that symbol.

## Select Mode

[Key Bindings](https://github.com/jmacdonald/amp/tree/master/src/input/modes/select.rs)

Type `v` to enter select mode, after which you can use movement commands to extend the selected range. Select mode fully supports [jump mode](#jump_mode) for cursor movement.

## Select Line Mode

[Key Bindings](https://github.com/jmacdonald/amp/tree/master/src/input/modes/select_line.rs)

Enables selection of complete lines of text. Type `V` to enter select line mode, after which you can use movement commands to extend the selected range. Select line mode fully supports [jump mode](#jump_mode) for cursor movement.

## Search Mode

[Key Bindings](https://github.com/jmacdonald/amp/tree/master/src/input/modes/search.rs)

Used to search for terms in the current buffer. Type `/` to enter search mode, type a search term, and hit `Enter` to jump to the closest result. Once a search has been performed, Amp returns to normal mode. The `n` and `p` keys can be used to jump to the next and previous results, respectively.
