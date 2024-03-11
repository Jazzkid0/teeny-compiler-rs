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

use crate::lexer::{Token, TokenIterator};
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
    SingleTerm(Box<Term>),
    WithTail(Box<Term>, Box<ExpressionTail>),
}

#[derive(Debug, PartialEq)]
pub enum ExpressionTail {
    Tail(Vec<TailTerm>),
}

#[derive(Debug, PartialEq)]
pub enum TailTerm {
    Add(Box<Term>),
    Subtract(Box<Term>),
}

#[derive(Debug, PartialEq)]
pub enum Term {
    SingleUnary(Box<Unary>),
    WithTail(Box<Unary>, Box<TermTail>),
}

#[derive(Debug, PartialEq)]
pub enum TermTail {
    Tail(Vec<TailUnary>),
}

#[derive(Debug, PartialEq)]
pub enum TailUnary {
    Multiply(Box<Unary>),
    Divide(Box<Unary>),
}

#[derive(Debug, PartialEq)]
pub enum Unary {
    Plus(Box<Primary>),
    Minus(Box<Primary>),
}

#[derive(Debug, PartialEq)]
pub enum Primary {
    Number(i32),
    Ident(String),
}

pub fn parse(tokens: &mut Peekable<TokenIterator>) -> Result<AST, Box<dyn Error>> {
    let mut statements = vec![];
    while let Some(token) = tokens.next() {
        println!("AST--- Parsing token: {:?}", token);
        match token {
            Token::Print => {
                let next = tokens.next();
                println!("AST--- Parsing print: {:?}", next);
                match next {
                    Some(Token::String { value }) => {
                        let contents = value.clone();
                        statements.push(Statement::PrintString(contents));
                    }
                    _ => {
                        println!("AST--- Parsing print expression");
                        let expression = parse_expression(tokens)?;
                        statements.push(Statement::PrintExpression(Box::new(expression)));
                    }
                }
            }
            Token::If => {
                println!("AST--- Parsing if");
                let comparison = parse_comparison(tokens)?;
                let mut body = vec![];
                while let Some(token) = tokens.peek() {
                    println!("AST--- Parsing if body: {:?}", token);
                    match token {
                        Token::Endif => {
                            tokens.next();
                            break;
                        }
                        Token::Then => {
                            tokens.next();
                        }
                        _ => {
                            body.push(parse_statement(tokens)?);
                        }
                    }
                }
                statements.push(Statement::If { comparison, body });
            }
            Token::While => {
                println!("AST--- Parsing while");
                let comparison = parse_comparison(tokens)?;
                let mut body = vec![];
                while let Some(token) = tokens.peek() {
                    println!("AST--- Parsing while body: {:?}", token);
                    match token {
                        Token::Endwhile => {
                            tokens.next();
                            break;
                        }
                        Token::Repeat => {
                            tokens.next();
                        }
                        _ => {
                            body.push(parse_statement(tokens)?);
                        }
                    }
                }
                statements.push(Statement::While { comparison, body });
            }
            Token::Label => {
                println!("AST--- Parsing label");
                let name = match tokens.next() {
                    Some(Token::Identifier { name }) => name,
                    _ => return Err("Expected identifier after LABEL".into()),
                };
                statements.push(Statement::Label(name.clone()));
            }
            Token::Goto => {
                println!("AST--- Parsing goto");
                let name = match tokens.next() {
                    Some(Token::Identifier { name }) => name,
                    _ => return Err("Expected identifier after GOTO".into()),
                };
                statements.push(Statement::Goto(name));
            }
            Token::Let => {
                println!("AST--- Parsing let");
                let ident = match tokens.next() {
                    Some(Token::Identifier { name }) => name,
                    _ => return Err("Expected identifier after LET".into()),
                };
                match tokens.next() {
                    Some(Token::Equal) => {}
                    _ => return Err("Expected = after identifier in LET".into()),
                }
                tokens.next();
                let expression = parse_expression(tokens)?;
                statements.push(Statement::Let { ident, expression });
            }
            Token::Input => {
                println!("AST--- Parsing input");
                let ident = match tokens.next() {
                    Some(Token::Identifier { name }) => name,
                    _ => return Err("Expected identifier after INPUT".into()),
                };
                statements.push(Statement::Input(ident));
            }
            _ => {
                return Err(format!(
                    "Unexpected token at AST: {:?}\nAST State: {:?}",
                    token,
                    AST::Program(statements)
                )
                .into())
            }
        }
    }
    Ok(AST::Program(statements))
}

