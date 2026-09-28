/**
 * Registers every type's custom views.
 *
 * Importing this module is what puts them in the registry, so the console
 * imports it once for the side effect. To add a type: create a module here
 * that calls `registerResourceViews`, then import it below.
 */
import "./AccessPolicyV2";
import "./OperationDefinition";
import "./ViewDefinition";

export { typeTemplate } from "./templates";
