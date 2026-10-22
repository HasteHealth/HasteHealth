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
    /// Value and unit; a unit is UCUM (from a string) or a calendar duration keyword.
    Quantity(f64, Unit),
}

#[derive(PartialEq, Debug, Clone)]
pub enum Unit {
    Ucum(String),
    Calendar(String),
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
pub struct QualifiedIdentifier(pub Vec<Identifier>);

#[derive(PartialEq, Debug)]
pub enum Operation {
    Polarity(Polarity, Expression),
    // Additive Operations
    Add(Expression, Expression),
    Subtraction(Expression, Expression),
    Concatenate(Expression, Expression),
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
        // Whitespace and <https://hl7.org/fhirpath/N1/#comments>.
        rule _w() = quiet!{(
            [' ' | '\n' | '\r' | '\t']
            / "//" [^'\n']*
            / "/*" (!"*/" [_])* "*/"
        )*}

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
          = "'" s:$(("\\" [_] / [^('\n' | '\r' | '\0' | '\'' | '\\')])*) "'" {?
            unescape(s).map(Literal::String)
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

        // <https://hl7.org/fhirpath/N1/#quantity>: `4.5 'mg'` or `2 years`.
        rule literal_quantity () -> Literal
          = value:$(['0'..='9']+ ("." ['0'..='9']+)?) _w() unit:quantity_unit() {?
            value.parse::<f64>()
                .map(|value| Literal::Quantity(value, unit))
                .map_err(|_| "Failed to parse number.")
        }

        rule quantity_unit () -> Unit
          = "'" s:$(("\\" [_] / [^('\n' | '\r' | '\0' | '\'' | '\\')])*) "'" {?
            unescape(s).map(Unit::Ucum)
          }
          / unit:$(("year" / "month" / "week" / "day" / "hour" / "minute" / "second" / "millisecond") "s"?) !['a'..='z' | 'A'..='Z' | '0'..='9' | '_'] {
            Unit::Calendar(unit.to_string())
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
          = date:$(['0'..='9']*<4,4> ("-" ['0'..='9']*<2,2> ("-" ['0'..='9']*<2,2>)?)?) {
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
          / literal_quantity()
          / literal_float()
          / literal_integer()
          / literal_datetime()
          / literal_time()
          / literal_date()

        pub rule expression() -> Expression = _w() e:operations() _w() { e }

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
            x:(@) _w() op:$("<=" / ">=" / "<" / ">" ) _w() y:@{
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
            x:(@) _w() op:$("+" / "-" / "&") _w() y:@ {
                match op{
                    "+" => Expression::Operation(Box::new(Operation::Add(x, y))),
                    "-" => Expression::Operation(Box::new(Operation::Subtraction(x, y))),
                    "&" => Expression::Operation(Box::new(Operation::Concatenate(x, y))),
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

pub fn parse(input: &str) -> Result<Expression, FHIRPathError> {
    parser::expression(input).map_err(FHIRPathError::ParseError)
}

/// Resolves the escapes of <https://hl7.org/fhirpath/N1/#string>; a `\` before
/// any other character is dropped.
fn unescape(raw: &str) -> Result<String, &'static str> {
    if !raw.contains('\\') {
        return Ok(raw.to_string());
    }

    let mut unescaped = String::with_capacity(raw.len());
    let mut chars = raw.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            unescaped.push(c);
            continue;
        }

        // The grammar always puts a character after a '\\'.
        let escaped = match chars.next() {
            Some('r') => '\r',
            Some('n') => '\n',
            Some('t') => '\t',
            Some('f') => '\u{c}',
            Some('u') => {
                let hex = chars.by_ref().take(4).collect::<String>();
                u32::from_str_radix(&hex, 16)
                    .ok()
                    .and_then(char::from_u32)
                    .ok_or("Invalid \\u escape.")?
            }
            Some(escaped) => escaped,
            None => break,
        };
        unescaped.push(escaped);
    }

    Ok(unescaped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literals() {
        let escaped_string = parser::literal("'hello \\' world'").unwrap();

        assert_eq!(escaped_string, Literal::String("hello ' world".to_string()));

        assert_eq!(
            parser::literal("4.5 'mg'").unwrap(),
            Literal::Quantity(4.5, Unit::Ucum("mg".to_string()))
        );
        assert_eq!(
            parser::literal("2 years").unwrap(),
            Literal::Quantity(2.0, Unit::Calendar("years".to_string()))
        );
        // A trailing word is not a unit.
        assert!(parser::literal("2 dayz").is_err());

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
        assert!(failure.is_err());
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
}