fn parse_statement(tokens: &mut Peekable<TokenIterator>) -> Result<Statement, Box<dyn Error>> {
    let token = tokens.next();
    println!("STATEMENT--- Parsing token: {:?}", token);
    match token {
        Some(Token::Print) => match tokens.peek() {
            Some(Token::String { value }) => {
                let contents = value.clone();
                Ok(Statement::PrintString(contents))
            }
            _ => {
                let expression = parse_expression(tokens)?;
                Ok(Statement::PrintExpression(Box::new(expression)))
            }
        },
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
            Ok(Statement::If { comparison, body })
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
            Ok(Statement::While { comparison, body })
        }
        Some(Token::Label { name }) => Ok(Statement::Label(name.clone())),
        Some(Token::Goto) => {
            let name = match tokens.next() {
                Some(Token::Identifier { name }) => name,
                _ => {
                    println!("Unexpected token in STATEMENT: {:?}", tokens.peek());
                    return Err("Expected identifier after GOTO".into());
                }
            };
            Ok(Statement::Goto(name))
        }
        Some(Token::Let) => {
            let ident = match tokens.next() {
                Some(Token::Identifier { name }) => name,
                _ => {
                    println!("Unexpected token in STATEMENT: {:?}", tokens.peek());
                    return Err("Expected identifier after LET".into());
                }
            };
            match tokens.next() {
                Some(Token::Equal) => {}
                _ => {
                    println!("Unexpected token in STATEMENT: {:?}", tokens.peek());
                    return Err("Expected = after identifier in LET".into());
                }
            }
            let expression = parse_expression(tokens)?;
            Ok(Statement::Let { ident, expression })
        }
        Some(Token::Input) => {
            let ident = match tokens.next() {
                Some(Token::Identifier { name }) => name,
                _ => {
                    println!("Unexpected token in STATEMENT: {:?}", tokens.peek());
                    return Err("Expected identifier after INPUT".into());
                }
            };
            Ok(Statement::Input(ident))
        }
        _ => {
            println!("Unexpected token in STATEMENT: {:?}", tokens.peek());
            Err("Unexpected token at root".into())
        }
    }
}

fn parse_comparison(tokens: &mut Peekable<TokenIterator>) -> Result<Comparison, Box<dyn Error>> {
    println!("COMPARISON--- Parsing token: {:?}", tokens.peek());
    let expression = parse_expression(tokens)?;
    println!("COMPARISON--- Got Comparator: {:?}", tokens.peek());
    let comparator = tokens.next();
    println!("COMPARISON--- Parsing token: {:?}", tokens.peek());
    let expression2 = parse_expression(tokens)?;
    println!(
        "COMPARISON: {:?} {:?} {:?}",
        expression, comparator, expression2
    );
    match comparator {
        Some(Token::EqualEqual) => Ok(Comparison::Equal(
            Box::new(expression),
            Box::new(expression2),
        )),
        Some(Token::NotEqual) => Ok(Comparison::NotEqual(
            Box::new(expression),
            Box::new(expression2),
        )),
        Some(Token::GreaterThan) => Ok(Comparison::GreaterThan(
            Box::new(expression),
            Box::new(expression2),
        )),
        Some(Token::GreaterThanEqual) => Ok(Comparison::GreaterThanEqual(
            Box::new(expression),
            Box::new(expression2),
        )),
        Some(Token::LessThan) => Ok(Comparison::LessThan(
            Box::new(expression),
            Box::new(expression2),
        )),
        Some(Token::LessThanEqual) => Ok(Comparison::LessThanEqual(
            Box::new(expression),
            Box::new(expression2),
        )),
        _ => Err("Expected comparison operator".into()),
    }
}

fn parse_expression(tokens: &mut Peekable<TokenIterator>) -> Result<Expression, Box<dyn Error>> {
    println!("EXPRESSION--- Parsing token: {:?}", tokens.peek());
    let initialterm = parse_term(tokens)?;
    if tokens.peek() == Some(&Token::Plus) || tokens.peek() == Some(&Token::Minus) {
        let mut tailterms = Vec::new();
        while let Some(token) = tokens.peek() {
            println!("EXPRESSION--- Parsing tail token: {:?}", token);
            match token {
                Token::Plus => {
                    tokens.next();
                    let tailterm = parse_term(tokens)?;
                    tailterms.push(TailTerm::Add(Box::new(tailterm)));
                }
                Token::Minus => {
                    tokens.next();
                    let term = parse_term(tokens)?;
                    tailterms.push(TailTerm::Subtract(Box::new(term)));
                }
                _ => break,
            }
        }
        Ok(Expression::WithTail(
            Box::new(initialterm),
            Box::new(ExpressionTail::Tail(tailterms)),
        ))
    } else {
        Ok(Expression::SingleTerm(Box::new(initialterm)))
    }
}

