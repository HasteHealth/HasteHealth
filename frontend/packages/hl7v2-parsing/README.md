# @haste-health/hl7v2-parsing

Parses an HL7 v2 message (ADT, ORU, ORM, ...) into plain JavaScript data. The encoding characters are read from the MSH segment, so any field, component, repetition and subcomponent separators work. The result maps each segment name to its fields, nesting arrays only where the message has repetitions, components or subcomponents.

The Haste Health server does the same job in Rust with [haste-hl7v2](https://crates.io/crates/haste-hl7v2) behind its `$hl7v2-parse` operation and MLLP receiver. This package is for TypeScript code, in Node or the browser.

## Install

```bash
npm install @haste-health/hl7v2-parsing
```

## Usage

```ts
import parseHl7V2Message from "@haste-health/hl7v2-parsing";

const message = parseHl7V2Message(
  [
    "MSH|^~\\&|SIMHOSP|SFAC|RAPP|RFAC|20200508130643||ADT^A01|5|T|2.3",
    "PID|1|2590157853^^^SIMULATOR MRN^MRN|2590157853^^^SIMULATOR MRN^MRN~2478684691^^^NHSNBR^NHSNMBR||Esterkin^AKI Scenario 6^^^Miss",
  ].join("\r"),
);

message.MSH[8]; // ["ADT", "A01"]            MSH-9, message type
message.PID[4][0]; // "Esterkin"               PID-5.1, family name
message.PID[2]; // [[...], [...]]              PID-3 repeats: one array per identifier
message.PID[0]; // "1"                         PID-1, a plain string
```

Indexes are the HL7 field numbers minus one: `PID[4]` is PID-5, and in MSH `MSH[0]` is the field separator (MSH-1) and `MSH[1]` the encoding characters (MSH-2). A field is a string when it has one value, an array of components when it has several, and an array of repetitions (each a string or component array) when the repetition separator appears. Subcomponents nest one level further.

Exported types: `Hl7V2Message`, `Hl7V2Segment`, `Hl7V2Field`, `Hl7V2Component`.

## Limitations

- Segments are keyed by name, so a message with several segments of the same name (several `OBX` or `NK1`, say) keeps only the last one.
- Batch messages (`FHS`/`BHS`) are rejected, and a message that does not start with `MSH` throws an `OperationError` from [`@haste-health/operation-outcomes`](https://www.npmjs.com/package/@haste-health/operation-outcomes).
- Escape sequences (`\F\`, `\S\`, ...) are left as written.

## Links

- [API reference](https://typescript.haste.health/modules/_haste-health_hl7v2-parsing.html)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/frontend/packages/hl7v2-parsing)
- [HL7v2 ingestion guide](https://haste.health/docs/guides/hl7v2) and the [TypeScript SDK overview](https://haste.health/docs/api/sdks/typescript)
- Rust equivalent: [haste-hl7v2](https://crates.io/crates/haste-hl7v2)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
