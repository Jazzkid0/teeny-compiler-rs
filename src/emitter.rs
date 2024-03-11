#![allow(dead_code, unused_imports)]

use crate::parser::{self, Statement};
use std::error::Error;

// Emit C code based on the AST we have generated.
//
// Things we need to keep track of:
// Variables
// Labels, and whether they have been GOTOed
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

fn headerline() -> String {
    "#include <stdio.h>\nint main(void){\n".to_string()
}

fn footerline() -> String {
    "return 0;\n}\n".to_string()
}

fn emit_program(statements: Vec<Statement>) -> Result<Vec<String>, Box<dyn Error>> {
    let mut codelines: Vec<String> = Vec::new();

    codelines.push(headerline());

    for statement in statements {
        match statement {
            _ => codelines.push("/* unimplemented statement */".to_string()),
        }
    }

    codelines.push(footerline());

    Ok(codelines)
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
                "#include <stdio.h>\nint main(void){\n".to_string(),
                "/* unimplemented statement */".to_string(),
                "return 0;\n}\n".to_string()
            ]
        );
    }
}
