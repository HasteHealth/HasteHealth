//! HL7 v2 acknowledgement messages.
//!
//! A sender waits for an `ACK` message, not for the single `<ACK>` control
//! byte: the byte says "the bytes arrived", while the message says "the
//! application accepted it, and here is the control id it accepted". An
//! interface engine that receives a bare control character generally treats
//! the exchange as failed and retries, so a listener that answers with one
//! looks healthy while every message is resent.
//!
//! The reply echoes the sender's `MSH-10` in `MSA-2`, which is how the sender
//! matches an acknowledgement to the message it sent.

use crate::parser::ParsedHL7V2Message;

/// `MSA-1`, the acknowledgement code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AckCode {
    /// Accepted.
    ApplicationAccept,
    /// Rejected for a reason the sender could fix (a malformed message).
    ApplicationReject,
    /// Failed for a reason the sender cannot fix (storage was unavailable).
    /// A sender is expected to retry these.
    ApplicationError,
}

impl AckCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            AckCode::ApplicationAccept => "AA",
            AckCode::ApplicationReject => "AR",
            AckCode::ApplicationError => "AE",
        }
    }
}

/// What the listener needs from an inbound message to answer it.
///
/// Read with a tolerant scan rather than the full parser: a message can be
/// malformed enough to reject yet still carry a usable control id, and a
/// rejection the sender cannot match to its message is nearly useless.
#[derive(Debug, Clone, Default)]
pub struct InboundHeader {
    /// `MSH-3`, the sending application. Becomes the receiving application of
    /// the reply.
    pub sending_application: String,
    /// `MSH-4`, the sending facility.
    pub sending_facility: String,
    /// `MSH-5`, the receiving application. Becomes the sending application of
    /// the reply.
    pub receiving_application: String,
    /// `MSH-6`, the receiving facility.
    pub receiving_facility: String,
    /// `MSH-10`, the message control id. Echoed in `MSA-2`.
    pub control_id: String,
    /// `MSH-11`, the processing id (`P`, `T`, `D`).
    pub processing_id: String,
    /// `MSH-12`, the version. The reply is sent in the sender's version.
    pub version: String,
}

/// Fields of a raw `MSH` segment, by HL7 number.
///
/// `MSH-1` is the separator itself, so `field(2)` is the first value after it.
struct MshFields<'a> {
    fields: Vec<&'a str>,
}

impl<'a> MshFields<'a> {
    /// Splits the `MSH` line of `message`, or `None` when there isn't one.
    fn parse(message: &'a str) -> Option<Self> {
        let line = message
            .split(['\r', '\n'])
            .find(|line| line.starts_with("MSH"))?;

        // The character after `MSH` is the field separator, by definition.
        let separator = line.chars().nth(3)?;

        Some(MshFields {
            fields: line.split(separator).collect(),
        })
    }

    /// The value of `MSH-<number>`, empty when absent.
    ///
    /// The split yields `["MSH", <encoding chars>, ...]`, so `MSH-2` is at
    /// index 1 and field `N` is at `N - 1`.
    fn field(&self, number: usize) -> &str {
        number
            .checked_sub(1)
            .and_then(|index| self.fields.get(index))
            .copied()
            .unwrap_or_default()
    }
}

impl InboundHeader {
    /// Reads what can be read from a possibly malformed message.
    #[must_use]
    pub fn scan(message: &str) -> Self {
        let Some(msh) = MshFields::parse(message) else {
            return Self::default();
        };

        Self {
            sending_application: msh.field(3).to_string(),
            sending_facility: msh.field(4).to_string(),
            receiving_application: msh.field(5).to_string(),
            receiving_facility: msh.field(6).to_string(),
            control_id: msh.field(10).to_string(),
            processing_id: msh.field(11).to_string(),
            version: msh.field(12).to_string(),
        }
    }

    /// Reads the header from an already-parsed message.
    #[must_use]
    pub fn from_parsed(message: &ParsedHL7V2Message) -> Self {
        let serialized: String = crate::serialize::SerializeMessage(&message.0).into();
        Self::scan(&serialized)
    }
}

