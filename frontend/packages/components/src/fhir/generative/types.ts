/* eslint @typescript-eslint/no-explicit-any: 0 */
import { Mutation } from "@oxidized-health/fhir-patch-building";
import { Loc } from "@oxidized-health/fhir-pointer";
import {
  ElementDefinitionType,
  StructureDefinition,
} from "@oxidized-health/fhir-types/r4/types";

import { ClientProps } from "../types";

export type MetaProps<T, R> = Readonly<
  {
    sd: StructureDefinition;
    elementIndex: number;
    value: unknown;
    pointer: Loc<T, R, any>;
    showLabel?: boolean;
    showInvalid?: boolean;
    onChange: (patches: Mutation<T, R>) => void;
    type: ElementDefinitionType | undefined;
  } & ClientProps
>;
