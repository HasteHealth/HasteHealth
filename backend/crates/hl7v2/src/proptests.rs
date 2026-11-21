//! Property tests for HL7v2: a generated message, with any delimiter set,
//! survives a parse and serialize round trip, directly and through FHIR
//! JSON; segment terminators are interchangeable; and the parser rejects
//! anything else without panicking.

use haste_fhir_model::r4::generated::resources::HL7V2;
use proptest::prelude::*;
use proptest::string::string_regex;
use proptest::test_runner::TestCaseError;

use crate::{parser::ParsedHL7V2Message, serialize::SerializeMessage};

/// What a field, component or subcomponent may hold: none of the characters
/// an [`Encoding`] can pick as a delimiter.
const TEXT: &str = "[A-Za-z0-9 .,:;'=+()_-]{0,12}";

/// The characters an [`Encoding`] picks its delimiters from.
const DELIMITERS: [char; 19] = [
    '|', '^', '~', '\\', '&', '#', '!', '%', '$', '@', '*', '?', '/', '<', '>', '[', ']', '{', '}',
];

/// A message's five delimiters: `MSH-1` and the four of `MSH-2`.
#[derive(Debug, Clone, Copy)]
struct Encoding {
    field: char,
    component: char,
    repetition: char,
    escape: char,
    subcomponent: char,
}

impl Encoding {
    const STANDARD: Self = Self {
        field: '|',
        component: '^',
        repetition: '~',
        escape: '\\',
        subcomponent: '&',
    };

    fn msh2(self) -> String {
        [
            self.component,
            self.repetition,
            self.escape,
            self.subcomponent,
        ]
        .iter()
        .collect()
    }
}

/// The standard delimiters most of the time, otherwise five distinct ones.
fn encoding() -> impl Strategy<Value = Encoding> {
    prop_oneof![
        3 => Just(Encoding::STANDARD),
        1 => prop::sample::subsequence(DELIMITERS.to_vec(), 5)
            .prop_shuffle()
            .prop_map(|chosen| Encoding {
                field: chosen[0],
                component: chosen[1],
                repetition: chosen[2],
                escape: chosen[3],
                subcomponent: chosen[4],
            }),
    ]
}

/// Text, which may hold HL7 escape sequences such as `\F\`; the parser keeps
/// them as they are.
fn text(escape: char) -> impl Strategy<Value = String> {
    let plain = string_regex(TEXT).expect("text generates");
    let escaped = prop::sample::select(vec!['F', 'S', 'T', 'R', 'E', 'H', 'N'])
        .prop_map(move |code| format!("{escape}{code}{escape}"));
    prop::collection::vec(prop_oneof![4 => plain, 1 => escaped], 1..3)
        .prop_map(|parts| parts.concat())
}

fn joined(
    part: impl Strategy<Value = String> + 'static,
    count: std::ops::Range<usize>,
    separator: char,
) -> impl Strategy<Value = String> {
    prop::collection::vec(part, count).prop_map(move |parts| parts.join(&separator.to_string()))
}

fn component(encoding: Encoding) -> impl Strategy<Value = String> {
    prop_oneof![
        3 => text(encoding.escape),
        1 => joined(text(encoding.escape), 2..4, encoding.subcomponent),
    ]
}

fn repetition(encoding: Encoding) -> impl Strategy<Value = String> {
    prop_oneof![
        3 => component(encoding),
        1 => joined(component(encoding), 2..5, encoding.component),
    ]
}

fn field(encoding: Encoding) -> impl Strategy<Value = String> {
    prop_oneof![
        4 => repetition(encoding),
        1 => joined(repetition(encoding), 2..4, encoding.repetition),
    ]
}

/// A segment after the header: an id and at least one field, which may be
/// empty.
fn segment(encoding: Encoding) -> impl Strategy<Value = String> {
    (
        "[A-Z][A-Z0-9]{2}",
        prop::collection::vec(field(encoding), 1..9),
    )
        .prop_filter("only the header is MSH", |(id, _)| id != "MSH")
        .prop_map(move |(id, fields)| {
            format!(
                "{id}{}{}",
                encoding.field,
                fields.join(&encoding.field.to_string())
            )
        })
}

/// A message: the header with its delimiters, then up to five segments,
/// joined by line feeds as the serializer writes them.
fn message() -> impl Strategy<Value = String> {
    encoding().prop_flat_map(|encoding| {
        let header = prop::collection::vec(field(encoding), 0..7).prop_map(move |fields| {
            let mut header = format!("MSH{}{}", encoding.field, encoding.msh2());
            for field in fields {
                header.push(encoding.field);
                header.push_str(&field);
            }
            header
        });
        (header, prop::collection::vec(segment(encoding), 0..6)).prop_map(|(header, segments)| {
            let mut lines = vec![header];
            lines.extend(segments);
            lines.join("\n")
        })
    })
}

fn parse(message: &str) -> Result<HL7V2, TestCaseError> {
    ParsedHL7V2Message::try_from(message)
        .map(|parsed| parsed.0)
        .map_err(|error| TestCaseError::fail(format!("{message:?} did not parse: {error}")))
}

fn serialize(message: &HL7V2) -> String {
    String::from(SerializeMessage(message))
}

proptest! {
    /// Parsing a message and serializing it again reproduces it exactly, for
    /// any delimiter set.
    #[test]
    fn parse_then_serialize_reproduces_the_message(message in message()) {
        prop_assert_eq!(serialize(&parse(&message)?), message);
    }

    /// Carriage returns, the wire's segment terminator, parse the same as
    /// line feeds, as do blank lines and a trailing terminator.
    #[test]
    fn segment_terminators_are_interchangeable(
        message in message(),
        terminator in prop::sample::select(vec!["\r", "\r\n", "\n\n"]),
        trailing in any::<bool>(),
    ) {
        let mut wire = message.replace('\n', terminator);
        if trailing {
            wire.push_str(terminator);
        }
        prop_assert_eq!(serialize(&parse(&wire)?), message);
    }

    /// A parsed message survives FHIR JSON, empty values in place.
    #[test]
    fn parsed_message_round_trips_through_json(message in message()) {
        let parsed = parse(&message)?;
        let json = serde_json::to_string(&parsed)
            .map_err(|error| TestCaseError::fail(format!("{message:?} did not serialize: {error}")))?;
        let again: HL7V2 = serde_json::from_str(&json)
            .map_err(|error| TestCaseError::fail(format!("{json} did not parse: {error}")))?;
        prop_assert_eq!(serialize(&again), message);
    }

    /// Parsing never panics, whatever the input.
    #[test]
    fn parsing_never_panics(input in ".*") {
        let _ = ParsedHL7V2Message::try_from(input.as_str());
    }

    /// Nor on input that starts like a message.
    #[test]
    fn parsing_header_like_input_never_panics(input in "MSH(.|\r|\n){0,40}") {
        let _ = ParsedHL7V2Message::try_from(input.as_str());
    }
}
