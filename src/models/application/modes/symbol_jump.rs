use crate::errors::*;
use crate::models::application::modes::{SearchSelectConfig, SearchSelectMode};
use crate::util::SelectableVec;
use fragment;
use fragment::matching::AsStr;
use scribe::buffer::{Position, Token, TokenSet};
use std::clone::Clone;
use std::fmt;
use std::iter::Iterator;
use std::slice::Iter;
use std::str::FromStr;
use syntect::highlighting::ScopeSelectors;

pub struct SymbolJumpMode {
    insert: bool,
    input: String,
    symbols: Vec<Symbol>,
    results: SelectableVec<Symbol>,
    config: SearchSelectConfig,
}

#[derive(PartialEq, Debug)]
pub struct Symbol {
    pub token: String,
    pub position: Position,
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.token)
    }
}

impl Clone for Symbol {
    fn clone(&self) -> Symbol {
        Symbol {
            token: self.token.clone(),
            position: self.position,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.token = source.token.clone();
        self.position = source.position;
    }
}

impl AsStr for Symbol {
    fn as_str(&self) -> &str {
        &self.token
    }
}

impl SymbolJumpMode {
    pub fn new(config: SearchSelectConfig) -> Result<SymbolJumpMode> {
        Ok(SymbolJumpMode {
            insert: true,
            input: String::new(),
            symbols: Vec::new(),
            results: SelectableVec::new(Vec::new()),
            config,
        })
    }

    pub fn reset(&mut self, tokens: &TokenSet, config: SearchSelectConfig) -> Result<()> {
        self.insert = true;
        self.input.clear();
        self.symbols = symbols(tokens.iter().chain_err(|| BUFFER_PARSE_FAILED)?);
        self.results = SelectableVec::new(Vec::new());
        self.config = config;

        Ok(())
    }
}

impl fmt::Display for SymbolJumpMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SYMBOL")
    }
}

impl SearchSelectMode<Symbol> for SymbolJumpMode {
    fn search(&mut self) {
        // Find the symbols we're looking for using the query.
        let results = fragment::matching::find(&self.input, &self.symbols, self.config.max_results);

        // We don't care about the result objects; we just want
        // the underlying symbols. Map the collection to get these.
        self.results = SelectableVec::new(results.into_iter().map(|r| r.clone()).collect());
    }

    fn query(&mut self) -> &mut String {
        &mut self.input
    }

    fn insert_mode(&self) -> bool {
        self.insert
    }

    fn set_insert_mode(&mut self, insert_mode: bool) {
        self.insert = insert_mode;
    }

    fn results(&self) -> Iter<Symbol> {
        self.results.iter()
    }

    fn selection(&self) -> Option<&Symbol> {
        self.results.selection()
    }

    fn selected_index(&self) -> usize {
        self.results.selected_index()
    }

    fn select_previous(&mut self) {
        self.results.select_previous();
    }

    fn select_next(&mut self) {
        self.results.select_next();
    }

    fn config(&self) -> &SearchSelectConfig {
        &self.config
    }
}

fn symbols<'a, T>(tokens: T) -> Vec<Symbol>
where
    T: Iterator<Item = Token<'a>>,
{
    let eligible_scopes =
        ScopeSelectors::from_str("entity.name.function, entity.name.class, entity.name.struct")
            .unwrap();
    tokens
        .filter_map(|token| {
            if let Token::Lexeme(lexeme) = token {
                // Build a symbol, provided it's of the right type.
                if eligible_scopes
                    .does_match(lexeme.scope.as_slice())
                    .is_some()
                {
                    return Some(Symbol {
                        token: lexeme.value.to_string(),
                        position: lexeme.position,
                    });
                }
            }

            None
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::SymbolJumpMode;
    use super::{symbols, Symbol};
    use crate::models::application::modes::{SearchSelectConfig, SearchSelectMode};
    use crate::models::application::Application;
    use scribe::buffer::{Lexeme, Position, ScopeStack, Token};
    use std::path::Path;
    use std::str::FromStr;

    #[test]
    fn symbols_are_limited_to_functions() {
        let tokens = vec![
            Token::Lexeme(Lexeme {
                value: "text",
                position: Position { line: 0, offset: 0 },
                scope: ScopeStack::from_str("meta.block.rust").unwrap(),
            }),
            Token::Lexeme(Lexeme {
                value: "function",
                position: Position { line: 1, offset: 0 },
                scope: ScopeStack::from_str("entity.name.function").unwrap(),
            }),
            Token::Lexeme(Lexeme {
                value: "non-function",
                position: Position { line: 2, offset: 0 },
                scope: ScopeStack::from_str("meta.entity.name.function").unwrap(),
            }),
        ];

        let results = symbols(tokens.into_iter());
        assert_eq!(results.len(), 1);
        assert_eq!(
            results.first().unwrap(),
            &Symbol {
                token: "function".to_string(),
                position: Position { line: 1, offset: 0 }
            }
        );
    }

    #[test]
    fn reset_clears_query_mode_and_results() {
        let config = SearchSelectConfig::default();
        let mut mode = SymbolJumpMode::new(config.clone()).unwrap();
        let mut app = Application::new(&[]).unwrap();
        app.workspace.open_buffer(&Path::new("build.rs")).unwrap();
        let token_set = app.workspace.current_buffer_tokens().unwrap();

        // Do an initial reset to get the results populated
        mode.reset(&token_set, config.clone()).unwrap();
        mode.query().push_str("main");
        mode.set_insert_mode(false);
        mode.search();

        // Ensure we have results before reset
        assert!(mode.results.len() > 0);

        mode.reset(&token_set, config).unwrap();
        assert_eq!(mode.query(), "");
        assert_eq!(mode.insert_mode(), true);
        assert_eq!(mode.results().len(), 0);
    }
}
