#![allow(dead_code)]

// Parser for the following grammar:
//
// program ::= {statement}
// statement ::= "PRINT" (expression | string) nl
//     | "IF" comparison "THEN" nl {statement} "ENDIF" nl
//     | "WHILE" comparison "REPEAT" nl {statement} "ENDWHILE" nl
//     | "LABEL" ident nl
//     | "GOTO" ident nl
//     | "LET" ident "=" expression nl
//     | "INPUT" ident nl
// comparison ::= expression (("==" | "!=" | ">" | ">=" | "<" | "<=") expression)+
// expression ::= term {( "-" | "+" ) term}
// term ::= unary {( "/" | "*" ) unary}
// unary ::= ["+" | "-"] primary
// primary ::= number | ident
// nl ::= '\n'+

use crate::lexer::{TokenIterator, Token};
use std::error::Error;
use std::iter::Peekable;

#[derive(Debug, PartialEq)]
pub enum AST {
    Program(Vec<Statement>),
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    // Make print allow for both strings and expressions
    PrintString(String),
    PrintExpression(Box<Expression>),
    If {
        comparison: Comparison,
        body: Vec<Statement>,
    },
    While {
        comparison: Comparison,
        body: Vec<Statement>,
    },
    Label(String),
    Goto(String),
    Let {
        ident: String,
        expression: Expression,
    },
    Input(String),
}

#[derive(Debug, PartialEq)]
pub enum Comparison {
    Equal(Box<Expression>, Box<Expression>),
    NotEqual(Box<Expression>, Box<Expression>),
    GreaterThan(Box<Expression>, Box<Expression>),
    GreaterThanEqual(Box<Expression>, Box<Expression>),
    LessThan(Box<Expression>, Box<Expression>),
    LessThanEqual(Box<Expression>, Box<Expression>),
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Add(Box<Expression>, Box<Expression>),
    Subtract(Box<Expression>, Box<Expression>),
    Multiply(Box<Expression>, Box<Expression>),
    Divide(Box<Expression>, Box<Expression>),
    UnaryMinus(Box<Expression>),
    UnaryPlus(Box<Expression>),
    Number(i32),
    Ident(String),
}

pub fn parse(tokens: &mut Peekable<TokenIterator>) -> Result<AST, Box<dyn Error>> {
    let mut statements = vec![];

    parse_ast(tokens.next().unwrap(), tokens, &mut statements)?;

    while let Some(token) = tokens.peek().cloned() {
        parse_ast(token, tokens, &mut statements)?;
    }
    Ok(AST::Program(statements))
}

fn parse_ast(token: Token, tokens: &mut Peekable<TokenIterator>, statements: &mut Vec<Statement>) -> Result<(), Box<dyn Error>> {
    match token {
        Token::Print => {
            match tokens.peek() {
                Some(Token::String { value }) => {
                    let contents = value.clone();
                    tokens.next();
                    statements.push(Statement::PrintString(contents));
                }
                _ => {
                    let expression = parse_expression(tokens)?;
                    statements.push(Statement::PrintExpression(Box::new(expression)));
                }
            }
        }
        Token::If => {
            let comparison = parse_comparison(tokens)?;
            let mut body = vec![];
            while let Some(token) = tokens.peek() {
                match token {
                    Token::Endif => {
                        tokens.next();
                        break;
                    }
                    _ => {
                        body.push(parse_statement(tokens)?);
                    }
                }
            }
            statements.push(Statement::If {
                comparison,
                body,
            });
        }
        Token::While => {
            tokens.next();
            let comparison = parse_comparison(tokens)?;
            let mut body = vec![];
            while let Some(token) = tokens.peek() {
                match token {
                    Token::Endwhile => {
                        tokens.next();
                        break;
                    }
                    _ => {
                        body.push(parse_statement(tokens)?);
                    }
                }
            }
            statements.push(Statement::While {
                comparison,
                body,
            });
        }
        Token::Label { name } => {
            statements.push(Statement::Label(name.clone()));
        }
        Token::Goto => {
            let name = match tokens.next() {
                Some(Token::Identifier { name }) => name,
                _ => return Err("Expected identifier after GOTO".into()),
            };
            statements.push(Statement::Goto(name));
        }
        Token::Let => {
            let ident = match tokens.next() {
                Some(Token::Identifier { name }) => name,
                _ => return Err("Expected identifier after LET".into()),
            };
            match tokens.next() {
                Some(Token::Equal) => {}
                _ => return Err("Expected = after identifier in LET".into()),
            }
            let expression = parse_expression(tokens)?;
            statements.push(Statement::Let {
                ident,
                expression,
            });
        }
        Token::Input => {
            tokens.next();
            let ident = match tokens.next() {
                Some(Token::Identifier { name }) => name,
                _ => return Err("Expected identifier after INPUT".into()),
            };
            statements.push(Statement::Input(ident));
        }
        _ => return Err(format!("Unexpected token: {:?}", token).into()),
    }
    Ok(())
}

