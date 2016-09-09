extern crate fragment;
extern crate scribe;

use scribe::buffer::{Position, Scope, Token, TokenSet};
use helpers::SelectableSet;
use std::fmt;
use std::clone::Clone;

pub struct SymbolJumpMode {
    pub insert: bool,
    pub input: String,
    pub symbols: Vec<Symbol>,
    pub results: SelectableSet<Symbol>,
}

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
        let symbols = symbols(tokens);

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

fn symbols(tokens: TokenSet) -> Vec<Symbol> {
    tokens.iter()
          .filter_map(|token| {
              if let Token::Lexeme(lexeme) = token {
                  if let Some(scope) = lexeme.scope {
                      // Build a symbol, provided it's of the right type.
                      if Scope::new("meta.function").unwrap().is_prefix_of(scope) {
                          return Some(Symbol {
                              token: lexeme.value.to_string(),
                              position: lexeme.position.clone(),
                          })
                      }
                  }
              }

              None
          })
          .collect()
}

#[cfg(test)]
mod tests {
    extern crate scribe;

    use super::symbols;
    use scribe::buffer::{Category, Position, Token};

    #[test]
    fn symbols_are_limited_to_functions_and_methods() {
        let tokens = vec![
            Token{ lexeme: String::new(), category: Category::Keyword },
            Token{ lexeme: String::new(), category: Category::Method },
            Token{ lexeme: String::new(), category: Category::Function },
            Token{ lexeme: String::new(), category: Category::Identifier },
        ];

        assert_eq!(symbols(tokens.clone()).first().unwrap().token.category,
                   Category::Method);
        assert_eq!(symbols(tokens.clone()).last().unwrap().token.category,
                   Category::Function);
    }

    #[test]
    fn symbols_have_correct_positions() {
        let tokens = vec![
            Token{ lexeme: "class".to_string(), category: Category::Keyword },
            Token{ lexeme: " ".to_string(), category: Category::Whitespace },
            Token{ lexeme: "Ruby".to_string(), category: Category::Identifier },
            Token{ lexeme: "\n  ".to_string(), category: Category::Whitespace },
            Token{ lexeme: "def".to_string(), category: Category::Keyword },
            Token{ lexeme: " ".to_string(), category: Category::Whitespace },
            Token{ lexeme: "method".to_string(), category: Category::Method },
        ];

        assert_eq!(symbols(tokens.clone()).last().unwrap().position,
                   Position {
                       line: 1,
                       offset: 6,
                   });
    }
}
