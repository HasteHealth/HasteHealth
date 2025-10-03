// use oxidized_fhir_model::r4::generated::resources::AccessPolicyV2;

// /* eslint-disable @typescript-eslint/no-explicit-any */
// import * as pt from "@iguhealth/fhir-pointer";
// import { AccessPolicyV2, Expression, id } from "@iguhealth/fhir-types/r4/types";
// import * as fp from "@iguhealth/fhirpath";
// import { OperationError, outcomeFatal } from "@iguhealth/operation-outcomes";
// import xFhirQuery from "@iguhealth/x-fhir-query";

// import { PolicyContext, PolicyResult } from "../types.js";

// export async function evaluateExpression<
//   CTX,
//   Role,
//   Context extends PolicyContext<CTX, Role> = PolicyContext<CTX, Role>,
// >(
//   context: Context,
//   policy: AccessPolicyV2,
//   loc: pt.Loc<AccessPolicyV2, Expression | undefined, any>,
//   resolveVariable: (
//     context: Context,
//     policy: AccessPolicyV2,
//     variableId: id,
//   ) => Promise<{ context: Context; value: unknown }>,
// ): Promise<PolicyResult<CTX, Role, string | boolean | undefined>> {
//   let nextContext = context;
//   const expression = pt.get(loc, policy);
//   if (!expression) return { context, result: undefined };

//   const fpOptions: fp.Options = {
//     variables: async (variableId) => {
//       const res = await resolveVariable(nextContext, policy, variableId as id);
//       nextContext = res.context;

//       return res.value;
//     },
//   };

//   switch (expression.language) {
//     case "application/x-fhir-query": {
//       if (!expression.expression) {
//         throw new OperationError(
//           outcomeFatal(
//             "exception",
//             `Invalid access policy expression at '${loc}'.`,
//           ),
//         );
//       }
//       const result = await xFhirQuery(expression.expression, fpOptions);
//       return { context: nextContext, result };
//     }
//     case "text/fhirpath": {
//       if (!expression.expression) {
//         throw new OperationError(
//           outcomeFatal(
//             "exception",
//             `Invalid access policy expression at '${loc}'.`,
//           ),
//         );
//       }

//       const result = await fp.evaluate(
//         expression.expression,
//         undefined,
//         fpOptions,
//       );

//       if (result.length !== 1) {
//         throw new OperationError(
//           outcomeFatal(
//             "exception",
//             `Invalid access policy expression at '${loc}'.`,
//           ),
//         );
//       }

//       if (typeof result[0] !== "string" && typeof result[0] !== "boolean") {
//         throw new OperationError(
//           outcomeFatal(
//             "exception",
//             `Invalid access policy expression at '${loc}'.`,
//           ),
//         );
//       }

//       return { context: nextContext, result: result[0] };
//     }
//     default: {
//       throw new OperationError(
//         outcomeFatal(
//           "exception",
//           `Invalid access policy expression language '${expression.language}' at ${loc}.`,
//         ),
//       );
//     }
//   }
// }
