# haste-hl7v2

HL7 v2 message handling for Haste Health: a parser that turns pipe-delimited text into the `HL7V2` resource from [`haste-fhir-model`](https://crates.io/crates/haste-fhir-model) (segments, fields, repetitions, components and subcomponents), a serializer that writes it back out, `ACK` message generation, and async MLLP framing for listeners.

Because the parsed form is a FHIR resource, it serializes to JSON like any other and can be queried with FHIRPath or rendered through a template. The server's `$hl7v2-parse` operation returns it, `haste-fhir-converter` renders Jinja templates against it, and `haste-health hl7v2 receiver` and `sender` use the MLLP and ACK modules to bridge MLLP feeds into the FHIR server.

## Install

```bash
cargo add haste-hl7v2
```

## Usage

Parse a message. Fields are indexed from zero after the segment id, so PID-5 is `fields[4]`; an empty field, component or subcomponent is kept so positions never shift:

```rust
use haste_hl7v2::parser::ParsedHL7V2Message;
use haste_hl7v2::serialize::SerializeMessage;

let raw = "MSH|^~\\&|SENDAPP|SENDFAC|RECVAPP|RECVFAC|20260101120000||ADT^A01|MSG00001|P|2.5.1\rPID|1||42||DOE^JOHN\r";

// A malformed message is an OperationOutcomeError with issue type `invalid`.
let message = ParsedHL7V2Message::try_from(raw)?;
let segments = message.0.segments.as_ref().unwrap();
assert_eq!(segments[1].id.value.as_deref(), Some("PID"));

// Write a parsed or edited message back out, using the separators its MSH declares.
let text: String = SerializeMessage(&message.0).into();
```

Acknowledge it. `InboundHeader::scan` tolerates malformed input, so a rejection still carries the control id the sender matches on:

```rust
use haste_hl7v2::ack::{AckCode, InboundHeader, build_ack};

let header = InboundHeader::scan(raw);
let ack = build_ack(&header, AckCode::ApplicationAccept, None, "20260101120001");
assert!(ack.contains("MSA|AA|MSG00001"));
```

Read and write MLLP frames on a Tokio socket:

```rust
use haste_hl7v2::mllp_async::{FrameEnd, encode_frame, read_frame};
use tokio::io::AsyncWriteExt;

match read_frame(&mut socket).await? {
    FrameEnd::Frame(payload) => {
        let raw = String::from_utf8_lossy(&payload);
        // parse, convert, store, then answer
        socket.write_all(&encode_frame(ack.as_bytes())).await?;
    }
    FrameEnd::Closed => {} // the sender hung up between frames
}
```

## API

| Module       | What it does                                                                                                                                                   |
| ------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `parser`     | `ParsedHL7V2Message::try_from(&str)`: reads the separators from `MSH`, splits segments on `\r` or `\n`, and keeps `MSH-2` as one value.                        |
| `serialize`  | `SerializeMessage(&HL7V2)` into `String`, plus the per-segment and per-field helpers it is built from.                                                         |
| `ack`        | `InboundHeader::scan` or `from_parsed`, `AckCode` (`AA`, `AR`, `AE`) and `build_ack`, which reverses the addressing and escapes delimiters in the reason text. |
| `mllp_async` | `read_frame` (cancel-safe, refuses frames above `MAX_FRAME_BYTES`, 16 MiB) and `encode_frame`.                                                                 |

## Command line

```bash
haste-health hl7v2 receiver --port 2575 --template-dir ./templates --main adt
haste-health hl7v2 sender --address 127.0.0.1 --port 2575 --file message.hl7
```

The receiver converts each message with your Jinja templates and submits the result to the active profile's server. It is experimental; see the guide below.

## Links

- [API reference](https://rust.haste.health/haste_hl7v2/)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/backend/crates/hl7v2)
- [Rust SDK overview](https://haste.health/docs/api/sdks/rust)
- [HL7v2 ingestion guide](https://haste.health/docs/guides/hl7v2) and the [CLI reference](https://github.com/HasteHealth/HasteHealth/blob/main/backend/documentation/cli_commands.md#haste-health-hl7v2)
- TypeScript equivalent: [@haste-health/hl7v2-parsing](https://www.npmjs.com/package/@haste-health/hl7v2-parsing)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
