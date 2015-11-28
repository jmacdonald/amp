extern crate luthor;

use self::luthor::{Tokenizer, StateFunction};
use self::luthor::token::{Token, Category};

fn initial_state(lexer: &mut Tokenizer) -> Option<StateFunction> {
    if lexer.has_prefix("::") {
      lexer.tokenize(Category::Text);
      lexer.tokenize_next(2, Category::Text);
    }

    match lexer.current_char() {
        Some(c) => {
            match c {
                ' ' | '\n' => {
                    lexer.tokenize(Category::Text);
                    lexer.advance();
                    return Some(StateFunction(whitespace));
                },
                '_' | '-' | '.' | '(' | ')' | '{' | '}' | ';' | '|' |
                ',' | ':' | '<' | '>' | '\'' | '"' | '?' | '/' | '\\' |
                '[' | ']' => {
                    lexer.tokenize(Category::Text);
                    lexer.tokenize_next(1, Category::Text);
                    return Some(StateFunction(whitespace));
                },
                _ => {
                    if c.is_uppercase() {
                        lexer.tokenize(Category::Text);
                        lexer.advance();
                        return Some(StateFunction(uppercase))
                    }

                    lexer.advance()
                }
            }

            Some(StateFunction(initial_state))
        }

        None => {
            lexer.tokenize(Category::Text);
            None
        }
    }
}

fn whitespace(lexer: &mut Tokenizer) -> Option<StateFunction> {
    match lexer.current_char() {
        Some(c) => {
            match c {
                ' ' | '\n' => {
                    lexer.advance();
                    Some(StateFunction(whitespace))
                },
                _ => {
                    lexer.tokenize(Category::Whitespace);
                    Some(StateFunction(initial_state))
                }
            }
        }

        None => {
            lexer.tokenize(Category::Whitespace);
            None
        }
    }
}

fn uppercase(lexer: &mut Tokenizer) -> Option<StateFunction> {
    match lexer.current_char() {
        Some(c) => {
            if c.is_alphabetic() {
                lexer.advance();
                Some(StateFunction(uppercase))
            } else {
                lexer.tokenize(Category::Text);
                Some(StateFunction(initial_state))
            }
        },
        None => {
            lexer.tokenize(Category::Text);
            None
        }
    }
}

pub fn lex(data: &str) -> Vec<Token> {
    let mut lexer = Tokenizer::new(data);
    let mut state_function = StateFunction(initial_state);
    loop {
        let StateFunction(actual_function) = state_function;
        match actual_function(&mut lexer) {
            Some(f) => state_function = f,
            None => return lexer.tokens(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::luthor::token::{Token, Category};

    #[test]
    fn it_works() {
        let data = "local_variable = camelCase.method(param)\n  something-else CONSTANT val";
        let tokens = lex(data);
        let expected_tokens = vec![
            Token{ lexeme: "local".to_string(), category: Category::Text },
            Token{ lexeme: "_".to_string(), category: Category::Text },
            Token{ lexeme: "variable".to_string(), category: Category::Text },
            Token{ lexeme: " ".to_string(), category: Category::Whitespace },
            Token{ lexeme: "=".to_string(), category: Category::Text },
            Token{ lexeme: " ".to_string(), category: Category::Whitespace },
            Token{ lexeme: "camel".to_string(), category: Category::Text },
            Token{ lexeme: "Case".to_string(), category: Category::Text },
            Token{ lexeme: ".".to_string(), category: Category::Text },
            Token{ lexeme: "method".to_string(), category: Category::Text },
            Token{ lexeme: "(".to_string(), category: Category::Text },
            Token{ lexeme: "param".to_string(), category: Category::Text },
            Token{ lexeme: ")".to_string(), category: Category::Text },
            Token{ lexeme: "\n  ".to_string(), category: Category::Whitespace },
            Token{ lexeme: "something".to_string(), category: Category::Text },
            Token{ lexeme: "-".to_string(), category: Category::Text },
            Token{ lexeme: "else".to_string(), category: Category::Text },
            Token{ lexeme: " ".to_string(), category: Category::Whitespace },
            Token{ lexeme: "CONSTANT".to_string(), category: Category::Text },
            Token{ lexeme: " ".to_string(), category: Category::Whitespace },
            Token{ lexeme: "val".to_string(), category: Category::Text },
        ];

        for (index, token) in tokens.iter().enumerate() {
            assert_eq!(*token, expected_tokens[index]);
        }
    }
}