/// The HL7 version used when the sender did not state one.
const DEFAULT_VERSION: &str = "2.5.1";

/// Builds an `ACK` message replying to `header`.
///
/// Addressing is reversed -- the sender's application and facility become the
/// receiver's -- so the reply routes back the way the message came. `text` is
/// placed in `MSA-3` and is what an operator reads when a message is rejected,
/// so it should say what was wrong rather than that something was.
#[must_use]
pub fn build_ack(
    header: &InboundHeader,
    code: AckCode,
    text: Option<&str>,
    timestamp: &str,
) -> String {
    let version = if header.version.is_empty() {
        DEFAULT_VERSION
    } else {
        header.version.as_str()
    };

    // A reply needs its own control id. Deriving it from the inbound one keeps
    // it unique per message without needing a counter or a clock.
    let ack_control_id = if header.control_id.is_empty() {
        format!("ACK{timestamp}")
    } else {
        format!("ACK{}", header.control_id)
    };

    let processing_id = if header.processing_id.is_empty() {
        "P"
    } else {
        header.processing_id.as_str()
    };

    let msh = format!(
        "MSH|^~\\&|{sending_app}|{sending_fac}|{receiving_app}|{receiving_fac}|{timestamp}||ACK|{ack_control_id}|{processing_id}|{version}",
        // Reversed: we are replying to whoever sent to us.
        sending_app = escape(&header.receiving_application),
        sending_fac = escape(&header.receiving_facility),
        receiving_app = escape(&header.sending_application),
        receiving_fac = escape(&header.sending_facility),
    );

    let msa = match text {
        Some(text) if !text.is_empty() => format!(
            "MSA|{}|{}|{}",
            code.as_str(),
            escape(&header.control_id),
            escape(text)
        ),
        _ => format!("MSA|{}|{}", code.as_str(), escape(&header.control_id)),
    };

    // Segments are separated by a carriage return, and the message ends with
    // one; a trailing separator is what most receivers expect.
    format!("{msh}\r{msa}\r")
}

