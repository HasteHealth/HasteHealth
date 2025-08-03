use std::fmt::Debug;

use peg::parser;

use crate::error::FHIRPathError;

#[derive(PartialEq, Debug, Clone)]
pub enum Literal {
    Null,
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Date(String),
    DateTime(String),
    Time(String),
    // Quantity(f64, String),
}

#[derive(PartialEq, Debug)]
pub enum Expression {
    Singular(Vec<Term>),
    Operation(Box<Operation>),
}

#[derive(PartialEq, Debug)]
pub enum Term {
    Invocation(Invocation),
    Literal(Literal),
    ExternalConstant(String),
    Parenthesized(Expression),
}

#[derive(PartialEq, Debug)]
pub enum Invocation {
    Identifier(Identifier),
    Function(FunctionInvocation),
    This,
    Index(Expression),
    IndexAccessor,
    Total,
}

#[derive(PartialEq, Debug)]
pub struct FunctionInvocation {
    pub name: Identifier,
    pub arguments: Vec<Expression>,
}

#[derive(PartialEq, Debug)]
pub struct Identifier(pub String);

#[derive(PartialEq, Debug)]
pub enum Polarity {
    Positive,
    Negative,
}

#[derive(PartialEq, Debug)]
pub struct QualifiedIdentifier(Vec<Identifier>);

#[derive(PartialEq, Debug)]
pub enum Operation {
    Polarity(Polarity, Expression),
    // Additive Operations
    Add(Expression, Expression),
    Subtraction(Expression, Expression),
    // Multiplicative Operations
    Multiplication(Expression, Expression),
    Division(Expression, Expression),
    DivisionTruncated(Expression, Expression),
    Modulo(Expression, Expression),
    // Type Operations
    Is(Expression, QualifiedIdentifier),
    As(Expression, QualifiedIdentifier),
    // Union Operations
    Union(Expression, Expression),
    // InEquality Operations
    LessThanEqual(Expression, Expression),
    GreaterThanEqual(Expression, Expression),
    LessThan(Expression, Expression),
    GreaterThan(Expression, Expression),
    // Equality Operations
    Equal(Expression, Expression),
    NotEqual(Expression, Expression),
    Equivalent(Expression, Expression),
    NotEquivalent(Expression, Expression),
    // Membership Operations
    In(Expression, Expression),
    Contains(Expression, Expression),
    // And Operations
    And(Expression, Expression),
    // Or Operations
    Or(Expression, Expression),
    XOr(Expression, Expression),
    // Implies Operations
    Implies(Expression, Expression),
}

