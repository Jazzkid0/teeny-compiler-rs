use std::error::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    EOF,
    Newline,
    Number { value: i32 },
    Identifier { name: String },
    String { value: String },
    // Keywords
    Label { name: String },
    Goto,
    Print,
    Input,
    Let,
    If,
    Then,
    Endif,
    While,
    Repeat,
    Endwhile,
    // Operators
    Equal,
    Plus,
    Minus,
    Asterisk,
    Slash,
    EqualEqual,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanEqual,
    GreaterThanEqual,
}

pub fn lex(input: &str) -> Result<Vec<Token>, Box<dyn Error>> {
    let mut tokens = vec![];
    let mut lines = input.lines().peekable();

    while let Some(line) = lines.next() {
        let mut chars = line.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '\0' => tokens.push(Token::EOF),
                ' ' => continue,
                '\t' => continue,
                '\r' => continue,
                '\n' => tokens.push(Token::Newline),
                '0'..='9' => {
                    let mut value = c.to_string();

                    while let Some('0'..='9') = chars.peek() {
                        value.push(chars.next().unwrap());
                    }

                    tokens.push(Token::Number {
                        value: value.parse().unwrap(),
                    });
                }
                '"' => {
                    let mut value = String::new();

                    while let Some(c) = chars.next() {
                        if c == '"' {
                            break;
                        }
                        value.push(c);
                    }

                    tokens.push(Token::String { value });
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let mut name = c.to_string();

                    while let Some('a'..='z') | Some('A'..='Z') | Some('0'..='9') | Some('_') = chars.peek() {
                        name.push(chars.next().unwrap());
                    }

                    match name.as_str() {
                        "label" => tokens.push(Token::Label { name }),
                        "goto" => tokens.push(Token::Goto),
                        "print" => tokens.push(Token::Print),
                        "input" => tokens.push(Token::Input),
                        "let" => tokens.push(Token::Let),
                        "if" => tokens.push(Token::If),
                        "then" => tokens.push(Token::Then),
                        "endif" => tokens.push(Token::Endif),
                        "while" => tokens.push(Token::While),
                        "repeat" => tokens.push(Token::Repeat),
                        "endwhile" => tokens.push(Token::Endwhile),
                        _ => tokens.push(Token::Identifier { name }),
                    }
                }
                '=' => {
                    if let Some('=') = chars.peek() {
                        chars.next();
                        tokens.push(Token::EqualEqual);
                    } else {
                        tokens.push(Token::Equal);
                    }
                }
                '+' => tokens.push(Token::Plus),
                '-' => tokens.push(Token::Minus),
                '*' => tokens.push(Token::Asterisk),
                '/' => tokens.push(Token::Slash),
                '!' => {
                    if let Some('=') = chars.peek() {
                        chars.next();
                        tokens.push(Token::NotEqual);
                    } else {
                        return Err("Unexpected character '!'".into());
                    }
                }
                '<' => {
                    if let Some('=') = chars.peek() {
                        chars.next();
                        tokens.push(Token::LessThanEqual);
                    } else {
                        tokens.push(Token::LessThan);
                    }
                }
                '>' => {
                    if let Some('=') = chars.peek() {
                        chars.next();
                        tokens.push(Token::GreaterThanEqual);
                    } else {
                        tokens.push(Token::GreaterThan);
                    }
                }
                _ => return Err(format!("Unexpected character '{}'", c).into()),
            }
        }
    }
    Ok(tokens)
}

#[derive(Debug, Clone)]
pub struct TokenIterator<'a> {
    tokens: &'a [Token],
    index: usize,
}

impl<'a> TokenIterator<'a> {
    pub fn new(tokens: &'a Vec<Token>) -> Self {
        TokenIterator { tokens, index: 0 }
    }
}

impl Iterator for TokenIterator<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.tokens.len() {
            let token = self.tokens[self.index].clone();
            self.index += 1;
            Some(token)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex() {
        let input = r#"
            label main
            let x = 10
            let y = 20
            if x < y
            then
                print "x is less than y"
            endif
            while x < y
            repeat
                input x
            endwhile
            goto main
        "#;

        let tokens = lex(input).unwrap();
        for token in &tokens {
            println!("{:?}", token);
        }
        assert_eq!(tokens.len(), 28);
    }
}
