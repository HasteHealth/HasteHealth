import { IMetaValue, Location } from "./interface.js";
import { isPrimitiveType } from "./utilities.js";

/**
 * Logic for special handling of primitive types.
 * @param loc
 * @param field
 * @returns
 */
function descendPrimitiveLoc(loc: Location, field: string): Location {
  if (field === "value") return loc;
  // Handle Array and singular
  // Array primitives parents will be number over field.
  if (typeof loc[loc.length - 1] === "number") {
    return [
      ...loc.slice(0, loc.length - 2),
      `_${loc[loc.length - 2]}`,
      loc[loc.length - 1],
      field,
    ];
  }

  return [...loc.slice(0, loc.length - 1), `_${loc[loc.length - 1]}`, field];
}

/**
 * Descend into a location. If the value is a primitive, special handling is needed.
 * @param v Meta value with loc to descend.
 * @param field Field to descend into.
 * @returns Location of the field.
 */
export function descendLoc<T>(v: IMetaValue<T>, field: string): Location {
  const loc = v.location() || [];
  // Need special handling for .value and extensions which are under _fieldName for primitives.
  const meta = v.meta();
  if (meta && isPrimitiveType(meta.fhirVersion, meta.type)) {
    return descendPrimitiveLoc(loc, field);
  }
  return [...loc, field];
}
