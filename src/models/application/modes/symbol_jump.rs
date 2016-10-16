extern crate fragment;
extern crate scribe;

use scribe::buffer::{Position, ScopeStack, Token, TokenSet};
use helpers::SelectableSet;
use std::fmt;
use std::iter::Iterator;
use std::clone::Clone;
use std::str::FromStr;

pub struct SymbolJumpMode {
    pub insert: bool,
    pub input: String,
    pub symbols: Vec<Symbol>,
    pub results: SelectableSet<Symbol>,
}

#[derive(PartialEq, Debug)]
pub struct Symbol {
    token: String,
    position: Position,
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.token)
    }
}

impl Clone for Symbol {
    fn clone(&self) -> Symbol {
        Symbol{ token: self.token.clone(), position: self.position.clone() }
    }

    fn clone_from(&mut self, source: &Self) {
        self.token = source.token.clone();
        self.position = source.position.clone();
    }
}

impl SymbolJumpMode {
    pub const MAX_RESULTS: usize = 5;

    pub fn new(tokens: TokenSet) -> SymbolJumpMode {
        let symbols = symbols(tokens.iter());

        SymbolJumpMode {
            insert: true,
            input: String::new(),
            symbols: symbols,
            results: SelectableSet::new(Vec::new()),
        }
    }

    pub fn selected_symbol_position(&self) -> Option<Position> {
        self.results.selection().map(|symbol| symbol.position.clone())
    }

    pub fn search(&mut self) {
        // Find the symbols we're looking for using the query.
        let results = fragment::matching::find(&self.input, &self.symbols, SymbolJumpMode::MAX_RESULTS);

        // We don't care about the result objects; we just want
        // the underlying symbols. Map the collection to get these.
        self.results = SelectableSet::new(results.into_iter().map(|r| r.clone()).collect());
    }
}

fn symbols<'a, T>(tokens: T) -> Vec<Symbol> where T: Iterator<Item=Token<'a>> {
    tokens.filter_map(|token| {
          if let Token::Lexeme(lexeme) = token {
              // Build a symbol, provided it's of the right type.
              if ScopeStack::from_str("entity.name.function").unwrap().does_match(lexeme.scope.as_slice()).is_some() {
                  return Some(Symbol {
                      token: lexeme.value.to_string(),
                      position: lexeme.position.clone(),
                  })
              }
          }

          None
    }).collect()
}

#[cfg(test)]
mod tests {
    extern crate scribe;

    use super::{Symbol, symbols};
    use scribe::buffer::{Lexeme, Position, Scope, Token};

    #[test]
    fn symbols_are_limited_to_functions() {
        let tokens = vec![
            Token::Lexeme(
                Lexeme{
                    value: "text",
                    position: Position{
                        line: 0,
                        offset: 0
                    },
                    scope: Scope::new("meta.block.rust").ok()
                }
            ),
            Token::Lexeme(
                Lexeme{
                    value: "function",
                    position: Position{
                        line: 1,
                        offset: 0
                    },
                    scope: Scope::new("entity.name.function").ok()
                }
            ),
            Token::Lexeme(
                Lexeme{
                    value: "non-function",
                    position: Position{
                        line: 2,
                        offset: 0
                    },
                    scope: Scope::new("meta.entity.name.function").ok()
                }
            )
        ];

        let results = symbols(tokens.into_iter());
        assert_eq!(results.len(), 1);
        assert_eq!(results.first().unwrap(), &Symbol{ token: "function".to_string(), position: Position{ line: 1, offset: 0 }});
    }
}