parser! {
    grammar parser() for str {
        rule _w() = quiet!{[' ' | '\n' | '\t']*}

        rule invocation() -> Invocation
          = "$this" { Invocation::This }
          / "$index" { Invocation::IndexAccessor }
          / "$total" { Invocation::Total }

          / identifier:IDENTIFIER()"(" args:(operations() ** ",") ")" {
                Invocation::Function(FunctionInvocation {
                    name: Identifier(identifier),
                    arguments: args,
                })
            }
          / identifier:IDENTIFIER() { Invocation::Identifier(Identifier(identifier)) }

          rule index_accessor() -> Invocation
            = "[" _w() e:operations() _w() "]" { Invocation::Index(e) }

          rule dot_access () -> Invocation
            = "." invocation:invocation() { invocation }

        rule term () -> Term
          = literal:literal() { Term::Literal(literal) }
          / invocation:invocation() { Term::Invocation(invocation) }
          / external_constant:external_constant() { external_constant }
          / "(" _w() e:operations() _w() ")" { Term::Parenthesized(e) }

        rule singular_expression () -> Vec<Term>
          = _w() term:term() invocations:(dot_access() / index_accessor())* _w()  {?
            let mut terms = Vec::with_capacity(invocations.len() + 1);
            terms.push(term);
            for inv in invocations {
                terms.push(Term::Invocation(inv));
            }
            Ok(terms)
           } /
           term:term() {?
            Ok(vec![term])
           }

        rule number() -> i64
          = n:$(['0'..='9']+) {? n.parse().or(Err("i64")) }

        rule external_constant() -> Term
          = "%" literal:literal_string() {?
            if let Literal::String(s) = literal {
                Ok(Term::ExternalConstant(s))
            } else {
                Err("Expected a string literal for external constant.")
            }
        } / "%" identifier:IDENTIFIER() {
            Term::ExternalConstant(identifier)
        }

        rule literal_string() -> Literal
        // / [^'\"']
          = "'" s:$(("\\'" / [^('\n' | '\r' | '\0' | '\'' )])*) "'" {
            Literal::String(s.to_string())
        }

        rule literal_integer () -> Literal
          = int:$(['0'..='9']+) {?
            int.parse::<i64>()
                .map(Literal::Integer)
                .map_err(|_| "Failed to parse number.")
        }

        rule literal_float () -> Literal
          = float:$(['0'..='9']+ "." ['0'..='9']+) {?
            float.parse::<f64>()
                .map(Literal::Float)
                .map_err(|_| "Failed to parse number.")
        }

        rule literal_boolean () -> Literal
          = "true" { Literal::Boolean(true) }
          / "false" { Literal::Boolean(false) }

        rule literal_date () -> Literal
          = "@" date:DATE() {
            Literal::Date(date)
        }

        rule literal_time () -> Literal
          = "@T" time:TIME() {
            Literal::Time(time)
        }

        rule literal_datetime () -> Literal
          = "@" datetime:$(DATE() "T" (TIME() TIMEZONE_FORMAT()?)?) {
            Literal::DateTime(datetime.to_string())
          }

        rule DATE () -> String
          = date:$(['0'..='9']*<4,4> "-" ['0'..='9']*<2,2> "-" ['0'..='9']*<2,2>) {
            date.to_string()
        }

        rule TIMEZONE_FORMAT () -> String
          = timezone:$("Z" / (("+" / "-") ['0'..='9']*<2,2> ":" ['0'..='9']*<2,2>)) {
            timezone.to_string()
        }

        rule TIME () -> String
          = time:$(['0'..='9']*<2,2> (":" ['0'..='9']*<2,2> (":"['0'..='9']*<2,2> ("."['0'..='9']+)?)?)?) {
            time.to_string()
        }

        rule IDENTIFIER () -> String
          = id:$((['a'..='z' | 'A'..='Z'] / "_")((['a'..='z' | 'A'..='Z' | '0'..='9']) / "_")*) {
            id.to_string()
        }

        rule ESC () -> String
          = esc:$("\\"("\\" / "/" / "f" / "n" / "r" / "t")) {
            esc.to_string()
          }

        rule identifier_dot_access () -> String
          = "." id:IDENTIFIER() {
           id
        }

        rule qualified_identifier() -> QualifiedIdentifier
         =  id:IDENTIFIER() parts:(identifier_dot_access())*
        {
            let mut full_id = Vec::with_capacity(parts.len() + 1);
            full_id.push(Identifier(id));
            for part in parts {
                full_id.push(Identifier(part));
            }
            QualifiedIdentifier(full_id)
        }

        rule WILDCARD () -> String
        // Write rust code for wildcard which matches any character except line terminals.
         =  s:$([^('\n' | '\r' | '\0' )])
          {
            s.to_string()
        }

        pub rule literal () -> Literal
          = "{}" { Literal::Null }
          / literal_boolean()
          / literal_string()
          / literal_float()
          / literal_integer()
          / literal_datetime()
          / literal_time()
          / literal_date()

        pub rule operations() -> Expression = precedence!{
            x:(@) _w() "implies"  _w() y:@ {
                Expression::Operation(Box::new(Operation::Implies(x, y)))
            }
            --
            x:(@) _w() op:$("or" / "xor")  _w() y:@ {
                match op {
                    "or" => Expression::Operation(Box::new(Operation::Or(x, y))),
                    "xor" => Expression::Operation(Box::new(Operation::XOr(x, y))),
                    _ => unreachable!(),
                }
            }
            --
            x:(@) _w() "and"  _w() y:@ {
                Expression::Operation(Box::new(Operation::And(x, y)))
            }
            --
            x:(@) _w() op:$("in" / "contains")  _w() y:@ {
                match op {
                    "in" => Expression::Operation(Box::new(Operation::In(x, y))),
                    "contains" => Expression::Operation(Box::new(Operation::Contains(x, y))),
                    _ => unreachable!(),
                }
            }
            --
            x:(@) _w() op:$("=" / "~" / "!=" / "!~")  _w() y:@ {
                match op {
                    "=" => Expression::Operation(Box::new(Operation::Equal(x, y))),
                    "!=" => Expression::Operation(Box::new(Operation::NotEqual(x, y))),
                    "~" => Expression::Operation(Box::new(Operation::Equivalent(x, y))),
                    "!~" => Expression::Operation(Box::new(Operation::NotEquivalent(x, y))),
                    _ => unreachable!(),
                }
             }
             --
            x:(@) _w() op:$("<=" / "<" / ">" / ">=") _w() y:@{
                match op {
                    "<=" => Expression::Operation(Box::new(Operation::LessThanEqual(x, y))),
                    "<" => Expression::Operation(Box::new(Operation::LessThan(x, y))),
                    ">" => Expression::Operation(Box::new(Operation::GreaterThan(x, y))),
                    ">=" => Expression::Operation(Box::new(Operation::GreaterThanEqual(x, y))),
                    _ => unreachable!(),
                }
            }
            --
            x:(@) _w() "|" _w() y:@ {
                Expression::Operation(Box::new(Operation::Union(x, y)))
            }
            --
            x:(@) _w() op:$("is" / "as") _w() type_id:qualified_identifier() {
                match op {
                    "is" => Expression::Operation(Box::new(Operation::Is(x, type_id))),
                    "as" => Expression::Operation(Box::new(Operation::As(x, type_id))),
                    _ => unreachable!(),
                }
            }
            --
            x:(@) _w() op:$("+" / "-") _w() y:@ {
                match op{
                    "+" => Expression::Operation(Box::new(Operation::Add(x, y))),
                    "-" => Expression::Operation(Box::new(Operation::Subtraction(x, y))),
                    _ => unreachable!(),
                }
             }
             --
            x:(@) _w() op:$("*" / "/" / "div" / "mod") _w() y:@ {
                match op {
                    "*" => Expression::Operation(Box::new(Operation::Multiplication(x, y))),
                    "/" => Expression::Operation(Box::new(Operation::Division(x, y))),
                    "div" => Expression::Operation(Box::new(Operation::DivisionTruncated(x, y))),
                    "mod" => Expression::Operation(Box::new(Operation::Modulo(x, y))),
                    _ => unreachable!(),
                }
            }
            --
            pol:$("+" / "-") x:(@)  {
                match pol {
                    "+" => Expression::Operation(Box::new(Operation::Polarity(Polarity::Positive, x))),
                    "-" => Expression::Operation(Box::new(Operation::Polarity(Polarity::Negative, x))),
                    _ => unreachable!(),
                }
             }
             --
            n:singular_expression() { Expression::Singular(n) }
            "(" _w() e:operations() _w() ")" { e }
        }
     }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate test;
    use test::Bencher;

    #[test]
    fn literals() {
        let escaped_string = parser::literal("'hello \\' world'").unwrap();

        assert_eq!(
            escaped_string,
            Literal::String("hello \\' world".to_string())
        );

        let time = parser::literal("@T14:34:28").unwrap();
        assert_eq!(time, Literal::Time("14:34:28".to_string()));

        let date = parser::literal("@1980-01-01").unwrap();
        assert_eq!(date, Literal::Date("1980-01-01".to_string()));

        let datetime: Literal = parser::literal("@1980-01-01T12:00:23.23").unwrap();
        assert_eq!(
            datetime,
            Literal::DateTime("1980-01-01T12:00:23.23".to_string())
        );

        let datetime_zone: Literal = parser::literal("@2017-01-01T00:00:00.000Z").unwrap();
        assert_eq!(
            datetime_zone,
            Literal::DateTime("2017-01-01T00:00:00.000Z".to_string())
        );

        let datetime_zone: Literal = parser::literal("@2015-02-07T13:28:17-05:00").unwrap();
        assert_eq!(
            datetime_zone,
            Literal::DateTime("2015-02-07T13:28:17-05:00".to_string())
        );
    }

    #[test]
    fn type_operations() {
        let result = parser::operations("1 + 2 is Patient").unwrap();
        insta::assert_debug_snapshot!(result);

        let failure = parser::operations("1 + 2 is $this");
        assert_eq!(failure.is_err(), true);
    }

    #[test]
    fn operations() {
        let result = parser::operations("1 + 2 * (3 - 4) / 5").unwrap();
        insta::assert_debug_snapshot!(result);

        let result = parser::operations("%test.asdf").unwrap();
        insta::assert_debug_snapshot!(result);

        let result = parser::operations("%test.field + $this.value").unwrap();
        insta::assert_debug_snapshot!(result);
        let result =
            parser::operations("$this.field + %test._asdf.test(45,    $this.field)").unwrap();
        insta::assert_debug_snapshot!(result);

        let result = parser::operations("$this.field + %test._asdf.test()").unwrap();
        insta::assert_debug_snapshot!(result);

        let result = parser::operations("-$this.field+%test._asdf.test()").unwrap();
        insta::assert_debug_snapshot!(result);

        let result = parser::operations("-$this.field[45 + 23]").unwrap();
        insta::assert_debug_snapshot!(result);
    }

    #[bench]
    fn parser_test_performance(b: &mut Bencher) {
        b.iter(|| parser::operations("1 + 2 * (3 - 4) / 5").unwrap());
    }

    #[bench]
    fn parser_test_complex(b: &mut Bencher) {
        b.iter(|| parser::operations("$this.field + %test._asdf.test(45, $this.field) * 64 * $this.where($this.field = '23'.length())").unwrap());
    }
    #[bench]
    fn parser_test_simple(b: &mut Bencher) {
        b.iter(|| parser::operations("$this.field").unwrap());
    }
}

pub fn parse(input: &str) -> Result<Expression, FHIRPathError> {
    parser::operations(input).map_err(|e| FHIRPathError::ParseError(e))
}
