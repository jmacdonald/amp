extern crate fragment;
extern crate scribe;

use scribe::buffer::{Category, Position, Token};
use helpers::SelectableSet;
use std::fmt;
use std::clone::Clone;

const MAX_RESULTS: usize = 5;

pub struct SymbolJumpMode {
    pub input: String,
    pub symbols: Vec<Symbol>,
    pub results: SelectableSet<Symbol>,
}

pub struct Symbol {
    token: Token,
    position: Position,
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.token.lexeme)
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
    pub fn new(tokens: Vec<Token>) -> SymbolJumpMode {
        let symbols = symbols(tokens);

        SymbolJumpMode {
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
        let results = fragment::matching::find(&self.input, &self.symbols, MAX_RESULTS);

        // We don't care about the result objects; we just want
        // the underlying symbols. Map the collection to get these.
        self.results = SelectableSet::new(results.into_iter().map(|r| r.clone()).collect());
    }
}

fn symbols(tokens: Vec<Token>) -> Vec<Symbol> {
    let mut position = Position {
        line: 0,
        offset: 0,
    };

    tokens.into_iter()
          .filter_map(|token| {
              // Build a symbol, provided it's of the right type.
              let symbol = if token.category == Category::Method ||
                              token.category == Category::Function {
                  Some(Symbol {
                      token: token.clone(),
                      position: position.clone(),
                  })
              } else {
                  None
              };

              // Move the tracked position beyond this lexeme.
              let lines: Vec<&str> = token.lexeme.split('\n').collect();
              position.line += lines.len() - 1;
              position.offset += match lines.last() {
                  Some(line) => line.len(),
                  None => 0,
              };

              symbol
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
            Token{ lexeme: "amp\n ".to_string(), category: Category::Method },
            Token{ lexeme: "editor".to_string(), category: Category::Function },
        ];

        assert_eq!(symbols(tokens.clone()).first().unwrap().position,
                   Position {
                       line: 0,
                       offset: 0,
                   });
        assert_eq!(symbols(tokens.clone()).last().unwrap().position,
                   Position {
                       line: 1,
                       offset: 1,
                   });
    }
}