fn parse_term(tokens: &mut Peekable<TokenIterator>) -> Result<Term, Box<dyn Error>> {
    println!("TERM--- Parsing token: {:?}", tokens.peek());
    let initialunary = parse_unary(tokens)?;
    if tokens.peek() == Some(&Token::Asterisk) || tokens.peek() == Some(&Token::Slash) {
        let mut tailunaries = Vec::new();
        while let Some(token) = tokens.peek() {
            println!("TERM--- Parsing tail token: {:?}", token);
            match token {
                Token::Asterisk => {
                    tokens.next();
                    let tailunary = parse_unary(tokens)?;
                    tailunaries.push(TailUnary::Multiply(Box::new(tailunary)));
                }
                Token::Slash => {
                    tokens.next();
                    let tailunary = parse_unary(tokens)?;
                    tailunaries.push(TailUnary::Divide(Box::new(tailunary)));
                }
                _ => break,
            }
        }
        Ok(Term::WithTail(
            Box::new(initialunary),
            Box::new(TermTail::Tail(tailunaries)),
        ))
    } else {
        Ok(Term::SingleUnary(Box::new(initialunary)))
    }
}

fn parse_unary(tokens: &mut Peekable<TokenIterator>) -> Result<Unary, Box<dyn Error>> {
    println!("UNARY--- Parsing token: {:?}", tokens.peek());
    let unary = tokens.peek();
    match unary {
        Some(Token::Plus) => {
            tokens.next();
            let primary = parse_primary(tokens)?;
            Ok(Unary::Plus(Box::new(primary)))
        }
        Some(Token::Minus) => {
            tokens.next();
            let primary = parse_primary(tokens)?;
            Ok(Unary::Minus(Box::new(primary)))
        }
        _ => {
            let primary = parse_primary(tokens)?;
            Ok(Unary::Plus(Box::new(primary)))
        }
    }
}

fn parse_primary(tokens: &mut Peekable<TokenIterator>) -> Result<Primary, Box<dyn Error>> {
    println!("PRIMARY--- Parsing token: {:?}", tokens.peek());
    let primary = tokens.next();
    match primary {
        Some(Token::Number { value }) => Ok(Primary::Number(value)),
        Some(Token::Identifier { name }) => Ok(Primary::Ident(name)),
        _ => {
            println!("Unexpected token at PRIMARY {:?}", tokens.peek());
            Err("Expected number or identifier".into())
        }
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
                    comparison: Comparison::Equal(
                        Box::new(Expression::SingleTerm(Box::new(Term::SingleUnary(
                            Box::new(Unary::Plus(Box::new(Primary::Number(1))))
                        )))),
                        Box::new(Expression::SingleTerm(Box::new(Term::SingleUnary(
                            Box::new(Unary::Plus(Box::new(Primary::Number(1))))
                        ))))
                    ),
                    body: vec![Statement::PrintExpression(Box::new(
                        Expression::SingleTerm(Box::new(Term::SingleUnary(Box::new(Unary::Plus(
                            Box::new(Primary::Number(2))
                        )))))
                    ))]
                },
                Statement::While {
                    comparison: Comparison::Equal(
                        Box::new(Expression::SingleTerm(Box::new(Term::SingleUnary(
                            Box::new(Unary::Plus(Box::new(Primary::Number(1))))
                        )))),
                        Box::new(Expression::SingleTerm(Box::new(Term::SingleUnary(
                            Box::new(Unary::Plus(Box::new(Primary::Number(1))))
                        ))))
                    ),
                    body: vec![Statement::PrintExpression(Box::new(
                        Expression::SingleTerm(Box::new(Term::SingleUnary(Box::new(Unary::Plus(
                            Box::new(Primary::Number(3))
                        )))))
                    ))]
                },
                Statement::Label("foo".to_string()),
                Statement::Goto("foo".to_string()),
                Statement::Let {
                    ident: "x".to_string(),
                    expression: Expression::SingleTerm(Box::new(Term::SingleUnary(Box::new(
                        Unary::Plus(Box::new(Primary::Number(1)))
                    ))))
                },
                Statement::Input("x".to_string())
            ])
        );
    }
}
