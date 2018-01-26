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