/// Replaces characters that would be read as delimiters.
///
/// A rejection reason is free text and an operator writes whatever explains the
/// failure; an unescaped `|` in it would silently shift every field after it.
fn escape(value: &str) -> String {
    value
        .replace('\\', "\\E\\")
        .replace('|', "\\F\\")
        .replace('^', "\\S\\")
        .replace('&', "\\T\\")
        .replace('~', "\\R\\")
        .replace(['\r', '\n'], " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    const MESSAGE: &str = "MSH|^~\\&|SENDAPP|SENDFAC|RECVAPP|RECVFAC|20260101120000||ADT^A01|MSG00001|P|2.5.1\rPID|1||42||DOE^JOHN\r";

    /// The named segment of a message. Segments are `\r`-separated, which
    /// `str::lines` does not split on.
    fn segment<'a>(message: &'a str, id: &str) -> &'a str {
        message
            .split(['\r', '\n'])
            .find(|line| line.starts_with(id))
            .unwrap_or_else(|| panic!("{id} segment in {message:?}"))
    }

    #[test]
    fn a_header_is_read_by_hl7_field_number() {
        let header = InboundHeader::scan(MESSAGE);

        assert_eq!(header.sending_application, "SENDAPP");
        assert_eq!(header.sending_facility, "SENDFAC");
        assert_eq!(header.receiving_application, "RECVAPP");
        assert_eq!(header.receiving_facility, "RECVFAC");
        assert_eq!(header.control_id, "MSG00001");
        assert_eq!(header.processing_id, "P");
        assert_eq!(header.version, "2.5.1");
    }

    /// The sender matches the reply to its message by the echoed control id,
    /// so this is the field that matters most in an acknowledgement.
    #[test]
    fn an_ack_echoes_the_control_id() {
        let header = InboundHeader::scan(MESSAGE);
        let ack = build_ack(&header, AckCode::ApplicationAccept, None, "20260101120001");

        assert!(ack.contains("MSA|AA|MSG00001"), "{ack}");
    }

    /// Addressing is reversed so the reply routes back to the sender.
    #[test]
    fn an_ack_reverses_the_addressing() {
        let header = InboundHeader::scan(MESSAGE);
        let ack = build_ack(&header, AckCode::ApplicationAccept, None, "20260101120001");

        let fields: Vec<&str> = segment(&ack, "MSH").split('|').collect();
        // MSH-3/4 of the reply are the inbound MSH-5/6.
        assert_eq!(fields[2], "RECVAPP");
        assert_eq!(fields[3], "RECVFAC");
        assert_eq!(fields[4], "SENDAPP");
        assert_eq!(fields[5], "SENDFAC");
    }

    #[test]
    fn an_ack_is_sent_in_the_senders_version() {
        let header = InboundHeader::scan(MESSAGE);
        let ack = build_ack(&header, AckCode::ApplicationAccept, None, "t");
        assert!(ack.contains("|2.5.1"), "{ack}");

        // A sender that states no version gets a default rather than an empty
        // field, which some receivers reject outright.
        let mut header = header;
        header.version = String::new();
        let ack = build_ack(&header, AckCode::ApplicationAccept, None, "t");
        assert!(ack.contains(DEFAULT_VERSION), "{ack}");
    }

    #[test]
    fn a_rejection_carries_its_reason() {
        let header = InboundHeader::scan(MESSAGE);
        let ack = build_ack(
            &header,
            AckCode::ApplicationReject,
            Some("PID-3 is required"),
            "t",
        );

        assert!(ack.contains("MSA|AR|MSG00001|PID-3 is required"), "{ack}");
    }

    /// A reason is free text; an unescaped delimiter in it would shift every
    /// field after it.
    #[test]
    fn a_reason_containing_a_delimiter_is_escaped() {
        let header = InboundHeader::scan(MESSAGE);
        let ack = build_ack(
            &header,
            AckCode::ApplicationError,
            Some("bad field|value^here"),
            "t",
        );

        // Segments are separated by `\r`, which `str::lines` does not split on.
        let msa = segment(&ack, "MSA");
        assert_eq!(
            msa.split('|').count(),
            4,
            "delimiters must not add fields: {msa}"
        );
        assert!(msa.contains("\\F\\"), "{msa}");
        assert!(msa.contains("\\S\\"), "{msa}");
    }

    /// A message malformed enough to reject may still carry a control id, and
    /// a rejection the sender cannot match to its message is nearly useless.
    #[test]
    fn a_control_id_is_recovered_from_a_malformed_message() {
        let malformed = "MSH|^~\\&|SENDAPP|SENDFAC|RECVAPP|RECVFAC|20260101||ADT^A01|MSG42|P|2.5.1\rPID|garbage";
        let header = InboundHeader::scan(malformed);

        assert_eq!(header.control_id, "MSG42");

        let ack = build_ack(&header, AckCode::ApplicationReject, Some("no PID-3"), "t");
        assert!(ack.contains("MSA|AR|MSG42"), "{ack}");
    }

    /// A message with no MSH at all still gets an answer, since the sender is
    /// waiting for one.
    #[test]
    fn a_message_without_an_msh_still_produces_an_ack() {
        let header = InboundHeader::scan("this is not HL7");
        let ack = build_ack(
            &header,
            AckCode::ApplicationReject,
            Some("no MSH"),
            "20260101",
        );

        assert!(ack.starts_with("MSH|^~\\&|"), "{ack}");
        assert!(ack.contains("MSA|AR"), "{ack}");
        // With no inbound control id, the reply still carries its own.
        assert!(ack.contains("ACK20260101"), "{ack}");
    }

    /// The reply must parse as HL7 v2, or the sender cannot read it.
    #[test]
    fn an_ack_round_trips_through_the_parser() {
        let header = InboundHeader::scan(MESSAGE);
        let ack = build_ack(&header, AckCode::ApplicationAccept, None, "20260101120001");

        let parsed = ParsedHL7V2Message::try_from(ack.as_str()).expect("ACK parses");
        let segments = parsed.0.segments.expect("segments");
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].id.value.as_deref(), Some("MSH"));
        assert_eq!(segments[1].id.value.as_deref(), Some("MSA"));
    }
}
