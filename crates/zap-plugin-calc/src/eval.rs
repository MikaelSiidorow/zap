use std::f64::consts::{E, PI};

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Op(char),
    LParen,
    RParen,
    Ident(String),
    Comma,
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' => {
                chars.next();
            }
            '0'..='9' | '.' => {
                let mut num_str = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() || c == '.' {
                        num_str.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let n: f64 = num_str
                    .parse()
                    .map_err(|_| format!("invalid number: {num_str}"))?;
                tokens.push(Token::Number(n));
            }
            '+' | '-' | '/' | '%' | '^' => {
                tokens.push(Token::Op(ch));
                chars.next();
            }
            '*' => {
                chars.next();
                if chars.peek() == Some(&'*') {
                    chars.next();
                    tokens.push(Token::Op('^')); // ** → ^
                } else {
                    tokens.push(Token::Op('*'));
                }
            }
            '(' => {
                tokens.push(Token::LParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::RParen);
                chars.next();
            }
            ',' => {
                tokens.push(Token::Comma);
                chars.next();
            }
            _ if ch.is_alphabetic() || ch == '_' => {
                let mut ident = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        ident.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Ident(ident));
            }
            _ => return Err(format!("unexpected character: {ch}")),
        }
    }

    Ok(tokens)
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Token> {
        let tok = self.tokens.get(self.pos).cloned();
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    fn expect(&mut self, expected: &Token) -> Result<(), String> {
        match self.next() {
            Some(ref tok) if tok == expected => Ok(()),
            Some(tok) => Err(format!("expected {expected:?}, got {tok:?}")),
            None => Err(format!("expected {expected:?}, got end of input")),
        }
    }

    fn parse(&mut self) -> Result<f64, String> {
        let result = self.expr()?;
        if self.pos < self.tokens.len() {
            return Err(format!("unexpected token: {:?}", self.tokens[self.pos]));
        }
        Ok(result)
    }

    // expr → term (('+' | '-') term)*
    fn expr(&mut self) -> Result<f64, String> {
        let mut left = self.term()?;
        while let Some(Token::Op(op @ ('+' | '-'))) = self.peek().cloned() {
            self.next();
            let right = self.term()?;
            left = match op {
                '+' => left + right,
                '-' => left - right,
                _ => unreachable!(),
            };
        }
        Ok(left)
    }

    // term → power (('*' | '/' | '%') power)*
    fn term(&mut self) -> Result<f64, String> {
        let mut left = self.power()?;
        while let Some(Token::Op(op @ ('*' | '/' | '%'))) = self.peek().cloned() {
            self.next();
            let right = self.power()?;
            left = match op {
                '*' => left * right,
                '/' => left / right,
                '%' => left % right,
                _ => unreachable!(),
            };
        }
        Ok(left)
    }

    // power → unary ('^' power)* (right-associative via recursion)
    fn power(&mut self) -> Result<f64, String> {
        let base = self.unary()?;
        if let Some(Token::Op('^')) = self.peek() {
            self.next();
            let exp = self.power()?; // right-associative recursion
            Ok(base.powf(exp))
        } else {
            Ok(base)
        }
    }

    // unary → '-' unary | call
    fn unary(&mut self) -> Result<f64, String> {
        if let Some(Token::Op('-')) = self.peek() {
            self.next();
            let val = self.unary()?;
            Ok(-val)
        } else {
            self.call()
        }
    }

    // call → IDENT '(' expr (',' expr)* ')' | primary
    fn call(&mut self) -> Result<f64, String> {
        if let Some(Token::Ident(_)) = self.peek() {
            // Check if next-next token is '(' → function call
            if self.tokens.get(self.pos + 1) == Some(&Token::LParen) {
                let name = match self.next() {
                    Some(Token::Ident(s)) => s,
                    _ => unreachable!(),
                };
                self.next(); // consume '('

                let mut args = Vec::new();
                if self.peek() != Some(&Token::RParen) {
                    args.push(self.expr()?);
                    while let Some(Token::Comma) = self.peek() {
                        self.next();
                        args.push(self.expr()?);
                    }
                }
                self.expect(&Token::RParen)?;

                return call_function(&name, &args);
            }
        }
        self.primary()
    }

    // primary → NUMBER | IDENT (constant) | '(' expr ')'
    fn primary(&mut self) -> Result<f64, String> {
        match self.next() {
            Some(Token::Number(n)) => Ok(n),
            Some(Token::Ident(name)) => resolve_constant(&name),
            Some(Token::LParen) => {
                let val = self.expr()?;
                self.expect(&Token::RParen)?;
                Ok(val)
            }
            Some(tok) => Err(format!("unexpected token: {tok:?}")),
            None => Err("unexpected end of input".into()),
        }
    }
}

