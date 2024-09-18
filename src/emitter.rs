#![allow(dead_code, unused_imports)]

use crate::parser::*;
use std::error::Error;

// Emit C code based on the AST we have generated.
//
// Things we need to keep track of:
// Variables
// Labels, and whether they have been GOTOed
// Indentation? Formatting isn't necessary, but it would be nice
// Maybe the scope, might call all vars globally for now
//
// With these, we can move through the AST and generate C code.

#[derive(Debug)]
struct Symbol {
    name: String,
    value: i32,
}

#[derive(Debug)]
struct Label {
    name: String,
    used: bool,
}

fn print_unary(unary: Unary) -> String {
    match unary {
        Unary::Plus(primary) => match *primary {
            Primary::Ident(ident) => format!("{}", ident),
            Primary::Number(number) => format!("{}", number),
        },
        Unary::Minus(primary) => match *primary {
            Primary::Ident(ident) => format!("-{}", ident),
            Primary::Number(number) => format!("-{}", number),
        },
    }
}

pub fn emit_program(statements: Vec<Statement>) -> Result<Vec<String>, Box<dyn Error>> {
    let mut code_header: Vec<String> = Vec::new();
    let mut code_body: Vec<String> = Vec::new();

    code_header.push("#include <stdio.h>".to_string());
    code_header.push("int main(void){\n".to_string());

    for statement in statements {
        match statement {
            Statement::PrintString(string) => code_body.push(format!("printf(\"{}\\n\");", string)),
            Statement::PrintExpression(expression) => match *expression {
                Expression::SingleTerm(term) => match *term {
                    Term::SingleUnary(unary) => match *unary {
                        Unary::Plus(primary) => match *primary {
                            Primary::Ident(ident) => {
                                code_body.push(format!("printf(\"%d\\n\", {});", ident))
                            }
                            Primary::Number(number) => {
                                code_body.push(format!("printf(\"%d\\n\", {});", number))
                            }
                        },
                        Unary::Minus(primary) => match *primary {
                            Primary::Ident(ident) => {
                                code_body.push(format!("printf(\"%d\\n\", -{});", ident))
                            }
                            Primary::Number(number) => {
                                code_body.push(format!("printf(\"%d\\n\", -{});", number))
                            }
                        },
                    },
                    Term::WithTail(_unary, _tailunaries) => {}
                },
                Expression::WithTail(_term, _tailterms) => {

                    code_body.push("/* unimplemented expression with tail */".to_string())
                }
            },
            Statement::If { comparison: _, body: _ } => {
                code_body.push("/* unimplemented if statement */".to_string())
            }
            Statement::While { comparison: _, body: _ } => {
                code_body.push("/* unimplemented while statement */".to_string())
            }
            Statement::Label(_ident) => code_body.push("/* unimplemented label */".to_string()),
            Statement::Goto(_ident) => code_body.push("/* unimplemented goto */".to_string()),
            Statement::Let { ident: _, expression: _ } => {
                code_body.push("/* unimplemented let */".to_string())
            }
            Statement::Input(_ident) => code_body.push("/* unimplemented input */".to_string()),
        }
    }

    code_body.push("return 0;".to_string());
    code_body.push("}".to_string());

    let mut output: Vec<String> = Vec::new();

    output.append(&mut code_header);
    output.append(&mut code_body);

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_emit_program() {
        let ast = vec![Statement::PrintString("waddup".to_string())];
        let result = emit_program(ast).unwrap();
        assert_eq!(
            result,
            vec![
                "#include <stdio.h>".to_string(),
                "int main(void){\n".to_string(),
                "printf(\"waddup\\n\");".to_string(),
                "return 0;".to_string(),
                "}".to_string(),
            ]
        );
    }
}