fn parse_statement(tokens: &mut Peekable<TokenIterator>) -> Result<Statement, Box<dyn Error>> {
    match tokens.next() {
        Some(Token::Print) => {
            let expression = parse_expression(tokens)?;
            Ok(Statement::PrintExpression(Box::new(expression)))
        }
        Some(Token::If) => {
            let comparison = parse_comparison(tokens)?;
            let mut body = vec![];
            while let Some(token) = tokens.peek() {
                match token {
                    Token::Endif => {
                        tokens.next();
                        break;
                    }
                    _ => {
                        body.push(parse_statement(tokens)?);
                    }
                }
            }
            Ok(Statement::If {
                comparison,
                body,
            })
        }
        Some(Token::While) => {
            let comparison = parse_comparison(tokens)?;
            let mut body = vec![];
            while let Some(token) = tokens.peek() {
                match token {
                    Token::Endwhile => {
                        tokens.next();
                        break;
                    }
                    _ => {
                        body.push(parse_statement(tokens)?);
                    }
                }
            }
            Ok(Statement::While {
                comparison,
                body,
            })
        }
        Some(Token::Label { name }) => Ok(Statement::Label(name.clone())),
        Some(Token::Goto) => {
            let name = match tokens.next() {
                Some(Token::Identifier { name }) => name,
                _ => return Err("Expected identifier after GOTO".into()),
            };
            Ok(Statement::Goto(name))
        }
        Some(Token::Let) => {
            let ident = match tokens.next() {
                Some(Token::Identifier { name }) => name,
                _ => return Err("Expected identifier after LET".into()),
            };
            match tokens.next() {
                Some(Token::Equal) => {}
                _ => return Err("Expected = after identifier in LET".into()),
            }
            let expression = parse_expression(tokens)?;
            Ok(Statement::Let {
                ident,
                expression,
            })
        }
        Some(Token::Input) => {
            let ident = match tokens.next() {
                Some(Token::Identifier { name }) => name,
                _ => return Err("Expected identifier after INPUT".into()),
            };
            Ok(Statement::Input(ident))
        }
        _ => Err("Unexpected token".into()),
    }
}

fn parse_comparison(tokens: &mut Peekable<TokenIterator>) -> Result<Comparison, Box<dyn Error>> {
    let expression = parse_expression(tokens)?;
    let token = match tokens.next() {
        Some(token) => token,
        None => return Err("Unexpected end of input".into()),
    };
    let expression2 = parse_expression(tokens)?;
    match token {
        Token::EqualEqual => Ok(Comparison::Equal(Box::new(expression), Box::new(expression2))),
        Token::NotEqual => Ok(Comparison::NotEqual(Box::new(expression), Box::new(expression2))),
        Token::GreaterThan => Ok(Comparison::GreaterThan(Box::new(expression), Box::new(expression2))),
        Token::GreaterThanEqual => Ok(Comparison::GreaterThanEqual(Box::new(expression), Box::new(expression2))),
        Token::LessThan => Ok(Comparison::LessThan(Box::new(expression), Box::new(expression2))),
        Token::LessThanEqual => Ok(Comparison::LessThanEqual(Box::new(expression), Box::new(expression2))),
        _ => Err("Expected comparison operator".into()),
    }
}