fn resolve_constant(name: &str) -> Result<f64, String> {
    match name.to_lowercase().as_str() {
        "pi" => Ok(PI),
        "e" => Ok(E),
        _ => Err(format!("unknown constant: {name}")),
    }
}

fn call_function(name: &str, args: &[f64]) -> Result<f64, String> {
    let expect_args = |n: usize| -> Result<(), String> {
        if args.len() != n {
            Err(format!(
                "{name}() expects {n} argument(s), got {}",
                args.len()
            ))
        } else {
            Ok(())
        }
    };

    match name.to_lowercase().as_str() {
        "sqrt" => {
            expect_args(1)?;
            Ok(args[0].sqrt())
        }
        "sin" => {
            expect_args(1)?;
            Ok(args[0].sin())
        }
        "cos" => {
            expect_args(1)?;
            Ok(args[0].cos())
        }
        "tan" => {
            expect_args(1)?;
            Ok(args[0].tan())
        }
        "log" => {
            expect_args(1)?;
            Ok(args[0].log10())
        }
        "ln" => {
            expect_args(1)?;
            Ok(args[0].ln())
        }
        "abs" => {
            expect_args(1)?;
            Ok(args[0].abs())
        }
        "floor" => {
            expect_args(1)?;
            Ok(args[0].floor())
        }
        "ceil" => {
            expect_args(1)?;
            Ok(args[0].ceil())
        }
        "round" => {
            expect_args(1)?;
            Ok(args[0].round())
        }
        _ => Err(format!("unknown function: {name}")),
    }
}

/// Evaluate a mathematical expression string and return the result.
pub fn evaluate(input: &str) -> Result<f64, String> {
    let tokens = tokenize(input)?;
    if tokens.is_empty() {
        return Err("empty expression".into());
    }
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval(input: &str) -> f64 {
        evaluate(input).unwrap()
    }

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn basic_arithmetic() {
        assert_eq!(eval("2+2"), 4.0);
        assert_eq!(eval("10 - 3"), 7.0);
        assert_eq!(eval("3 * 4"), 12.0);
        assert_eq!(eval("15 / 3"), 5.0);
        assert_eq!(eval("10 % 3"), 1.0);
    }

    #[test]
    fn operator_precedence() {
        assert_eq!(eval("2 + 3 * 4"), 14.0);
        assert_eq!(eval("(2 + 3) * 4"), 20.0);
        assert_eq!(eval("10 - 2 * 3"), 4.0);
    }

    #[test]
    fn exponentiation() {
        assert_eq!(eval("2 ^ 10"), 1024.0);
        assert_eq!(eval("2 ** 10"), 1024.0);
        // right-associative: 2^3^2 = 2^(3^2) = 2^9 = 512
        assert_eq!(eval("2 ^ 3 ^ 2"), 512.0);
    }

    #[test]
    fn unary_minus() {
        assert_eq!(eval("-5 + 3"), -2.0);
        assert_eq!(eval("-(3 + 2)"), -5.0);
        assert_eq!(eval("--5"), 5.0);
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn decimals() {
        assert!(approx_eq(eval("3.14 * 2"), 6.28));
        assert!(approx_eq(eval("0.1 + 0.2"), 0.3));
    }

    #[test]
    fn constants() {
        assert!(approx_eq(eval("pi"), std::f64::consts::PI));
        assert!(approx_eq(eval("e"), std::f64::consts::E));
        assert!(approx_eq(eval("pi * 2"), std::f64::consts::PI * 2.0));
    }

    #[test]
    fn functions() {
        assert_eq!(eval("sqrt(144)"), 12.0);
        assert_eq!(eval("abs(-42)"), 42.0);
        assert_eq!(eval("floor(3.7)"), 3.0);
        assert_eq!(eval("ceil(3.2)"), 4.0);
        assert_eq!(eval("round(3.5)"), 4.0);
        assert!(approx_eq(eval("sin(0)"), 0.0));
        assert!(approx_eq(eval("cos(0)"), 1.0));
        assert!(approx_eq(eval("tan(0)"), 0.0));
        assert!(approx_eq(eval("log(100)"), 2.0));
        assert!(approx_eq(eval("ln(e)"), 1.0));
    }

    #[test]
    fn nested_expressions() {
        assert_eq!(eval("sqrt(16) + 2 ^ 3"), 12.0);
        assert_eq!(eval("(3 + 4) * 2"), 14.0);
        assert_eq!(eval("sqrt(3^2 + 4^2)"), 5.0);
    }

    #[test]
    fn division_by_zero() {
        assert!(eval("1/0").is_infinite());
    }

    #[test]
    fn error_cases() {
        assert!(evaluate("").is_err());
        assert!(evaluate("abc").is_err());
        assert!(evaluate("2 +").is_err());
        assert!(evaluate("(2 + 3").is_err());
        assert!(evaluate("unknown(5)").is_err());
    }
}
