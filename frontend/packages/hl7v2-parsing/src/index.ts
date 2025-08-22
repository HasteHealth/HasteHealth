import {
  OperationError,
  outcomeFatal,
} from "@oxidized-health/operation-outcomes";

export type Hl7V2Component = string | string[];

export type Hl7V2Field = Hl7V2Component | Hl7V2Component[];

export type Hl7V2Segment = {
  name: string;
  fields: (Hl7V2Field | Hl7V2Field[])[];
};

export type Hl7V2Message = Record<Hl7V2Segment["name"], Hl7V2Segment["fields"]>;

type SpecialCharacters = {
  fieldSeperator: string;
  seperatorCharacter: string;
  repetitionCharacter: string;
  escapeCharacter: string;
  subComponentSeperatorCharacter: string;
  truncationCharacter?: string;
};

function parseComponent(
  specialCharacters: SpecialCharacters,
  component: string
): Hl7V2Component {
  const subCompoenents = component.split(
    specialCharacters.subComponentSeperatorCharacter
  );

  return subCompoenents.length === 1 ? subCompoenents[0] : subCompoenents;
}

function parseField(
  specialCharacters: SpecialCharacters,
  field: string
): Hl7V2Field | Hl7V2Field[] {
  const repititions = field
    .split(specialCharacters.repetitionCharacter)
    .map((repitition) => {
      const components = repitition.split(specialCharacters.seperatorCharacter);
      return components.length === 1
        ? parseComponent(specialCharacters, components[0])
        : components.map((c) => parseComponent(specialCharacters, c));
    });

  return repititions.length === 1 ? repititions[0] : repititions;
}

function parseSegment(
  specialCharacters: SpecialCharacters,
  segment: string
): Hl7V2Segment {
  const [name, ...fields] = segment.split(specialCharacters.fieldSeperator);
  return { name, fields: fields.map((f) => parseField(specialCharacters, f)) };
}

const SegmentEnd = /\r?\n|\r|\n/;

/**
 *
 */
function parseSingleMessage(hl7v2Message: string): Hl7V2Message {
  const mshEnd = hl7v2Message.search(SegmentEnd);
  const mshSegment = hl7v2Message.slice(0, mshEnd);
  if (!mshSegment.startsWith("MSH")) {
    throw new OperationError(outcomeFatal("invalid", "Invalid MSH Segment"));
  }

  const fieldSeperator = mshSegment[3];
  //  component separator, repetition separator, escape character, subcomponent separator, and truncation character.
  // Recommneded Characters: ^~\& #
  const specialCharEndIndex = mshSegment.indexOf(fieldSeperator, 4);
  const specialCharString = mshSegment.slice(4, specialCharEndIndex);

  const [
    seperatorCharacter,
    repetitionCharacter,
    escapeCharacter,
    subComponentSeperatorCharacter,
    truncationCharacter,
  ] = specialCharString.split("");

  const specialCharacters: SpecialCharacters = {
    fieldSeperator,
    seperatorCharacter,
    repetitionCharacter,
    escapeCharacter,
    subComponentSeperatorCharacter,
    truncationCharacter,
  };

  // Special handling of MSH segment
  const hl7v2MessageParsed: Hl7V2Message = {
    MSH: [
      fieldSeperator,
      specialCharString,
      ...mshSegment
        .slice(specialCharEndIndex + 1)
        .split(fieldSeperator)
        .map((f) => parseField(specialCharacters, f)),
    ],
  };

  const segments = hl7v2Message
    .slice(mshEnd + 1)
    .split(SegmentEnd)
    .map((segment) => parseSegment(specialCharacters, segment));

  return segments.reduce<Hl7V2Message>((acc, segment) => {
    acc[segment.name] = segment.fields;
    return acc;
  }, hl7v2MessageParsed);
}

export default function parseHl7V2Message(hl7v2Message: string): Hl7V2Message {
  switch (true) {
    case hl7v2Message.startsWith("MSH"): {
      return parseSingleMessage(hl7v2Message);
    }
    // Batch messages
    case hl7v2Message.startsWith("FHS"): {
      throw new OperationError(outcomeFatal("invalid", "FHS not supported"));
    }
    default: {
      throw new OperationError(outcomeFatal("invalid", "Invalid Message"));
    }
  }
}