fn parse_expression(tokens: &mut Peekable<TokenIterator>) -> Result<Expression, Box<dyn Error>> {
    let mut expression = parse_term(tokens)?;
    while let Some(token) = tokens.peek() {
        match token {
            Token::Plus => {
                tokens.next();
                let term = parse_term(tokens)?;
                expression = Expression::Add(Box::new(expression), Box::new(term));
            }
            Token::Minus => {
                tokens.next();
                let term = parse_term(tokens)?;
                expression = Expression::Subtract(Box::new(expression), Box::new(term));
            }
            _ => break,
        }
    }
    Ok(expression)
}

fn parse_term(tokens: &mut Peekable<TokenIterator>) -> Result<Expression, Box<dyn Error>> {
    let mut expression = parse_unary(tokens)?;
    while let Some(token) = tokens.peek() {
        match token {
            Token::Asterisk => {
                tokens.next();
                let unary = parse_unary(tokens)?;
                expression = Expression::Multiply(Box::new(expression), Box::new(unary));
            }
            Token::Slash => {
                tokens.next();
                let unary = parse_unary(tokens)?;
                expression = Expression::Divide(Box::new(expression), Box::new(unary));
            }
            _ => break,
        }
    }
    Ok(expression)
}

fn parse_unary(tokens: &mut Peekable<TokenIterator>) -> Result<Expression, Box<dyn Error>> {
    match tokens.next() {
        Some(Token::Plus) => {
            let primary = parse_primary(tokens)?;
            Ok(Expression::UnaryPlus(Box::new(primary)))
        }
        Some(Token::Minus) => {
            let primary = parse_primary(tokens)?;
            Ok(Expression::UnaryMinus(Box::new(primary)))
        }
        _ => parse_primary(tokens),
    }
}

fn parse_primary(tokens: &mut Peekable<TokenIterator>) -> Result<Expression, Box<dyn Error>> {
    match tokens.next() {
        Some(Token::Number { value }) => Ok(Expression::Number(value)),
        Some(Token::Identifier { name }) => Ok(Expression::Ident(name)),
        _ => Err("Expected number or identifier".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;

    #[allow(dead_code)]
    #[test]
    fn test_parse() {
        let input = r#"
print "waddup"
if 1 == 1 then
print 2
endif
while 1 == 1 repeat
print 3
endwhile
label foo
goto foo
let x = 1
input x
"#;

        for line in input.lines() {
            println!("{}", line);
        }

        let tokens = lex(input).unwrap();

        for token in tokens.clone() {
            println!("Token: {:?}", token);
        }

        let mut tokens = TokenIterator::new(&tokens).peekable();
        let ast = parse(&mut tokens).unwrap();

        assert_eq!(
            ast,
            AST::Program(vec![
                Statement::PrintString("waddup".to_string()),
                Statement::If {
                    comparison: Comparison::Equal(Box::new(Expression::Number(1)), Box::new(Expression::Number(1))),
                    body: vec![Statement::PrintExpression(Box::new(Expression::Number(2)))],
                },
                Statement::While {
                    comparison: Comparison::Equal(Box::new(Expression::Number(1)), Box::new(Expression::Number(1))),
                    body: vec![Statement::PrintExpression(Box::new(Expression::Number(3)))],
                },
                Statement::Label("foo".to_string()),
                Statement::Goto("foo".to_string()),
                Statement::Let {
                    ident: "x".to_string(),
                    expression: Expression::Number(1),
                },
                Statement::Input("x".to_string()),
            ])
        );
    }
}
