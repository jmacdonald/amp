### 0.7.0

* Lots of under-the-hood improvements to syntax highlighting, as well as:
  * Replaced Oniguruma regex dependency with Rust-based fancy-regex
  * Better error handling
* Improved terminal renderer error handling
* Cursor types now change based on mode
* Added ability to build amp using Nix
* Added Wayland clipboard support
* Updated to build using Rust 2021 edition
* Added ability to reflow text based on line length guide
* Improved unicode support when adding trailing newline
* Added ability to configure syntax overrides based on file extension
* Added ability to comment out a selection of text
* Improved handling of very small terminal sizes
* Updated to correctly restore terminal contents after quitting

### 0.6.2

* Rewrote the build script's command parsing to work with Rust 1.41
  * See https://github.com/jmacdonald/amp/issues/173 for details

### 0.6.1

* Added the ability to choose a syntax definition for the current buffer
* Updated `git2` dependency
* Disabled unused `git2` features, removing transitive openssl dependency
* Fixed an issue where tabs were ignored when removing trailing whitespace (thanks, BearOve!)
* Specified Rust 1.31.1 (2018 Edition) as the minimum required version

### 0.6.0

* Added more information to quick start guide (thanks, John M-W!)
* Added snapcraft build file (thanks, Alan Pope!)
* Added proper delete key support (thanks, Jérôme Martin!)
* Added https support to GitHub URL generation command (thanks, king6cong!)
* Use a vendored version of OpenSSL (thanks, Cecile Tonglet!)
* Fixed buffer::outdent_line when using hard tabs (thanks, Gaby!)
* Fixed an issue where user syntax definitions were loaded after argument buffers (#122)
* Update to compile with Rust 2018 edition
* Added keybindings to support jumping directly into symbol and open modes from search mode
* Handle missing themes gracefully, falling back to default (#149)
* Migrate from termbox library to termion
  * Removes termbox's build process Python dependency
  * Adds 24-bit colour support
  * Built a TerminalBuffer to allow successive screen updates within a single
    render cycle without introducing screen flicker.
    * Improves support for UTF-8 grapheme clusters.
      * Since termbox uses 32-bit char values to represent cells, anything larger
        than 32 bits, even if spilled over into adjacent cells, will be overwritten
        by adjacent characters. The new TerminalBuffer type uses Cow<&str> values,
        allowing arbitrary-length cells, which will be streamed to the terminal
        and overlaid as a single visible "character", without any loss of data.
    * Created a new Presenter type to hold the contents of this buffer, as well
      as extract common bits of functionality from various mode-specific presenters.

### 0.5.2

* Fixed a regression that would raise an error when trying to open Amp with a
  new file argument
  * See https://github.com/jmacdonald/amp/issues/112 for details

### 0.5.1

* Added ability to open Amp in another directory via `amp path/to/directory`
* Improved newline indentation heuristics
  * See https://github.com/jmacdonald/amp/issues/103 for details
* Added `>` prefix and bold style to selection in search/select mode
  * See https://github.com/jmacdonald/amp/issues/106 for details
* Amp will now refresh its syntax definition after a buffer's path is changed
  * See https://github.com/jmacdonald/amp/issues/97 for details
* Added a quick start guide, referenced from the splash page
* Added suspend command key binding to search/select normal mode
* Added the ability to configure number of results in search/select mode
  * See https://amp.rs/docs/configuration/#searchselect-results for details
* Updated `termbox-sys` dependency, which fixes `.termbox already exists` build errors
  * See https://github.com/gchp/termbox-sys/issues/11 for details

### 0.5.0

* Added caching to syntax highlighting, to improve performance for large buffers
  * See https://medium.com/@jordan_98525/incremental-parsing-in-amp-ba5e8c3e85dc for details

### 0.4.1

* Fixed syntax highlighting
  * Scopes were bleeding into one another; we now defer to HighlightIterator
  * See https://github.com/jmacdonald/amp/issues/22 for details

### 0.4.0

* Application event loop is now threaded
  * Most notably, open mode indexing is now run in a separate thread
* Scrolling is now line wrap-aware
* View now redraws when terminal is resized
* Search/select modes now have empty state messages
  * e.g. open mode will now display "Enter a query" rather than "No results" when no query is present
* Open mode now displays its path when indexing
* Escape in normal mode now scrolls cursor to center, via new default keybinding
* app_dirs dependency bumped to a version that compiles on newer versions of Rust
* Type-specific configuration now supports full filenames (e.g. "Makefile")
* Various refactoring

### 0.3.4

* Documentation updates
* Added the ability to save new buffers without paths (created in normal mode
  using the `B` key binding); a new "path" mode prompts before saving.
* Added the ability to load user/custom themes from the `themes` configuration
  sub-directory
* Added a benchmark for buffer rendering
* Bumped native clipboard library dependency
* Added semi-colon delete key binding to select line mode

### 0.3.3

* Documentation updates
* buffer::backspace command no longer switches to insert mode
  (this is relegated to the default keymap)
* Invalid keymap configurations now display the offending mode

### 0.3.2

* Fix case-insensitive open mode search with uppercase characters
* Add class and struct identifiers to symbol mode whitelist
* Documentation and README updates

### 0.3.1

* Bumped copyright year to 2018
* Updated CI config to run on stable release channel
* Documentation site and content updates
* Added `application::display_default_keymap` command
* Added ability to delete current result in search mode

### 0.3.0

* Switched to Rust stable release channel
* New command mode (run any built-in commands through a search/select UI)
* User-defined preferences, syntaxes, and keymaps
* New confirm mode, applied primarily to closing or reloading buffers
* New command to view syntax scope at cursor
* Extracted all logic from input handlers
* Migrated input handling to simple key => command mappings
* New select_all command
* Updated native clipboard library


### 0.2.0

* Added theme selection mode
* Quality improvements to command error reporting
* Updated search mode to better handle "no matches" state
* Treat hash/pound symbol as delimeter when using word-based movement
* Added initial preference implementation
* Under the hood improvements to search/select modes (open, symbol, theme, etc.)
* Updated search/select modes to perform case insensitive searches

### 0.1.0

* Initial release
* Added proper error handling to all commands
* Updated main loop to display command errors
