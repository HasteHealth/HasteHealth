use std::{cell::Cell, fmt::Debug};

use peg::parser;

use crate::error::FHIRPathError;

/// How many levels an expression's tree may have: a path is one, and so is each operator
/// over it and each expression around it (parentheses, an index, a function argument).
/// Parsing, evaluating and dropping an expression each recurse per level, so the stack
/// they need grows with it. The deepest expression in the FHIR core packages has 34.
pub const MAX_DEPTH: usize = 100;

/// A parsed piece of an expression and the height of its tree.
type Tree<T> = (T, usize);

/// What a parse tracks besides its position.
#[derive(Default)]
pub struct Depth {
    /// How many expressions the parser is inside of.
    level: Cell<usize>,
    /// Set once the expression is deeper than [`MAX_DEPTH`]. Nothing parses after that,
    /// so the parse ends there however much of the expression is left.
    exceeded: Cell<bool>,
}

impl Depth {
    /// Fails once the expression is known to be too deep.
    fn within(&self) -> Result<(), &'static str> {
        if self.exceeded.get() {
            return Err("a shallower expression");
        }
        Ok(())
    }

    /// Goes into a nested expression, and returns the level to [`Depth::leave`] to.
    fn enter(&self) -> Result<usize, &'static str> {
        let level = self.level.get();
        if level >= MAX_DEPTH {
            self.exceeded.set(true);
            return Err("a shallower expression");
        }
        self.level.set(level + 1);
        Ok(level)
    }

    fn leave(&self, level: usize) {
        self.level.set(level);
    }

    /// The height of a node whose tallest child has the height `below`.
    fn height_over(&self, below: usize) -> Result<usize, &'static str> {
        if below >= MAX_DEPTH {
            self.exceeded.set(true);
            return Err("a shallower expression");
        }
        Ok(below + 1)
    }

    /// Puts an operation over its operand. An operator's action cannot fail, so when the
    /// tree would be too tall the operand stands in for it: what is built stays bounded
    /// while the parse runs on, and [`parse`] reports the expression as too deep.
    fn unary(
        &self,
        operation: impl FnOnce(Expression) -> Operation,
        operand: Tree<Expression>,
    ) -> Tree<Expression> {
        match self.height_over(operand.1) {
            Ok(height) => (
                Expression::Operation(Box::new(operation(operand.0))),
                height,
            ),
            Err(_) => operand,
        }
    }

    /// Like [`Depth::unary`], with the left operand standing in.
    fn binary(
        &self,
        operation: fn(Expression, Expression) -> Operation,
        left: Tree<Expression>,
        right: Tree<Expression>,
    ) -> Tree<Expression> {
        match self.height_over(left.1.max(right.1)) {
            Ok(height) => (
                Expression::Operation(Box::new(operation(left.0, right.0))),
                height,
            ),
            Err(_) => left,
        }
    }

    /// Puts the signs written before an operand over it, the first one outermost.
    fn signed(&self, signs: &str, operand: Tree<Expression>) -> Tree<Expression> {
        signs.bytes().rev().fold(operand, |operand, sign| {
            let polarity = if sign == b'+' {
                Polarity::Positive
            } else {
                Polarity::Negative
            };
            self.unary(|operand| Operation::Polarity(polarity, operand), operand)
        })
    }
}

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
    grammar parser(depth: &Depth) for str {
        // Whitespace and <https://hl7.org/fhirpath/N1/#comments>.
        rule _w() = quiet!{(
            [' ' | '\n' | '\r' | '\t']
            / "//" [^'\n']*
            / "/*" (!"*/" [_])* "*/"
        )*}

        // An expression inside another one, a level further down. The level is restored
        // whether or not the expression parses, and what a failure expected is left as
        // the expression reported it.
        rule nested() -> Tree<Expression>
          = level:({? depth.enter() }) e:tree()? e:quiet!{({?
            depth.leave(level);
            e.ok_or("expression")
          })} { e }

        rule invocation() -> Tree<Invocation>
          = "$this" { (Invocation::This, 0) }
          / "$index" { (Invocation::IndexAccessor, 0) }
          / "$total" { (Invocation::Total, 0) }

          / identifier:IDENTIFIER() "(" args:((_w() o:nested() _w() { o }) ** ",") _w() ")" {
                let height = args.iter().map(|arg| arg.1).max().unwrap_or(0);
                (
                    Invocation::Function(FunctionInvocation {
                        name: Identifier(identifier),
                        arguments: args.into_iter().map(|arg| arg.0).collect(),
                    }),
                    height,
                )
            }
          / identifier:IDENTIFIER() { (Invocation::Identifier(Identifier(identifier)), 0) }

          rule index_accessor() -> Tree<Invocation>
            = "[" _w() e:nested() _w() "]" { (Invocation::Index(e.0), e.1) }

          rule dot_access () -> Tree<Invocation>
            = "." invocation:invocation() { invocation }

        rule term () -> Tree<Term>
          = literal:literal() { (Term::Literal(literal), 0) }
          / invocation:invocation() { (Term::Invocation(invocation.0), invocation.1) }
          / external_constant:external_constant() { (external_constant, 0) }
          / "(" _w() e:nested() _w() ")" { (Term::Parenthesized(e.0), e.1) }

        rule singular_expression () -> Tree<Vec<Term>>
          = ({? depth.within() }) terms:singular_terms() { terms }

        rule singular_terms () -> Tree<Vec<Term>>
          = _w() term:term() invocations:(dot_access() / index_accessor())* _w()  {?
            let mut height = term.1;
            let mut terms = Vec::with_capacity(invocations.len() + 1);
            terms.push(term.0);
            for inv in invocations {
                height = height.max(inv.1);
                terms.push(Term::Invocation(inv.0));
            }
            Ok((terms, depth.height_over(height)?))
           } /
           term:term() {?
            Ok((vec![term.0], depth.height_over(term.1)?))
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
          = "'" s:$(("\\" [_] / [^('\0' | '\'' | '\\')])*) "'" {?
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

        // Simple or backtick-delimited <https://hl7.org/fhirpath/N1/#identifiers>.
        rule IDENTIFIER () -> String
          = SIMPLE_IDENTIFIER()
          / "`" s:$(("\\" [_] / [^('\0' | '`' | '\\')])*) "`" {? unescape(s) }

        rule SIMPLE_IDENTIFIER () -> String
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

        pub rule expression() -> Expression = _w() e:tree() _w() { e.0 }

        pub rule operations() -> Expression = e:tree() { e.0 }

        rule tree() -> Tree<Expression> = precedence!{
            x:(@) _w() "implies"  _w() y:@ {
                depth.binary(Operation::Implies, x, y)
            }
            --
            x:(@) _w() op:$("or" / "xor")  _w() y:@ {
                match op {
                    "or" => depth.binary(Operation::Or, x, y),
                    "xor" => depth.binary(Operation::XOr, x, y),
                    _ => unreachable!(),
                }
            }
            --
            x:(@) _w() "and"  _w() y:@ {
                depth.binary(Operation::And, x, y)
            }
            --
            x:(@) _w() op:$("in" / "contains")  _w() y:@ {
                match op {
                    "in" => depth.binary(Operation::In, x, y),
                    "contains" => depth.binary(Operation::Contains, x, y),
                    _ => unreachable!(),
                }
            }
            --
            x:(@) _w() op:$("=" / "~" / "!=" / "!~")  _w() y:@ {
                match op {
                    "=" => depth.binary(Operation::Equal, x, y),
                    "!=" => depth.binary(Operation::NotEqual, x, y),
                    "~" => depth.binary(Operation::Equivalent, x, y),
                    "!~" => depth.binary(Operation::NotEquivalent, x, y),
                    _ => unreachable!(),
                }
             }
             --
            x:(@) _w() op:$("<=" / ">=" / "<" / ">" ) _w() y:@{
                match op {
                    "<=" => depth.binary(Operation::LessThanEqual, x, y),
                    "<" => depth.binary(Operation::LessThan, x, y),
                    ">" => depth.binary(Operation::GreaterThan, x, y),
                    ">=" => depth.binary(Operation::GreaterThanEqual, x, y),
                    _ => unreachable!(),
                }
            }
            --
            x:(@) _w() "|" _w() y:@ {
                depth.binary(Operation::Union, x, y)
            }
            --
            x:(@) ({? depth.within() }) _w() op:$("is" / "as") _w() type_id:qualified_identifier() {
                match op {
                    "is" => depth.unary(|x| Operation::Is(x, type_id), x),
                    "as" => depth.unary(|x| Operation::As(x, type_id), x),
                    _ => unreachable!(),
                }
            }
            --
            x:(@) _w() op:$("+" / "-" / "&") _w() y:@ {
                match op{
                    "+" => depth.binary(Operation::Add, x, y),
                    "-" => depth.binary(Operation::Subtraction, x, y),
                    "&" => depth.binary(Operation::Concatenate, x, y),
                    _ => unreachable!(),
                }
             }
             --
            x:(@) _w() op:$("*" / "/" / "div" / "mod") _w() y:@ {
                match op {
                    "*" => depth.binary(Operation::Multiplication, x, y),
                    "/" => depth.binary(Operation::Division, x, y),
                    "div" => depth.binary(Operation::DivisionTruncated, x, y),
                    "mod" => depth.binary(Operation::Modulo, x, y),
                    _ => unreachable!(),
                }
            }
            --
            // Signs bind tighter than any operator. They are taken as a run with what
            // they apply to, where an operator of their own would recurse for each one.
            signs:$(("+" / "-")*) n:singular_expression() {
                depth.signed(signs, (Expression::Singular(n.0), n.1))
            }
            signs:$(("+" / "-")*) "(" _w() e:nested() _w() ")" { depth.signed(signs, e) }
        }
     }
}

pub fn parse(input: &str) -> Result<Expression, FHIRPathError> {
    let depth = Depth::default();
    let parsed = parser::expression(input, &depth);
    // Checked first: past the limit the parse fails wherever it happened to be, or
    // succeeds with a tree that stops short.
    if depth.exceeded.get() {
        return Err(FHIRPathError::TooCostly(format!(
            "expression is more than {MAX_DEPTH} levels deep"
        )));
    }
    parsed.map_err(FHIRPathError::ParseError)
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
                // `from_str_radix` alone would accept a leading `+`.
                (hex.len() == 4 && hex.bytes().all(|b| b.is_ascii_hexdigit()))
                    .then(|| u32::from_str_radix(&hex, 16).ok())
                    .flatten()
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
    use peg::{error::ParseError, str::LineCol};

    fn parse_literal(input: &str) -> Result<Literal, ParseError<LineCol>> {
        parser::literal(input, &Depth::default())
    }

    fn parse_operations(input: &str) -> Result<Expression, ParseError<LineCol>> {
        parser::operations(input, &Depth::default())
    }

    #[test]
    fn literals() {
        let escaped_string = parse_literal("'hello \\' world'").unwrap();

        assert_eq!(escaped_string, Literal::String("hello ' world".to_string()));

        assert_eq!(
            parse_literal("4.5 'mg'").unwrap(),
            Literal::Quantity(4.5, Unit::Ucum("mg".to_string()))
        );
        assert_eq!(
            parse_literal("2 years").unwrap(),
            Literal::Quantity(2.0, Unit::Calendar("years".to_string()))
        );
        // A trailing word is not a unit.
        assert!(parse_literal("2 dayz").is_err());

        let time = parse_literal("@T14:34:28").unwrap();
        assert_eq!(time, Literal::Time("14:34:28".to_string()));

        let date = parse_literal("@1980-01-01").unwrap();
        assert_eq!(date, Literal::Date("1980-01-01".to_string()));

        let datetime: Literal = parse_literal("@1980-01-01T12:00:23.23").unwrap();
        assert_eq!(
            datetime,
            Literal::DateTime("1980-01-01T12:00:23.23".to_string())
        );

        let datetime_zone: Literal = parse_literal("@2017-01-01T00:00:00.000Z").unwrap();
        assert_eq!(
            datetime_zone,
            Literal::DateTime("2017-01-01T00:00:00.000Z".to_string())
        );

        let datetime_zone: Literal = parse_literal("@2015-02-07T13:28:17-05:00").unwrap();
        assert_eq!(
            datetime_zone,
            Literal::DateTime("2015-02-07T13:28:17-05:00".to_string())
        );
    }

    #[test]
    fn type_operations() {
        let result = parse_operations("1 + 2 is Patient").unwrap();
        insta::assert_debug_snapshot!(result);

        let failure = parse_operations("1 + 2 is $this");
        assert!(failure.is_err());
    }

    #[test]
    fn operations() {
        let result = parse_operations("1 + 2 * (3 - 4) / 5").unwrap();
        insta::assert_debug_snapshot!(result);

        let result = parse_operations("%test.asdf").unwrap();
        insta::assert_debug_snapshot!(result);

        let result = parse_operations("%test.field + $this.value").unwrap();
        insta::assert_debug_snapshot!(result);
        let result =
            parse_operations("$this.field + %test._asdf.test(45,    $this.field)").unwrap();
        insta::assert_debug_snapshot!(result);

        let result = parse_operations("$this.field + %test._asdf.test()").unwrap();
        insta::assert_debug_snapshot!(result);

        let result = parse_operations("-$this.field+%test._asdf.test()").unwrap();
        insta::assert_debug_snapshot!(result);

        let result = parse_operations("-$this.field[45 + 23]").unwrap();
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn grammar_gaps() {
        assert_eq!(
            parse_operations("Patient.`\\u006eame`").unwrap(),
            parse_operations("Patient.name").unwrap()
        );
        assert!(parse_operations("`Patient`.name").is_ok());
        assert!(parse_operations("Patient.is(FHIR.`Patient`)").is_ok());
        assert!(parse_operations("%`vs-administrative-gender`").is_ok());
        assert!(parse_literal("'\\u+041'").is_err());
        assert_eq!(
            parse_literal("'\\u0041'").unwrap(),
            Literal::String("A".to_string())
        );
        assert_eq!(
            parse_literal("'A\nB'").unwrap(),
            Literal::String("A\nB".to_string())
        );
        assert!(parse_operations("(1|2).combine( -1)").is_ok());
        assert!(parse_operations("(1|2).combine(1, -1)").is_ok());
        assert!(parse_operations("(1|2).count( )").is_ok());
    }

    #[test]
    fn depth_limit() {
        let too_deep =
            |expression: &str| matches!(parse(expression), Err(FHIRPathError::TooCostly(_)));

        // A level for each expression inside another, each sign and each operator.
        for (before, innermost, after) in [
            ("(", "1", ")"),
            ("a[", "1", "]"),
            ("a.select(", "1", ")"),
            ("-", "1", ""),
            ("", "1", " + 1"),
            ("", "a", " | a"),
            ("", "1", " is Integer"),
        ] {
            let levels = |levels: usize| {
                format!(
                    "{}{innermost}{}",
                    before.repeat(levels),
                    after.repeat(levels)
                )
            };

            assert!(parse(&levels(MAX_DEPTH - 1)).is_ok());
            assert!(too_deep(&levels(MAX_DEPTH)));
            assert!(too_deep(&levels(1_000_000)));
        }

        // Expressions side by side do not add up.
        let chain = format!("1{}", " + 1".repeat(MAX_DEPTH - 3));
        assert!(parse(&format!("iif({chain}, {chain}, {chain}) | {chain}")).is_ok());

        // An expression that does not parse is still reported as that.
        assert!(matches!(
            parse("((1 1))"),
            Err(FHIRPathError::ParseError(_))
        ));
    }
}
