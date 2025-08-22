import type * as fhirTypes from "@oxidized-health/fhir-types/r4/types";
import { IOperation, Operation } from "@oxidized-health/operation-execution";

export namespace ActivityDefinitionApply {
  export type Input = {
    activityDefinition?: fhirTypes.ActivityDefinition;
    subject: Array<fhirTypes.string>;
    encounter?: fhirTypes.string;
    practitioner?: fhirTypes.string;
    organization?: fhirTypes.string;
    userType?: fhirTypes.CodeableConcept;
    userLanguage?: fhirTypes.CodeableConcept;
    userTaskContext?: fhirTypes.CodeableConcept;
    setting?: fhirTypes.CodeableConcept;
    settingContext?: fhirTypes.CodeableConcept;
  };
  export type Output = fhirTypes.Resource;
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "ActivityDefinition-apply",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Apply</h2><p>OPERATION: Apply</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/ActivityDefinition-apply</pre><div><p>The apply operation applies a definition in a specific context</p>\n</div><p>URL: [base]/ActivityDefinition/$apply</p><p>URL: [base]/ActivityDefinition/[id]/$apply</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>activityDefinition</td><td>0..1</td><td><a href="activitydefinition.html">ActivityDefinition</a></td><td/><td><div><p>The activity definition to apply. If the operation is invoked on an instance, this parameter is not allowed. If the operation is invoked at the type level, this parameter is required</p>\n</div></td></tr><tr><td>IN</td><td>subject</td><td>1..*</td><td><a href="datatypes.html#string">string</a><br/>(<a href="search.html#reference">reference</a>)</td><td/><td><div><p>The subject(s) that is/are the target of the activity definition to be applied. The subject may be a Patient, Practitioner, Organization, Location, Device, or Group. Subjects provided in this parameter will be resolved as the subject of the PlanDefinition based on the type of the subject. If multiple subjects of the same type are provided, the behavior is implementation-defined</p>\n</div></td></tr><tr><td>IN</td><td>encounter</td><td>0..1</td><td><a href="datatypes.html#string">string</a><br/>(<a href="search.html#reference">reference</a>)</td><td/><td><div><p>The encounter in context, if any</p>\n</div></td></tr><tr><td>IN</td><td>practitioner</td><td>0..1</td><td><a href="datatypes.html#string">string</a><br/>(<a href="search.html#reference">reference</a>)</td><td/><td><div><p>The practitioner in context</p>\n</div></td></tr><tr><td>IN</td><td>organization</td><td>0..1</td><td><a href="datatypes.html#string">string</a><br/>(<a href="search.html#reference">reference</a>)</td><td/><td><div><p>The organization in context</p>\n</div></td></tr><tr><td>IN</td><td>userType</td><td>0..1</td><td><a href="datatypes.html#CodeableConcept">CodeableConcept</a></td><td/><td><div><p>The type of user initiating the request, e.g. patient, healthcare provider, or specific type of healthcare provider (physician, nurse, etc.)</p>\n</div></td></tr><tr><td>IN</td><td>userLanguage</td><td>0..1</td><td><a href="datatypes.html#CodeableConcept">CodeableConcept</a></td><td/><td><div><p>Preferred language of the person using the system</p>\n</div></td></tr><tr><td>IN</td><td>userTaskContext</td><td>0..1</td><td><a href="datatypes.html#CodeableConcept">CodeableConcept</a></td><td/><td><div><p>The task the system user is performing, e.g. laboratory results review, medication list review, etc. This information can be used to tailor decision support outputs, such as recommended information resources</p>\n</div></td></tr><tr><td>IN</td><td>setting</td><td>0..1</td><td><a href="datatypes.html#CodeableConcept">CodeableConcept</a></td><td/><td><div><p>The current setting of the request (inpatient, outpatient, etc.)</p>\n</div></td></tr><tr><td>IN</td><td>settingContext</td><td>0..1</td><td><a href="datatypes.html#CodeableConcept">CodeableConcept</a></td><td/><td><div><p>Additional detail about the setting of the request, if any</p>\n</div></td></tr><tr><td>OUT</td><td>return</td><td>1..1</td><td>Any</td><td/><td><div><p>The resource that is the result of applying the definition</p>\n</div></td></tr></table><div><p>The result of invoking this operation is a resource of the type specified by the activity definition, with all the definitions resolved as appropriate for the type of resource. Any dynamicValue elements will be evaluated (in the order in which they appear in the resource) and the results applied to the returned resource.  If the ActivityDefinition includes library references, those libraries will be available to the evaluated expressions. If those libraries have parameters, those parameters will be bound by name to the parameters given to the operation. In addition, parameters to the $apply operation are available within dynamicValue expressions as context variables, accessible by the name of the parameter, prefixed with a percent (%) symbol. For a more detailed description, refer to the ActivityDefinition resource</p>\n</div></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 2,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/ActivityDefinition-apply",
    version: "4.0.1",
    name: "Apply",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "The apply operation applies a definition in a specific context",
    code: "apply",
    comment:
      "The result of invoking this operation is a resource of the type specified by the activity definition, with all the definitions resolved as appropriate for the type of resource. Any dynamicValue elements will be evaluated (in the order in which they appear in the resource) and the results applied to the returned resource.  If the ActivityDefinition includes library references, those libraries will be available to the evaluated expressions. If those libraries have parameters, those parameters will be bound by name to the parameters given to the operation. In addition, parameters to the $apply operation are available within dynamicValue expressions as context variables, accessible by the name of the parameter, prefixed with a percent (%) symbol. For a more detailed description, refer to the ActivityDefinition resource",
    resource: ["ActivityDefinition"],
    system: false,
    type: true,
    instance: true,
    parameter: [
      {
        name: "activityDefinition",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The activity definition to apply. If the operation is invoked on an instance, this parameter is not allowed. If the operation is invoked at the type level, this parameter is required",
        type: "ActivityDefinition",
      },
      {
        name: "subject",
        use: "in",
        min: 1,
        max: "*",
        documentation:
          "The subject(s) that is/are the target of the activity definition to be applied. The subject may be a Patient, Practitioner, Organization, Location, Device, or Group. Subjects provided in this parameter will be resolved as the subject of the PlanDefinition based on the type of the subject. If multiple subjects of the same type are provided, the behavior is implementation-defined",
        type: "string",
        searchType: "reference",
      },
      {
        name: "encounter",
        use: "in",
        min: 0,
        max: "1",
        documentation: "The encounter in context, if any",
        type: "string",
        searchType: "reference",
      },
      {
        name: "practitioner",
        use: "in",
        min: 0,
        max: "1",
        documentation: "The practitioner in context",
        type: "string",
        searchType: "reference",
      },
      {
        name: "organization",
        use: "in",
        min: 0,
        max: "1",
        documentation: "The organization in context",
        type: "string",
        searchType: "reference",
      },
      {
        name: "userType",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The type of user initiating the request, e.g. patient, healthcare provider, or specific type of healthcare provider (physician, nurse, etc.)",
        type: "CodeableConcept",
      },
      {
        name: "userLanguage",
        use: "in",
        min: 0,
        max: "1",
        documentation: "Preferred language of the person using the system",
        type: "CodeableConcept",
      },
      {
        name: "userTaskContext",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The task the system user is performing, e.g. laboratory results review, medication list review, etc. This information can be used to tailor decision support outputs, such as recommended information resources",
        type: "CodeableConcept",
      },
      {
        name: "setting",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The current setting of the request (inpatient, outpatient, etc.)",
        type: "CodeableConcept",
      },
      {
        name: "settingContext",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "Additional detail about the setting of the request, if any",
        type: "CodeableConcept",
      },
      {
        name: "return",
        use: "out",
        min: 1,
        max: "1",
        documentation:
          "The resource that is the result of applying the definition",
        type: "Any",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace ActivityDefinitionDataRequirements {
  export type Input = Record<string, never>;
  export type Output = fhirTypes.Library;
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "ActivityDefinition-data-requirements",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Data Requirements</h2><p>OPERATION: Data Requirements</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/ActivityDefinition-data-requirements</pre><div><p>The data-requirements operation aggregates and returns the parameters and data requirements for the activity definition and all its dependencies as a single module definition library</p>\n</div><p>URL: [base]/ActivityDefinition/[id]/$data-requirements</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>OUT</td><td>return</td><td>1..1</td><td><a href="library.html">Library</a></td><td/><td><div><p>The result of the requirements gathering represented as a module-definition Library that describes the aggregate parameters, data requirements, and dependencies of the activity definition</p>\n</div></td></tr></table><div><p>The effect of invoking this operation is to determine the aggregate set of data requirements and dependencies for the activity definition. The result is a Library resource with a type of module-definition that contains all the parameter definitions and data requirements of the activity definition and any libraries referenced by it. Implementations SHOULD aggregate data requirements intelligently (i.e. by collapsing overlapping data requirements)</p>\n</div></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 2,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/ActivityDefinition-data-requirements",
    version: "4.0.1",
    name: "Data Requirements",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "The data-requirements operation aggregates and returns the parameters and data requirements for the activity definition and all its dependencies as a single module definition library",
    code: "data-requirements",
    comment:
      "The effect of invoking this operation is to determine the aggregate set of data requirements and dependencies for the activity definition. The result is a Library resource with a type of module-definition that contains all the parameter definitions and data requirements of the activity definition and any libraries referenced by it. Implementations SHOULD aggregate data requirements intelligently (i.e. by collapsing overlapping data requirements)",
    resource: ["ActivityDefinition"],
    system: false,
    type: false,
    instance: true,
    parameter: [
      {
        name: "return",
        use: "out",
        min: 1,
        max: "1",
        documentation:
          "The result of the requirements gathering represented as a module-definition Library that describes the aggregate parameters, data requirements, and dependencies of the activity definition",
        type: "Library",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace CapabilityStatementConforms {
  export type Input = {
    left?: fhirTypes.canonical;
    right?: fhirTypes.canonical;
    mode?: fhirTypes.code;
  };
  export type Output = {
    issues: fhirTypes.OperationOutcome;
    union?: fhirTypes.CapabilityStatement;
    intersection?: fhirTypes.CapabilityStatement;
  };
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "CapabilityStatement-conforms",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Test if a server implements a client\'s required operations</h2><p>OPERATION: Test if a server implements a client\'s required operations</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/CapabilityStatement-conforms</pre><div><p>This operation asks the server to check that it implements all the resources, interactions, search parameters, and operations that the client provides in its capability statement. The client provides both capability statements by reference, and must ensure that all the referenced resources are available to the conformance server</p>\n</div><p>URL: [base]/CapabilityStatement/$conforms</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>left</td><td>0..1</td><td><a href="datatypes.html#canonical">canonical</a></td><td/><td><div><p>A canonical reference to the left-hand system\'s capability statement</p>\n</div></td></tr><tr><td>IN</td><td>right</td><td>0..1</td><td><a href="datatypes.html#canonical">canonical</a></td><td/><td><div><p>A canonical reference to the right-hand system\'s capability statement</p>\n</div></td></tr><tr><td>IN</td><td>mode</td><td>0..1</td><td><a href="datatypes.html#code">code</a></td><td/><td><div><p>What kind of comparison to perform - server to server, or client to server (use the codes \'server/server\' or \'client/server\')</p>\n</div></td></tr><tr><td>OUT</td><td>issues</td><td>1..1</td><td><a href="operationoutcome.html">OperationOutcome</a></td><td/><td><div><p>Outcome of the CapabilityStatement test</p>\n</div></td></tr><tr><td>OUT</td><td>union</td><td>0..1</td><td><a href="capabilitystatement.html">CapabilityStatement</a></td><td/><td><div><p>The intersection of the functionality described by the CapabilityStatement resources</p>\n</div></td></tr><tr><td>OUT</td><td>intersection</td><td>0..1</td><td><a href="capabilitystatement.html">CapabilityStatement</a></td><td/><td><div><p>The union of the functionality described by the CapabilityStatement resources</p>\n</div></td></tr></table><div><p>The operation performs a full comparison of the functionality described by the two capability statements, including the profiles and value sets they reference, and also including concept maps and structure maps.</p>\n<p>The full execution of this operation is still a matter of research, but it is intended to support comparison of systems to see if they will interoperate</p>\n<p>If the capability statements can be successfully compared, then the return value is a 200 OK with an OperationOutcome along with intersection and union capability statements. The operation outcome can contain errors relating to differences between the capability statements. If the capability statements cannot be compared, because dependencies cannot be located, the return value is a 4xx error, with an OperationOutcome with at least one issue with severity &gt;= error</p>\n</div></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 5,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/CapabilityStatement-conforms",
    version: "4.0.1",
    name: "Test if a server implements a client's required operations",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "This operation asks the server to check that it implements all the resources, interactions, search parameters, and operations that the client provides in its capability statement. The client provides both capability statements by reference, and must ensure that all the referenced resources are available to the conformance server",
    code: "conforms",
    comment:
      "The operation performs a full comparison of the functionality described by the two capability statements, including the profiles and value sets they reference, and also including concept maps and structure maps.     \n\nThe full execution of this operation is still a matter of research, but it is intended to support comparison of systems to see if they will interoperate    \n\nIf the capability statements can be successfully compared, then the return value is a 200 OK with an OperationOutcome along with intersection and union capability statements. The operation outcome can contain errors relating to differences between the capability statements. If the capability statements cannot be compared, because dependencies cannot be located, the return value is a 4xx error, with an OperationOutcome with at least one issue with severity >= error",
    resource: ["CapabilityStatement"],
    system: false,
    type: true,
    instance: false,
    parameter: [
      {
        name: "left",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "A canonical reference to the left-hand system's capability statement",
        type: "canonical",
      },
      {
        name: "right",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "A canonical reference to the right-hand system's capability statement",
        type: "canonical",
      },
      {
        name: "mode",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "What kind of comparison to perform - server to server, or client to server (use the codes 'server/server' or 'client/server')",
        type: "code",
      },
      {
        name: "issues",
        use: "out",
        min: 1,
        max: "1",
        documentation: "Outcome of the CapabilityStatement test",
        type: "OperationOutcome",
      },
      {
        name: "union",
        use: "out",
        min: 0,
        max: "1",
        documentation:
          "The intersection of the functionality described by the CapabilityStatement resources",
        type: "CapabilityStatement",
      },
      {
        name: "intersection",
        use: "out",
        min: 0,
        max: "1",
        documentation:
          "The union of the functionality described by the CapabilityStatement resources",
        type: "CapabilityStatement",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace CapabilityStatementImplements {
  export type Input = {
    server?: fhirTypes.canonical;
    client?: fhirTypes.canonical;
    resource?: fhirTypes.CapabilityStatement;
  };
  export type Output = fhirTypes.OperationOutcome;
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "CapabilityStatement-implements",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Test if a server implements a client\'s required operations</h2><p>OPERATION: Test if a server implements a client\'s required operations</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/CapabilityStatement-implements</pre><div><p>This operation asks the server to check that it implements all the resources, interactions, search parameters, and operations that the client provides in its capability statement. The client provides its capability statement inline, or by referring the server to the canonical URL of its capability statement</p>\n</div><p>URL: [base]/CapabilityStatement/$implements</p><p>URL: [base]/CapabilityStatement/[id]/$implements</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>server</td><td>0..1</td><td><a href="datatypes.html#canonical">canonical</a></td><td/><td><div><p>A canonical reference to the server capability statement - use this if the implements is not invoked on an instance (or on the /metadata end-point)</p>\n</div></td></tr><tr><td>IN</td><td>client</td><td>0..1</td><td><a href="datatypes.html#canonical">canonical</a></td><td/><td><div><p>A canonical reference to the client capability statement - use this if the implements is not invoked on an instance (or on the /metadata end-point)</p>\n</div></td></tr><tr><td>IN</td><td>resource</td><td>0..1</td><td><a href="capabilitystatement.html">CapabilityStatement</a></td><td/><td><div><p>The client capability statement, provided inline</p>\n</div></td></tr><tr><td>OUT</td><td>return</td><td>1..1</td><td><a href="operationoutcome.html">OperationOutcome</a></td><td/><td><div><p>Outcome of the CapabilityStatement test</p>\n</div></td></tr></table><div><p>The operation does not perform a full conformance check; in particular it does not check that the profiles align. It merely checks that the behaviors the client wishes to use are provided    Technically, this operation is implemented as follows:</p>\n<ul>\n<li>The server\'s capability statement must have an entry for each resource in the client\'s capability statement</li>\n<li>The server\'s resource support must have matching flags for updateCreate, conditionalCreate, conditionalRead, conditionalUpdate, conditionalDelete, searchInclude, searchRevInclude</li>\n<li>The server\'s capability statement must have a matching interaction for each interaction in the client capability statement (whether or not it is on a resource)</li>\n<li>The server\'s capability statement must have a search parameter with matching name and definition for any search parameters in the client capability statement</li>\n<li>The server must have an operation definition with a matching reference for any operations in the client capability statement</li>\n</ul>\n<p>If the capability statements match by these rules, then the return value is a 200 OK with an operation outcome that contains no issues with severity &gt;= error. If the capability statement doesn\'t match, the return value is a 4xx error, with an OperationOutcome with at least one issue with severity &gt;= error</p>\n</div></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 5,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/CapabilityStatement-implements",
    version: "4.0.1",
    name: "Test if a server implements a client's required operations",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "This operation asks the server to check that it implements all the resources, interactions, search parameters, and operations that the client provides in its capability statement. The client provides its capability statement inline, or by referring the server to the canonical URL of its capability statement",
    code: "implements",
    comment:
      "The operation does not perform a full conformance check; in particular it does not check that the profiles align. It merely checks that the behaviors the client wishes to use are provided    Technically, this operation is implemented as follows:   \n\n* The server's capability statement must have an entry for each resource in the client's capability statement    \n* The server's resource support must have matching flags for updateCreate, conditionalCreate, conditionalRead, conditionalUpdate, conditionalDelete, searchInclude, searchRevInclude   \n* The server's capability statement must have a matching interaction for each interaction in the client capability statement (whether or not it is on a resource)   \n* The server's capability statement must have a search parameter with matching name and definition for any search parameters in the client capability statement   \n* The server must have an operation definition with a matching reference for any operations in the client capability statement   \n\nIf the capability statements match by these rules, then the return value is a 200 OK with an operation outcome that contains no issues with severity >= error. If the capability statement doesn't match, the return value is a 4xx error, with an OperationOutcome with at least one issue with severity >= error",
    resource: ["CapabilityStatement"],
    system: false,
    type: true,
    instance: true,
    parameter: [
      {
        name: "server",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "A canonical reference to the server capability statement - use this if the implements is not invoked on an instance (or on the /metadata end-point)",
        type: "canonical",
      },
      {
        name: "client",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "A canonical reference to the client capability statement - use this if the implements is not invoked on an instance (or on the /metadata end-point)",
        type: "canonical",
      },
      {
        name: "resource",
        use: "in",
        min: 0,
        max: "1",
        documentation: "The client capability statement, provided inline",
        type: "CapabilityStatement",
      },
      {
        name: "return",
        use: "out",
        min: 1,
        max: "1",
        documentation: "Outcome of the CapabilityStatement test",
        type: "OperationOutcome",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace CapabilityStatementSubset {
  export type Input = {
    server?: fhirTypes.uri;
    resource: Array<fhirTypes.code>;
  };
  export type Output = fhirTypes.CapabilityStatement;
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "CapabilityStatement-subset",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Fetch a subset of the CapabilityStatement resource</h2><p>OPERATION: Fetch a subset of the CapabilityStatement resource</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/CapabilityStatement-subset</pre><div><p>This operation asks the server to return a subset of the CapabilityStatement resource - just the REST parts that relate to a set of nominated resources - the resources that the client is interested in</p>\n</div><p>URL: [base]/CapabilityStatement/$subset</p><p>URL: [base]/CapabilityStatement/[id]/$subset</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>server</td><td>0..1</td><td><a href="datatypes.html#uri">uri</a></td><td/><td><div><p>The canonical URL - use this if the subset is not invoked on an instance (or on the /metadata end-point)</p>\n</div></td></tr><tr><td>IN</td><td>resource</td><td>1..*</td><td><a href="datatypes.html#code">code</a></td><td/><td><div><p>A resource that the client would like to include in the return</p>\n</div></td></tr><tr><td>OUT</td><td>return</td><td>1..1</td><td><a href="capabilitystatement.html">CapabilityStatement</a></td><td/><td><div><p>The subsetted CapabilityStatement resource that is returned. This should be tagged with the SUBSETTED code</p>\n</div></td></tr></table><div/></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 5,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/CapabilityStatement-subset",
    version: "4.0.1",
    name: "Fetch a subset of the CapabilityStatement resource",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "This operation asks the server to return a subset of the CapabilityStatement resource - just the REST parts that relate to a set of nominated resources - the resources that the client is interested in",
    code: "subset",
    resource: ["CapabilityStatement"],
    system: false,
    type: true,
    instance: true,
    parameter: [
      {
        name: "server",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The canonical URL - use this if the subset is not invoked on an instance (or on the /metadata end-point)",
        type: "uri",
      },
      {
        name: "resource",
        use: "in",
        min: 1,
        max: "*",
        documentation:
          "A resource that the client would like to include in the return",
        type: "code",
      },
      {
        name: "return",
        use: "out",
        min: 1,
        max: "1",
        documentation:
          "The subsetted CapabilityStatement resource that is returned. This should be tagged with the SUBSETTED code",
        type: "CapabilityStatement",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace CapabilityStatementVersions {
  export type Input = Record<string, never>;
  export type Output = {
    version: Array<fhirTypes.code>;
    default: fhirTypes.code;
  };
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "CapabilityStatement-versions",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Discover what versions a server supports</h2><p>OPERATION: Discover what versions a server supports</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/CapabilityStatement-versions</pre><div><p>Using the <a href="http.html#version-parameter">FHIR Version Mime Type Parameter</a>, a server can support <a href="versioning.html#mt-version">multiple versions on the same end-point</a>. The only way for client to find out what versions a server supports in this fashion is the $versions operation. The client invokes the operation with no parameters. and the server returns the list of supported versions, along with the default version it will use if no fhirVersion parameter is present</p>\n</div><p>URL: [base]/$versions</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>OUT</td><td>version</td><td>1..*</td><td><a href="datatypes.html#code">code</a></td><td/><td><div><p>A version supported by the server. Use the major.minor version like 3.0</p>\n</div></td></tr><tr><td>OUT</td><td>default</td><td>1..1</td><td><a href="datatypes.html#code">code</a></td><td/><td><div><p>The default version for the server. Use the major.minor version like 3.0</p>\n</div></td></tr></table><div/></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 5,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/CapabilityStatement-versions",
    version: "4.0.1",
    name: "Discover what versions a server supports",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "Using the [FHIR Version Mime Type Parameter](http.html#version-parameter), a server can support [multiple versions on the same end-point](versioning.html#mt-version). The only way for client to find out what versions a server supports in this fashion is the $versions operation. The client invokes the operation with no parameters. and the server returns the list of supported versions, along with the default version it will use if no fhirVersion parameter is present",
    code: "versions",
    resource: ["CapabilityStatement"],
    system: true,
    type: false,
    instance: false,
    parameter: [
      {
        name: "version",
        use: "out",
        min: 1,
        max: "*",
        documentation:
          "A version supported by the server. Use the major.minor version like 3.0",
        type: "code",
      },
      {
        name: "default",
        use: "out",
        min: 1,
        max: "1",
        documentation:
          "The default version for the server. Use the major.minor version like 3.0",
        type: "code",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace ChargeItemDefinitionApply {
  export type Input = {
    chargeItem: fhirTypes.Reference;
    account?: fhirTypes.Reference;
  };
  export type Output = fhirTypes.Resource;
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "ChargeItemDefinition-apply",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Apply</h2><p>OPERATION: Apply</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/ChargeItemDefinition-apply</pre><div><p>The apply operation applies a definition in a specific context</p>\n</div><p>URL: [base]/ChargeItemDefinition/[id]/$apply</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>chargeItem</td><td>1..1</td><td><a href="references.html#Reference">Reference</a></td><td/><td><div><p>The ChargeItem on which the definition is to ba applies</p>\n</div></td></tr><tr><td>IN</td><td>account</td><td>0..1</td><td><a href="references.html#Reference">Reference</a></td><td/><td><div><p>The account in context, if any</p>\n</div></td></tr><tr><td>OUT</td><td>return</td><td>1..1</td><td>Any</td><td/><td><div><p>The resource that is the result of applying the definition</p>\n</div></td></tr></table><div><p>The result of invoking this operation is a resource of the type specified by the activity definition, with all the definitions resolved as appropriate for the type of resource. Any dynamicValue elements will be evaluated (in the order in which they appear in the resource) and the results applied to the returned resource.  If the ActivityDefinition includes library references, those libraries will be available to the evaluated expressions. If those libraries have parameters, those parameters will be bound by name to the parameters given to the operation. For a more detailed description, refer to the ActivityDefinition resource</p>\n</div></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 0,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/ChargeItemDefinition-apply",
    version: "4.0.1",
    name: "Apply",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "The apply operation applies a definition in a specific context",
    code: "apply",
    comment:
      "The result of invoking this operation is a resource of the type specified by the activity definition, with all the definitions resolved as appropriate for the type of resource. Any dynamicValue elements will be evaluated (in the order in which they appear in the resource) and the results applied to the returned resource.  If the ActivityDefinition includes library references, those libraries will be available to the evaluated expressions. If those libraries have parameters, those parameters will be bound by name to the parameters given to the operation. For a more detailed description, refer to the ActivityDefinition resource",
    resource: ["ChargeItemDefinition"],
    system: false,
    type: false,
    instance: true,
    parameter: [
      {
        name: "chargeItem",
        use: "in",
        min: 1,
        max: "1",
        documentation:
          "The ChargeItem on which the definition is to ba applies",
        type: "Reference",
        targetProfile: ["http://hl7.org/fhir/StructureDefinition/ChargeItem"],
      },
      {
        name: "account",
        use: "in",
        min: 0,
        max: "1",
        documentation: "The account in context, if any",
        type: "Reference",
        targetProfile: ["http://hl7.org/fhir/StructureDefinition/Account"],
      },
      {
        name: "return",
        use: "out",
        min: 1,
        max: "1",
        documentation:
          "The resource that is the result of applying the definition",
        type: "Any",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace ClaimSubmit {
  export type Input = { resource: fhirTypes.Resource };
  export type Output = fhirTypes.Resource;
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "Claim-submit",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Submit a Claim resource for adjudication</h2><p>OPERATION: Submit a Claim resource for adjudication</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/Claim-submit</pre><div><p>This operation is used to submit a Claim, Pre-Authorization or Pre-Determination (all instances of Claim resources) for adjudication either as a single Claim resource instance or as a Bundle containing the Claim and other referenced resources, or Bundle containing a batch of Claim resources, either as single Claims resources or Bundle resources, for processing. The only input parameter is the single Claim or Bundle resource and the only output is a single ClaimResponse, Bundle of ClaimResponses or an OperationOutcome resource.</p>\n</div><p>URL: [base]/Claim/$submit</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>resource</td><td>1..1</td><td><a href="resource.html">Resource</a></td><td/><td><div><p>A Claim resource or Bundle of claims, either as individual Claim resources or as Bundles each containing a single Claim plus referenced resources.</p>\n</div></td></tr><tr><td>OUT</td><td>return</td><td>1..1</td><td><a href="resource.html">Resource</a></td><td/><td><div><p>A ClaimResponse resource or Bundle of claim responses, either as individual ClaimResponse resources or as Bundles each containing a single ClaimResponse plus referenced resources.</p>\n</div></td></tr></table><div/></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 2,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/Claim-submit",
    version: "4.0.1",
    name: "Submit a Claim resource for adjudication",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "This operation is used to submit a Claim, Pre-Authorization or Pre-Determination (all instances of Claim resources) for adjudication either as a single Claim resource instance or as a Bundle containing the Claim and other referenced resources, or Bundle containing a batch of Claim resources, either as single Claims resources or Bundle resources, for processing. The only input parameter is the single Claim or Bundle resource and the only output is a single ClaimResponse, Bundle of ClaimResponses or an OperationOutcome resource.",
    code: "submit",
    resource: ["Claim"],
    system: false,
    type: true,
    instance: false,
    parameter: [
      {
        name: "resource",
        use: "in",
        min: 1,
        max: "1",
        documentation:
          "A Claim resource or Bundle of claims, either as individual Claim resources or as Bundles each containing a single Claim plus referenced resources.",
        type: "Resource",
      },
      {
        name: "return",
        use: "out",
        min: 1,
        max: "1",
        documentation:
          "A ClaimResponse resource or Bundle of claim responses, either as individual ClaimResponse resources or as Bundles each containing a single ClaimResponse plus referenced resources.",
        type: "Resource",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace CodeSystemFindMatches {
  export type Input = {
    system?: fhirTypes.uri;
    version?: fhirTypes.string;
    property?: Array<{
      code: fhirTypes.code;
      value?: fhirTypes.Element;
      subproperty?: Array<{ code: fhirTypes.code; value: fhirTypes.Element }>;
    }>;
    exact: fhirTypes.boolean;
    compositional?: fhirTypes.boolean;
  };
  export type Output = {
    match?: Array<{
      code: fhirTypes.Coding;
      unmatched?: Array<{
        code: fhirTypes.code;
        value: fhirTypes.Element;
        property?: Array<{ code: fhirTypes.code; value: fhirTypes.Element }>;
      }>;
      comment?: fhirTypes.string;
    }>;
  };
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "CodeSystem-find-matches",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Finding codes based on supplied properties</h2><p>OPERATION: Finding codes based on supplied properties</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/CodeSystem-find-matches</pre><div><p>Given a set of properties (and text), return one or more possible matching codes</p>\n<p>This operation takes a set of properties, and examines the code system looking for codes in the code system that match a set of known properties.</p>\n<p>When looking for matches, there are 3 possible types of match:</p>\n<ul>\n<li>a complete match - a code that represents all the provided properties correctly</li>\n<li>a partial match - a code that represents some of the provided properties correctly, and not others</li>\n<li>a possible match - a code that may represent the provided properties closely, but may capture less or more precise information for some of the properties</li>\n</ul>\n<p>The $find-matches operation can be called in one of 2 modes:</p>\n<ul>\n<li>By a human, looking for the best match for a set of properties. In this mode, the server returns a list of complete, possible or partial matches (possibly with comments), so that the user can choose (or not) the most appropriate code</li>\n<li>By a machine (typically in a system interface performing a transformation). In this mode, the server returns only a list of complete and partial matches, but no possible matches. The machine can choose a code from the list (or not) based on what properties are not coded</li>\n</ul>\n<p>These modes are differentiated by the \'exact\' parameter, so the client can indicate whether it only wants exact matches (including partial matches) or whether potential matches based on text matching are desired</p>\n<p>The find-matches operation is still preliminary. The interface can be expected to change as more experience is gained from implementations.</p>\n</div><p>URL: [base]/CodeSystem/$find-matches</p><p>URL: [base]/CodeSystem/[id]/$find-matches</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>system</td><td>0..1</td><td><a href="datatypes.html#uri">uri</a></td><td/><td><div><p>The system in which composition is to be performed. This must be provided unless the operation is invoked on a code system instance</p>\n</div></td></tr><tr><td>IN</td><td>version</td><td>0..1</td><td><a href="datatypes.html#string">string</a></td><td/><td><div><p>The version of the system for the inferencing to be performed</p>\n</div></td></tr><tr><td>IN</td><td>property</td><td>0..*</td><td></td><td/><td><div><p>One or more properties that contain information to be composed into the code</p>\n</div></td></tr><tr><td>IN</td><td>property.code</td><td>1..1</td><td><a href="datatypes.html#code">code</a></td><td/><td><div><p>Identifies the property provided</p>\n</div></td></tr><tr><td>IN</td><td>property.value</td><td>0..1</td><td><a href="datatypes.html#code">code</a> | <a href="datatypes.html#Coding">Coding</a> | <a href="datatypes.html#string">string</a> | <a href="datatypes.html#integer">integer</a> | <a href="datatypes.html#boolean">boolean</a> | <a href="datatypes.html#dateTime">dateTime</a></td><td/><td><div><p>The value of the property provided</p>\n</div></td></tr><tr><td>IN</td><td>property.subproperty</td><td>0..*</td><td></td><td/><td><div><p>Nested Properties (mainly used for SNOMED CT composition, for relationship Groups)</p>\n</div></td></tr><tr><td>IN</td><td>property.subproperty.code</td><td>1..1</td><td><a href="datatypes.html#code">code</a></td><td/><td><div><p>Identifies the sub-property provided</p>\n</div></td></tr><tr><td>IN</td><td>property.subproperty.value</td><td>1..1</td><td><a href="datatypes.html#code">code</a> | <a href="datatypes.html#Coding">Coding</a> | <a href="datatypes.html#string">string</a> | <a href="datatypes.html#integer">integer</a> | <a href="datatypes.html#boolean">boolean</a> | <a href="datatypes.html#dateTime">dateTime</a></td><td/><td><div><p>The value of the sub-property provided</p>\n</div></td></tr><tr><td>IN</td><td>exact</td><td>1..1</td><td><a href="datatypes.html#boolean">boolean</a></td><td/><td><div><p>Whether the operation is being used by a human (\'false\'), or a machine (\'true\'). If the operation is being used by a human, the terminology server can return a list of possible matches, with commentary. For a machine, the server returns complete or partial matches, not possible matches. The default value is \'false\'</p>\n</div></td></tr><tr><td>IN</td><td>compositional</td><td>0..1</td><td><a href="datatypes.html#boolean">boolean</a></td><td/><td><div><p>Post-coordinated expressions are allowed to be returned in the matching codes (mainly for SNOMED CT). Default = false</p>\n</div></td></tr><tr><td>OUT</td><td>match</td><td>0..*</td><td></td><td/><td><div><p>Concepts returned by the server as a result of the inferencing operation</p>\n</div></td></tr><tr><td>OUT</td><td>match.code</td><td>1..1</td><td><a href="datatypes.html#Coding">Coding</a></td><td/><td><div><p>A code that matches the properties provided</p>\n</div></td></tr><tr><td>OUT</td><td>match.unmatched</td><td>0..*</td><td></td><td/><td><div><p>One or more properties that contain properties that could not be matched into the code</p>\n</div></td></tr><tr><td>OUT</td><td>match.unmatched.code</td><td>1..1</td><td><a href="datatypes.html#code">code</a></td><td/><td><div><p>Identifies the property provided</p>\n</div></td></tr><tr><td>OUT</td><td>match.unmatched.value</td><td>1..1</td><td><a href="datatypes.html#code">code</a> | <a href="datatypes.html#Coding">Coding</a> | <a href="datatypes.html#string">string</a> | <a href="datatypes.html#integer">integer</a> | <a href="datatypes.html#boolean">boolean</a> | <a href="datatypes.html#dateTime">dateTime</a></td><td/><td><div><p>The value of the property provided</p>\n</div></td></tr><tr><td>OUT</td><td>match.unmatched.property</td><td>0..*</td><td></td><td/><td><div><p>Nested Properties (mainly used for SNOMED CT composition, for relationship Groups)</p>\n</div></td></tr><tr><td>OUT</td><td>match.unmatched.property.code</td><td>1..1</td><td><a href="datatypes.html#code">code</a></td><td/><td><div><p>Identifies the sub-property provided</p>\n</div></td></tr><tr><td>OUT</td><td>match.unmatched.property.value</td><td>1..1</td><td><a href="datatypes.html#code">code</a> | <a href="datatypes.html#Coding">Coding</a> | <a href="datatypes.html#string">string</a> | <a href="datatypes.html#integer">integer</a> | <a href="datatypes.html#boolean">boolean</a> | <a href="datatypes.html#dateTime">dateTime</a></td><td/><td><div><p>The value of the sub-property provided</p>\n</div></td></tr><tr><td>OUT</td><td>match.comment</td><td>0..1</td><td><a href="datatypes.html#string">string</a></td><td/><td><div><p>Information about the quality of the match, if operation is for a human</p>\n</div></td></tr></table><div/></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 0,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/CodeSystem-find-matches",
    version: "4.0.1",
    name: "Finding codes based on supplied properties",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "Given a set of properties (and text), return one or more possible matching codes\n\nThis operation takes a set of properties, and examines the code system looking for codes in the code system that match a set of known properties. \n\nWhen looking for matches, there are 3 possible types of match:\n* a complete match - a code that represents all the provided properties correctly\n* a partial match - a code that represents some of the provided properties correctly, and not others \n* a possible match - a code that may represent the provided properties closely, but may capture less or more precise information for some of the properties\n\nThe $find-matches operation can be called in one of 2 modes:\n* By a human, looking for the best match for a set of properties. In this mode, the server returns a list of complete, possible or partial matches (possibly with comments), so that the user can choose (or not) the most appropriate code\n* By a machine (typically in a system interface performing a transformation). In this mode, the server returns only a list of complete and partial matches, but no possible matches. The machine can choose a code from the list (or not) based on what properties are not coded\n\nThese modes are differentiated by the 'exact' parameter, so the client can indicate whether it only wants exact matches (including partial matches) or whether potential matches based on text matching are desired\n \nThe find-matches operation is still preliminary. The interface can be expected to change as more experience is gained from implementations.",
    code: "find-matches",
    resource: ["CodeSystem"],
    system: false,
    type: true,
    instance: true,
    parameter: [
      {
        name: "system",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The system in which composition is to be performed. This must be provided unless the operation is invoked on a code system instance",
        type: "uri",
      },
      {
        name: "version",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The version of the system for the inferencing to be performed",
        type: "string",
      },
      {
        name: "property",
        use: "in",
        min: 0,
        max: "*",
        documentation:
          "One or more properties that contain information to be composed into the code",
        part: [
          {
            name: "code",
            use: "in",
            min: 1,
            max: "1",
            documentation: "Identifies the property provided",
            type: "code",
          },
          {
            extension: [
              {
                url: "http://hl7.org/fhir/StructureDefinition/operationdefinition-allowed-type",
                valueUri: "code",
              },
              {
                url: "http://hl7.org/fhir/StructureDefinition/operationdefinition-allowed-type",
                valueUri: "Coding",
              },
              {
                url: "http://hl7.org/fhir/StructureDefinition/operationdefinition-allowed-type",
                valueUri: "string",
              },
              {
                url: "http://hl7.org/fhir/StructureDefinition/operationdefinition-allowed-type",
                valueUri: "integer",
              },
              {
                url: "http://hl7.org/fhir/StructureDefinition/operationdefinition-allowed-type",
                valueUri: "boolean",
              },
              {
                url: "http://hl7.org/fhir/StructureDefinition/operationdefinition-allowed-type",
                valueUri: "dateTime",
              },
            ],
            name: "value",
            use: "in",
            min: 0,
            max: "1",
            documentation: "The value of the property provided",
            type: "Element",
          },
          {
            name: "subproperty",
            use: "in",
            min: 0,
            max: "*",
            documentation:
              "Nested Properties (mainly used for SNOMED CT composition, for relationship Groups)",
            part: [
              {
                name: "code",
                use: "in",
                min: 1,
                max: "1",
                documentation: "Identifies the sub-property provided",
                type: "code",
              },
              {
                extension: [
                  {
                    url: "http://hl7.org/fhir/StructureDefinition/operationdefinition-allowed-type",
                    valueUri: "code",
                  },
                  {
                    url: "http://hl7.org/fhir/StructureDefinition/operationdefinition-allowed-type",
                    valueUri: "Coding",
                  },
                  {
                    url: "http://hl7.org/fhir/StructureDefinition/operationdefinition-allowed-type",
                    valueUri: "string",
                  },
                  {
                    url: "http://hl7.org/fhir/StructureDefinition/operationdefinition-allowed-type",
                    valueUri: "integer",
                  },
                  {
                    url: "http://hl7.org/fhir/StructureDefinition/operationdefinition-allowed-type",
                    valueUri: "boolean",
                  },
                  {
                    url: "http://hl7.org/fhir/StructureDefinition/operationdefinition-allowed-type",
                    valueUri: "dateTime",
                  },
                ],
                name: "value",
                use: "in",
                min: 1,
                max: "1",
                documentation: "The value of the sub-property provided",
                type: "Element",
              },
            ],
          },
        ],
      },
      {
        name: "exact",
        use: "in",
        min: 1,
        max: "1",
        documentation:
          "Whether the operation is being used by a human ('false'), or a machine ('true'). If the operation is being used by a human, the terminology server can return a list of possible matches, with commentary. For a machine, the server returns complete or partial matches, not possible matches. The default value is 'false'",
        type: "boolean",
      },
      {
        name: "compositional",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "Post-coordinated expressions are allowed to be returned in the matching codes (mainly for SNOMED CT). Default = false",
        type: "boolean",
      },
      {
        name: "match",
        use: "out",
        min: 0,
        max: "*",
        documentation:
          "Concepts returned by the server as a result of the inferencing operation",
        part: [
          {
            name: "code",
            use: "out",
            min: 1,
            max: "1",
            documentation: "A code that matches the properties provided",
            type: "Coding",
          },
          {
            name: "unmatched",
            use: "out",
            min: 0,
            max: "*",
            documentation:
              "One or more properties that contain properties that could not be matched into the code",
            part: [
              {
                name: "code",
                use: "out",
                min: 1,
                max: "1",
                documentation: "Identifies the property provided",
                type: "code",
              },
              {
                extension: [
                  {
                    url: "http://hl7.org/fhir/StructureDefinition/operationdefinition-allowed-type",
                    valueUri: "code",
                  },
                  {
                    url: "http://hl7.org/fhir/StructureDefinition/operationdefinition-allowed-type",
                    valueUri: "Coding",
                  },
                  {
                    url: "http://hl7.org/fhir/StructureDefinition/operationdefinition-allowed-type",
                    valueUri: "string",
                  },
                  {
                    url: "http://hl7.org/fhir/StructureDefinition/operationdefinition-allowed-type",
                    valueUri: "integer",
                  },
                  {
                    url: "http://hl7.org/fhir/StructureDefinition/operationdefinition-allowed-type",
                    valueUri: "boolean",
                  },
                  {
                    url: "http://hl7.org/fhir/StructureDefinition/operationdefinition-allowed-type",
                    valueUri: "dateTime",
                  },
                ],
                name: "value",
                use: "out",
                min: 1,
                max: "1",
                documentation: "The value of the property provided",
                type: "Element",
              },
              {
                name: "property",
                use: "out",
                min: 0,
                max: "*",
                documentation:
                  "Nested Properties (mainly used for SNOMED CT composition, for relationship Groups)",
                part: [
                  {
                    name: "code",
                    use: "out",
                    min: 1,
                    max: "1",
                    documentation: "Identifies the sub-property provided",
                    type: "code",
                  },
                  {
                    extension: [
                      {
                        url: "http://hl7.org/fhir/StructureDefinition/operationdefinition-allowed-type",
                        valueUri: "code",
                      },
                      {
                        url: "http://hl7.org/fhir/StructureDefinition/operationdefinition-allowed-type",
                        valueUri: "Coding",
                      },
                      {
                        url: "http://hl7.org/fhir/StructureDefinition/operationdefinition-allowed-type",
                        valueUri: "string",
                      },
                      {
                        url: "http://hl7.org/fhir/StructureDefinition/operationdefinition-allowed-type",
                        valueUri: "integer",
                      },
                      {
                        url: "http://hl7.org/fhir/StructureDefinition/operationdefinition-allowed-type",
                        valueUri: "boolean",
                      },
                      {
                        url: "http://hl7.org/fhir/StructureDefinition/operationdefinition-allowed-type",
                        valueUri: "dateTime",
                      },
                    ],
                    name: "value",
                    use: "out",
                    min: 1,
                    max: "1",
                    documentation: "The value of the sub-property provided",
                    type: "Element",
                  },
                ],
              },
            ],
          },
          {
            name: "comment",
            use: "out",
            min: 0,
            max: "1",
            documentation:
              "Information about the quality of the match, if operation is for a human",
            type: "string",
          },
        ],
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace CodeSystemLookup {
  export type Input = {
    code?: fhirTypes.code;
    system?: fhirTypes.uri;
    version?: fhirTypes.string;
    coding?: fhirTypes.Coding;
    date?: fhirTypes.dateTime;
    displayLanguage?: fhirTypes.code;
    property?: Array<fhirTypes.code>;
  };
  export type Output = {
    name: fhirTypes.string;
    version?: fhirTypes.string;
    display: fhirTypes.string;
    designation?: Array<{
      language?: fhirTypes.code;
      use?: fhirTypes.Coding;
      value: fhirTypes.string;
    }>;
    property?: Array<{
      code: fhirTypes.code;
      value?: fhirTypes.Element;
      description?: fhirTypes.string;
      subproperty?: Array<{
        code: fhirTypes.code;
        value: fhirTypes.Element;
        description?: fhirTypes.string;
      }>;
    }>;
  };
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "CodeSystem-lookup",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Concept Look Up &amp; Decomposition</h2><p>OPERATION: Concept Look Up &amp; Decomposition</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/CodeSystem-lookup</pre><div><p>Given a code/system, or a Coding, get additional details about the concept, including definition, status, designations, and properties. One of the products of this operation is a full decomposition of a code from a structured terminology.</p>\n<p>When invoking this operation, a client SHALL provide both a system and a code, either using the system+code parameters, or in the coding parameter. Other parameters are optional</p>\n</div><p>URL: [base]/CodeSystem/$lookup</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>code</td><td>0..1</td><td><a href="datatypes.html#code">code</a></td><td/><td><div><p>The code that is to be located. If a code is provided, a system must be provided</p>\n</div></td></tr><tr><td>IN</td><td>system</td><td>0..1</td><td><a href="datatypes.html#uri">uri</a></td><td/><td><div><p>The system for the code that is to be located</p>\n</div></td></tr><tr><td>IN</td><td>version</td><td>0..1</td><td><a href="datatypes.html#string">string</a></td><td/><td><div><p>The version of the system, if one was provided in the source data</p>\n</div></td></tr><tr><td>IN</td><td>coding</td><td>0..1</td><td><a href="datatypes.html#Coding">Coding</a></td><td/><td><div><p>A coding to look up</p>\n</div></td></tr><tr><td>IN</td><td>date</td><td>0..1</td><td><a href="datatypes.html#dateTime">dateTime</a></td><td/><td><div><p>The date for which the information should be returned. Normally, this is the current conditions (which is the default value) but under some circumstances, systems need to acccess this information as it would have been in the past. A typical example of this would be where code selection is constrained to the set of codes that were available when the patient was treated, not when the record is being edited. Note that which date is appropriate is a matter for implementation policy.</p>\n</div></td></tr><tr><td>IN</td><td>displayLanguage</td><td>0..1</td><td><a href="datatypes.html#code">code</a></td><td/><td><div><p>The requested language for display (see $expand.displayLanguage)</p>\n</div></td></tr><tr><td>IN</td><td>property</td><td>0..*</td><td><a href="datatypes.html#code">code</a></td><td/><td><div><p>A property that the client wishes to be returned in the output. If no properties are specified, the server chooses what to return. The following properties are defined for all code systems: url, name, version (code system info) and code information: display, definition, designation, parent and child, and for designations, lang.X where X is a designation language code. Some of the properties are returned explicit in named parameters (when the names match), and the rest (except for lang.X) in the property parameter group</p>\n</div></td></tr><tr><td>OUT</td><td>name</td><td>1..1</td><td><a href="datatypes.html#string">string</a></td><td/><td><div><p>A display name for the code system</p>\n</div></td></tr><tr><td>OUT</td><td>version</td><td>0..1</td><td><a href="datatypes.html#string">string</a></td><td/><td><div><p>The version that these details are based on</p>\n</div></td></tr><tr><td>OUT</td><td>display</td><td>1..1</td><td><a href="datatypes.html#string">string</a></td><td/><td><div><p>The preferred display for this concept</p>\n</div></td></tr><tr><td>OUT</td><td>designation</td><td>0..*</td><td></td><td/><td><div><p>Additional representations for this concept</p>\n</div></td></tr><tr><td>OUT</td><td>designation.language</td><td>0..1</td><td><a href="datatypes.html#code">code</a></td><td/><td><div><p>The language this designation is defined for</p>\n</div></td></tr><tr><td>OUT</td><td>designation.use</td><td>0..1</td><td><a href="datatypes.html#Coding">Coding</a></td><td/><td><div><p>A code that details how this designation would be used</p>\n</div></td></tr><tr><td>OUT</td><td>designation.value</td><td>1..1</td><td><a href="datatypes.html#string">string</a></td><td/><td><div><p>The text value for this designation</p>\n</div></td></tr><tr><td>OUT</td><td>property</td><td>0..*</td><td></td><td/><td><div><p>One or more properties that contain additional information about the code, including status. For complex terminologies (e.g. SNOMED CT, LOINC, medications), these properties serve to decompose the code</p>\n</div></td></tr><tr><td>OUT</td><td>property.code</td><td>1..1</td><td><a href="datatypes.html#code">code</a></td><td/><td><div><p>Identifies the property returned</p>\n</div></td></tr><tr><td>OUT</td><td>property.value</td><td>0..1</td><td><a href="datatypes.html#code">code</a> | <a href="datatypes.html#Coding">Coding</a> | <a href="datatypes.html#string">string</a> | <a href="datatypes.html#integer">integer</a> | <a href="datatypes.html#boolean">boolean</a> | <a href="datatypes.html#dateTime">dateTime</a> | <a href="datatypes.html#decimal">decimal</a></td><td/><td><div><p>The value of the property returned</p>\n</div></td></tr><tr><td>OUT</td><td>property.description</td><td>0..1</td><td><a href="datatypes.html#string">string</a></td><td/><td><div><p>Human Readable representation of the property value (e.g. display for a code)</p>\n</div></td></tr><tr><td>OUT</td><td>property.subproperty</td><td>0..*</td><td></td><td/><td><div><p>Nested Properties (mainly used for SNOMED CT decomposition, for relationship Groups)</p>\n</div></td></tr><tr><td>OUT</td><td>property.subproperty.code</td><td>1..1</td><td><a href="datatypes.html#code">code</a></td><td/><td><div><p>Identifies the sub-property returned</p>\n</div></td></tr><tr><td>OUT</td><td>property.subproperty.value</td><td>1..1</td><td><a href="datatypes.html#code">code</a> | <a href="datatypes.html#Coding">Coding</a> | <a href="datatypes.html#string">string</a> | <a href="datatypes.html#integer">integer</a> | <a href="datatypes.html#boolean">boolean</a> | <a href="datatypes.html#dateTime">dateTime</a> | <a href="datatypes.html#decimal">decimal</a></td><td/><td><div><p>The value of the sub-property returned</p>\n</div></td></tr><tr><td>OUT</td><td>property.subproperty.description</td><td>0..1</td><td><a href="datatypes.html#string">string</a></td><td/><td><div><p>Human Readable representation of the property value (e.g. display for a code)</p>\n</div></td></tr></table><div><p>Note that the $lookup operation is more than just a code system search  - the server finds the concept, and gathers the return information from the underlying code system definitions.</p>\n</div></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 5,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "normative",
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-normative-version",
        valueCode: "4.0.1",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/CodeSystem-lookup",
    version: "4.0.1",
    name: "Concept Look Up & Decomposition",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "Given a code/system, or a Coding, get additional details about the concept, including definition, status, designations, and properties. One of the products of this operation is a full decomposition of a code from a structured terminology.\n\nWhen invoking this operation, a client SHALL provide both a system and a code, either using the system+code parameters, or in the coding parameter. Other parameters are optional",
    code: "lookup",
    comment:
      "Note that the $lookup operation is more than just a code system search  - the server finds the concept, and gathers the return information from the underlying code system definitions.",
    resource: ["CodeSystem"],
    system: false,
    type: true,
    instance: false,
    parameter: [
      {
        name: "code",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The code that is to be located. If a code is provided, a system must be provided",
        type: "code",
      },
      {
        name: "system",
        use: "in",
        min: 0,
        max: "1",
        documentation: "The system for the code that is to be located",
        type: "uri",
      },
      {
        name: "version",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The version of the system, if one was provided in the source data",
        type: "string",
      },
      {
        name: "coding",
        use: "in",
        min: 0,
        max: "1",
        documentation: "A coding to look up",
        type: "Coding",
      },
      {
        name: "date",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The date for which the information should be returned. Normally, this is the current conditions (which is the default value) but under some circumstances, systems need to acccess this information as it would have been in the past. A typical example of this would be where code selection is constrained to the set of codes that were available when the patient was treated, not when the record is being edited. Note that which date is appropriate is a matter for implementation policy.",
        type: "dateTime",
      },
      {
        name: "displayLanguage",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The requested language for display (see $expand.displayLanguage)",
        type: "code",
      },
      {
        name: "property",
        use: "in",
        min: 0,
        max: "*",
        documentation:
          "A property that the client wishes to be returned in the output. If no properties are specified, the server chooses what to return. The following properties are defined for all code systems: url, name, version (code system info) and code information: display, definition, designation, parent and child, and for designations, lang.X where X is a designation language code. Some of the properties are returned explicit in named parameters (when the names match), and the rest (except for lang.X) in the property parameter group",
        type: "code",
      },
      {
        name: "name",
        use: "out",
        min: 1,
        max: "1",
        documentation: "A display name for the code system",
        type: "string",
      },
      {
        name: "version",
        use: "out",
        min: 0,
        max: "1",
        documentation: "The version that these details are based on",
        type: "string",
      },
      {
        name: "display",
        use: "out",
        min: 1,
        max: "1",
        documentation: "The preferred display for this concept",
        type: "string",
      },
      {
        name: "designation",
        use: "out",
        min: 0,
        max: "*",
        documentation: "Additional representations for this concept",
        part: [
          {
            name: "language",
            use: "out",
            min: 0,
            max: "1",
            documentation: "The language this designation is defined for",
            type: "code",
          },
          {
            name: "use",
            use: "out",
            min: 0,
            max: "1",
            documentation:
              "A code that details how this designation would be used",
            type: "Coding",
          },
          {
            name: "value",
            use: "out",
            min: 1,
            max: "1",
            documentation: "The text value for this designation",
            type: "string",
          },
        ],
      },
      {
        name: "property",
        use: "out",
        min: 0,
        max: "*",
        documentation:
          "One or more properties that contain additional information about the code, including status. For complex terminologies (e.g. SNOMED CT, LOINC, medications), these properties serve to decompose the code",
        part: [
          {
            name: "code",
            use: "out",
            min: 1,
            max: "1",
            documentation: "Identifies the property returned",
            type: "code",
          },
          {
            extension: [
              {
                url: "http://hl7.org/fhir/StructureDefinition/operationdefinition-allowed-type",
                valueUri: "code",
              },
              {
                url: "http://hl7.org/fhir/StructureDefinition/operationdefinition-allowed-type",
                valueUri: "Coding",
              },
              {
                url: "http://hl7.org/fhir/StructureDefinition/operationdefinition-allowed-type",
                valueUri: "string",
              },
              {
                url: "http://hl7.org/fhir/StructureDefinition/operationdefinition-allowed-type",
                valueUri: "integer",
              },
              {
                url: "http://hl7.org/fhir/StructureDefinition/operationdefinition-allowed-type",
                valueUri: "boolean",
              },
              {
                url: "http://hl7.org/fhir/StructureDefinition/operationdefinition-allowed-type",
                valueUri: "dateTime",
              },
              {
                url: "http://hl7.org/fhir/StructureDefinition/operationdefinition-allowed-type",
                valueUri: "decimal",
              },
            ],
            name: "value",
            use: "out",
            min: 0,
            max: "1",
            documentation: "The value of the property returned",
            type: "Element",
          },
          {
            name: "description",
            use: "out",
            min: 0,
            max: "1",
            documentation:
              "Human Readable representation of the property value (e.g. display for a code)",
            type: "string",
          },
          {
            name: "subproperty",
            use: "out",
            min: 0,
            max: "*",
            documentation:
              "Nested Properties (mainly used for SNOMED CT decomposition, for relationship Groups)",
            part: [
              {
                name: "code",
                use: "out",
                min: 1,
                max: "1",
                documentation: "Identifies the sub-property returned",
                type: "code",
              },
              {
                extension: [
                  {
                    url: "http://hl7.org/fhir/StructureDefinition/operationdefinition-allowed-type",
                    valueUri: "code",
                  },
                  {
                    url: "http://hl7.org/fhir/StructureDefinition/operationdefinition-allowed-type",
                    valueUri: "Coding",
                  },
                  {
                    url: "http://hl7.org/fhir/StructureDefinition/operationdefinition-allowed-type",
                    valueUri: "string",
                  },
                  {
                    url: "http://hl7.org/fhir/StructureDefinition/operationdefinition-allowed-type",
                    valueUri: "integer",
                  },
                  {
                    url: "http://hl7.org/fhir/StructureDefinition/operationdefinition-allowed-type",
                    valueUri: "boolean",
                  },
                  {
                    url: "http://hl7.org/fhir/StructureDefinition/operationdefinition-allowed-type",
                    valueUri: "dateTime",
                  },
                  {
                    url: "http://hl7.org/fhir/StructureDefinition/operationdefinition-allowed-type",
                    valueUri: "decimal",
                  },
                ],
                name: "value",
                use: "out",
                min: 1,
                max: "1",
                documentation: "The value of the sub-property returned",
                type: "Element",
              },
              {
                name: "description",
                use: "out",
                min: 0,
                max: "1",
                documentation:
                  "Human Readable representation of the property value (e.g. display for a code)",
                type: "string",
              },
            ],
          },
        ],
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace CodeSystemSubsumes {
  export type Input = {
    codeA?: fhirTypes.code;
    codeB?: fhirTypes.code;
    system?: fhirTypes.uri;
    version?: fhirTypes.string;
    codingA?: fhirTypes.Coding;
    codingB?: fhirTypes.Coding;
  };
  export type Output = { outcome: fhirTypes.code };
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "CodeSystem-subsumes",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Subsumption Testing</h2><p>OPERATION: Subsumption Testing</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/CodeSystem-subsumes</pre><div><p>Test the subsumption relationship between code/Coding A and code/Coding B given the semantics of subsumption in the underlying code system (see <a href="codesystem-definitions.html#CodeSystem.hierarchyMeaning">hierarchyMeaning</a>).</p>\n<p>When invoking this operation, a client SHALL provide both a and codes, either as code or Coding parameters. The system parameter is required unless the operation is invoked on an instance of a code system resource. Other parameters are optional</p>\n</div><p>URL: [base]/CodeSystem/$subsumes</p><p>URL: [base]/CodeSystem/[id]/$subsumes</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>codeA</td><td>0..1</td><td><a href="datatypes.html#code">code</a></td><td/><td><div><p>The &quot;A&quot; code that is to be tested. If a code is provided, a system must be provided</p>\n</div></td></tr><tr><td>IN</td><td>codeB</td><td>0..1</td><td><a href="datatypes.html#code">code</a></td><td/><td><div><p>The &quot;B&quot; code that is to be tested. If a code is provided, a system must be provided</p>\n</div></td></tr><tr><td>IN</td><td>system</td><td>0..1</td><td><a href="datatypes.html#uri">uri</a></td><td/><td><div><p>The code system in which subsumption testing is to be performed. This must be provided unless the operation is invoked on a code system instance</p>\n</div></td></tr><tr><td>IN</td><td>version</td><td>0..1</td><td><a href="datatypes.html#string">string</a></td><td/><td><div><p>The version of the code system, if one was provided in the source data</p>\n</div></td></tr><tr><td>IN</td><td>codingA</td><td>0..1</td><td><a href="datatypes.html#Coding">Coding</a></td><td/><td><div><p>The &quot;A&quot; Coding that is to be tested. The code system does not have to match the specified subsumption code system, but the relationships between the code systems must be well established</p>\n</div></td></tr><tr><td>IN</td><td>codingB</td><td>0..1</td><td><a href="datatypes.html#Coding">Coding</a></td><td/><td><div><p>The &quot;B&quot; Coding that is to be tested. The code system does not have to match the specified subsumption code system, but the relationships between the code systems must be well established</p>\n</div></td></tr><tr><td>OUT</td><td>outcome</td><td>1..1</td><td><a href="datatypes.html#code">code</a></td><td><a href="valueset-concept-subsumption-outcome.html">http://hl7.org/fhir/ValueSet/concept-subsumption-outcome|4.0.1</a> (Required)</td><td><div><p>The subsumption relationship between code/Coding &quot;A&quot; and code/Coding &quot;B&quot;. There are 4 possible codes to be returned (equivalent, subsumes, subsumed-by, and not-subsumed) as defined in the concept-subsumption-outcome value set.  If the server is unable to determine the relationship between the codes/Codings, then it returns an error response with an OperationOutcome.</p>\n</div></td></tr></table><div/></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 5,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "normative",
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-normative-version",
        valueCode: "4.0.1",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/CodeSystem-subsumes",
    version: "4.0.1",
    name: "Subsumption Testing",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "Test the subsumption relationship between code/Coding A and code/Coding B given the semantics of subsumption in the underlying code system (see [hierarchyMeaning](codesystem-definitions.html#CodeSystem.hierarchyMeaning)).\n\nWhen invoking this operation, a client SHALL provide both a and codes, either as code or Coding parameters. The system parameter is required unless the operation is invoked on an instance of a code system resource. Other parameters are optional",
    code: "subsumes",
    resource: ["CodeSystem"],
    system: false,
    type: true,
    instance: true,
    parameter: [
      {
        name: "codeA",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          'The "A" code that is to be tested. If a code is provided, a system must be provided',
        type: "code",
      },
      {
        name: "codeB",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          'The "B" code that is to be tested. If a code is provided, a system must be provided',
        type: "code",
      },
      {
        name: "system",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The code system in which subsumption testing is to be performed. This must be provided unless the operation is invoked on a code system instance",
        type: "uri",
      },
      {
        name: "version",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The version of the code system, if one was provided in the source data",
        type: "string",
      },
      {
        name: "codingA",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          'The "A" Coding that is to be tested. The code system does not have to match the specified subsumption code system, but the relationships between the code systems must be well established',
        type: "Coding",
      },
      {
        name: "codingB",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          'The "B" Coding that is to be tested. The code system does not have to match the specified subsumption code system, but the relationships between the code systems must be well established',
        type: "Coding",
      },
      {
        name: "outcome",
        use: "out",
        min: 1,
        max: "1",
        documentation:
          'The subsumption relationship between code/Coding "A" and code/Coding "B". There are 4 possible codes to be returned (equivalent, subsumes, subsumed-by, and not-subsumed) as defined in the concept-subsumption-outcome value set.  If the server is unable to determine the relationship between the codes/Codings, then it returns an error response with an OperationOutcome.',
        type: "code",
        binding: {
          strength: "required",
          valueSet:
            "http://hl7.org/fhir/ValueSet/concept-subsumption-outcome|4.0.1",
        },
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace CodeSystemValidateCode {
  export type Input = {
    url?: fhirTypes.uri;
    codeSystem?: fhirTypes.CodeSystem;
    code?: fhirTypes.code;
    version?: fhirTypes.string;
    display?: fhirTypes.string;
    coding?: fhirTypes.Coding;
    codeableConcept?: fhirTypes.CodeableConcept;
    date?: fhirTypes.dateTime;
    abstract?: fhirTypes.boolean;
    displayLanguage?: fhirTypes.code;
  };
  export type Output = {
    result: fhirTypes.boolean;
    message?: fhirTypes.string;
    display?: fhirTypes.string;
  };
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "CodeSystem-validate-code",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Code System based Validation</h2><p>OPERATION: Code System based Validation</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/CodeSystem-validate-code</pre><div><p>Validate that a coded value is in the code system. If the operation is not called at the instance level, one of the parameters &quot;url&quot; or &quot;codeSystem&quot; must be provided. The operation returns a result (true / false), an error message, and the recommended display for the code.</p>\n<p>When invoking this operation, a client SHALL provide one (and only one) of the parameters (code+system, coding, or codeableConcept). Other parameters (including version and display) are optional</p>\n</div><p>URL: [base]/CodeSystem/$validate-code</p><p>URL: [base]/CodeSystem/[id]/$validate-code</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>url</td><td>0..1</td><td><a href="datatypes.html#uri">uri</a></td><td/><td><div><p>CodeSystem URL. The server must know the code system (e.g. it is defined explicitly in the server\'scode systems, or it is known implicitly by the server</p>\n</div></td></tr><tr><td>IN</td><td>codeSystem</td><td>0..1</td><td><a href="codesystem.html">CodeSystem</a></td><td/><td><div><p>The codeSystem is provided directly as part of the request. Servers may choose not to accept code systems in this fashion. This parameter is used when the client wants the server to check against a code system that is not stored on the server</p>\n</div></td></tr><tr><td>IN</td><td>code</td><td>0..1</td><td><a href="datatypes.html#code">code</a></td><td/><td><div><p>The code that is to be validated</p>\n</div></td></tr><tr><td>IN</td><td>version</td><td>0..1</td><td><a href="datatypes.html#string">string</a></td><td/><td><div><p>The version of the code system, if one was provided in the source data</p>\n</div></td></tr><tr><td>IN</td><td>display</td><td>0..1</td><td><a href="datatypes.html#string">string</a></td><td/><td><div><p>The display associated with the code, if provided. If a display is provided a code must be provided. If no display is provided, the server cannot validate the display value, but may choose to return a recommended display name in an extension in the outcome. Whether displays are case sensitive is code system dependent</p>\n</div></td></tr><tr><td>IN</td><td>coding</td><td>0..1</td><td><a href="datatypes.html#Coding">Coding</a></td><td/><td><div><p>A coding to validate. The system must match the specified code system</p>\n</div></td></tr><tr><td>IN</td><td>codeableConcept</td><td>0..1</td><td><a href="datatypes.html#CodeableConcept">CodeableConcept</a></td><td/><td><div><p>A full codeableConcept to validate. The server returns true if one of the coding values is in the code system, and may also validate that the codings are not in conflict with each other if more than one is present</p>\n</div></td></tr><tr><td>IN</td><td>date</td><td>0..1</td><td><a href="datatypes.html#dateTime">dateTime</a></td><td/><td><div><p>The date for which the validation should be checked. Normally, this is the current conditions (which is the default values) but under some circumstances, systems need to validate that a correct code was used at some point in the past. A typical example of this would be where code selection is constrained to the set of codes that were available when the patient was treated, not when the record is being edited. Note that which date is appropriate is a matter for implementation policy.</p>\n</div></td></tr><tr><td>IN</td><td>abstract</td><td>0..1</td><td><a href="datatypes.html#boolean">boolean</a></td><td/><td><div><p>If this parameter has a value of true, the client is stating that the validation is being performed in a context where a concept designated as \'abstract\' is appropriate/allowed to be used, and the server should regard abstract codes as valid. If this parameter is false, abstract codes are not considered to be valid.</p>\n<p>Note that. \'abstract\' is a property defined by many HL7 code systems that indicates that the concept is a logical grouping concept that is not intended to be used asa \'concrete\' concept to in an actual patient/care/process record. This language is borrowed from Object Orienated theory where \'asbtract\' objects are never instantiated. However in the general record and terminology eco-system, there are many contexts where it is appropraite to use these codes e.g. as decision making criterion, or when editing value sets themselves. This parameter allows a client to indicate to the server that it is working in such a context.</p>\n</div></td></tr><tr><td>IN</td><td>displayLanguage</td><td>0..1</td><td><a href="datatypes.html#code">code</a></td><td/><td><div><p>Specifies the language to be used for description when validating the display property</p>\n</div></td></tr><tr><td>OUT</td><td>result</td><td>1..1</td><td><a href="datatypes.html#boolean">boolean</a></td><td/><td><div><p>True if the concept details supplied are valid</p>\n</div></td></tr><tr><td>OUT</td><td>message</td><td>0..1</td><td><a href="datatypes.html#string">string</a></td><td/><td><div><p>Error details, if result = false. If this is provided when result = true, the message carries hints and warnings</p>\n</div></td></tr><tr><td>OUT</td><td>display</td><td>0..1</td><td><a href="datatypes.html#string">string</a></td><td/><td><div><p>A valid display for the concept if the system wishes to display this to a user</p>\n</div></td></tr></table><div/></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 5,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "normative",
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-normative-version",
        valueCode: "4.0.1",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/CodeSystem-validate-code",
    version: "4.0.1",
    name: "Code System based Validation",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      'Validate that a coded value is in the code system. If the operation is not called at the instance level, one of the parameters "url" or "codeSystem" must be provided. The operation returns a result (true / false), an error message, and the recommended display for the code.\n\nWhen invoking this operation, a client SHALL provide one (and only one) of the parameters (code+system, coding, or codeableConcept). Other parameters (including version and display) are optional',
    code: "validate-code",
    resource: ["CodeSystem"],
    system: false,
    type: true,
    instance: true,
    parameter: [
      {
        name: "url",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "CodeSystem URL. The server must know the code system (e.g. it is defined explicitly in the server'scode systems, or it is known implicitly by the server",
        type: "uri",
      },
      {
        name: "codeSystem",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The codeSystem is provided directly as part of the request. Servers may choose not to accept code systems in this fashion. This parameter is used when the client wants the server to check against a code system that is not stored on the server",
        type: "CodeSystem",
      },
      {
        name: "code",
        use: "in",
        min: 0,
        max: "1",
        documentation: "The code that is to be validated",
        type: "code",
      },
      {
        name: "version",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The version of the code system, if one was provided in the source data",
        type: "string",
      },
      {
        name: "display",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The display associated with the code, if provided. If a display is provided a code must be provided. If no display is provided, the server cannot validate the display value, but may choose to return a recommended display name in an extension in the outcome. Whether displays are case sensitive is code system dependent",
        type: "string",
      },
      {
        name: "coding",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "A coding to validate. The system must match the specified code system",
        type: "Coding",
      },
      {
        name: "codeableConcept",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "A full codeableConcept to validate. The server returns true if one of the coding values is in the code system, and may also validate that the codings are not in conflict with each other if more than one is present",
        type: "CodeableConcept",
      },
      {
        name: "date",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The date for which the validation should be checked. Normally, this is the current conditions (which is the default values) but under some circumstances, systems need to validate that a correct code was used at some point in the past. A typical example of this would be where code selection is constrained to the set of codes that were available when the patient was treated, not when the record is being edited. Note that which date is appropriate is a matter for implementation policy.",
        type: "dateTime",
      },
      {
        name: "abstract",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "If this parameter has a value of true, the client is stating that the validation is being performed in a context where a concept designated as 'abstract' is appropriate/allowed to be used, and the server should regard abstract codes as valid. If this parameter is false, abstract codes are not considered to be valid.\n\nNote that. 'abstract' is a property defined by many HL7 code systems that indicates that the concept is a logical grouping concept that is not intended to be used asa 'concrete' concept to in an actual patient/care/process record. This language is borrowed from Object Orienated theory where 'asbtract' objects are never instantiated. However in the general record and terminology eco-system, there are many contexts where it is appropraite to use these codes e.g. as decision making criterion, or when editing value sets themselves. This parameter allows a client to indicate to the server that it is working in such a context.",
        type: "boolean",
      },
      {
        name: "displayLanguage",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "Specifies the language to be used for description when validating the display property",
        type: "code",
      },
      {
        name: "result",
        use: "out",
        min: 1,
        max: "1",
        documentation: "True if the concept details supplied are valid",
        type: "boolean",
      },
      {
        name: "message",
        use: "out",
        min: 0,
        max: "1",
        documentation:
          "Error details, if result = false. If this is provided when result = true, the message carries hints and warnings",
        type: "string",
      },
      {
        name: "display",
        use: "out",
        min: 0,
        max: "1",
        documentation:
          "A valid display for the concept if the system wishes to display this to a user",
        type: "string",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace CompositionDocument {
  export type Input = {
    id?: fhirTypes.uri;
    persist?: fhirTypes.boolean;
    graph?: fhirTypes.uri;
  };
  export type Output = Record<string, never>;
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "Composition-document",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Generate a Document</h2><p>OPERATION: Generate a Document</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/Composition-document</pre><div><p>A client can ask a server to generate a fully bundled document from a composition resource. The server takes the composition resource, locates all the referenced resources and other additional resources as configured or requested and either returns a full document bundle, or returns an error. Note that since this is a search operation, the document bundle is  wrapped inside the search bundle.  If some of the resources are located on other servers, it is at the discretion of the  server whether to retrieve them or return an error. If the correct version of the document  that would be generated already exists, then the server can return the existing one.</p>\n</div><p>URL: [base]/Composition/$document</p><p>URL: [base]/Composition/[id]/$document</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>id</td><td>0..1</td><td><a href="datatypes.html#uri">uri</a></td><td/><td><div><p>Identifies the composition to use. This can either be a simple id, which identifies a composition, or it can be a full URL, which identifies a composition on another server.</p>\n<p>Notes:</p>\n<ul>\n<li>GET [base]/Composition/[id]/$document is identical in meaning to GET [base]/Composition/$document?id=[id]</li>\n<li>the id parameter SHALL NOT be used if the operation is requested on a particular composition (e.g.  GET [base]/Composition/[id]/$document?id=[id] is not allowed)</li>\n<li>Servers are not required to support generating documents on Compositions located on another server</li>\n</ul>\n</div></td></tr><tr><td>IN</td><td>persist</td><td>0..1</td><td><a href="datatypes.html#boolean">boolean</a></td><td/><td><div><p>Whether to store the document at the bundle end-point (/Bundle) or not once it is generated. Value = true or false (default is for the server to decide). If the document is stored, it\'s location can be inferred from the Bundle.id, but it SHOULD be provided explicitly in the HTTP Location header in the response</p>\n</div></td></tr><tr><td>IN</td><td>graph</td><td>0..1</td><td><a href="datatypes.html#uri">uri</a></td><td/><td><div><p>Canonical reference to a GraphDefinition. If a URL is provided, it is the canonical reference to a <a href="graphdefinition.html">GraphDefinition</a> that it controls what resources are to be added to the bundle when building the document. The GraphDefinition can also specify profiles that apply to the various resources</p>\n</div></td></tr></table><div><p>Note: this operation definition does not resolve the question how document signatures are created. This is an open issue during the period of trial use, and feedback is requested regarding this question</p>\n</div></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 2,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/Composition-document",
    version: "4.0.1",
    name: "Generate a Document",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "A client can ask a server to generate a fully bundled document from a composition resource. The server takes the composition resource, locates all the referenced resources and other additional resources as configured or requested and either returns a full document bundle, or returns an error. Note that since this is a search operation, the document bundle is  wrapped inside the search bundle.  If some of the resources are located on other servers, it is at the discretion of the  server whether to retrieve them or return an error. If the correct version of the document  that would be generated already exists, then the server can return the existing one.",
    code: "document",
    comment:
      "Note: this operation definition does not resolve the question how document signatures are created. This is an open issue during the period of trial use, and feedback is requested regarding this question",
    resource: ["Composition"],
    system: false,
    type: true,
    instance: true,
    parameter: [
      {
        name: "id",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "Identifies the composition to use. This can either be a simple id, which identifies a composition, or it can be a full URL, which identifies a composition on another server. \n\nNotes: \n\n* GET [base]/Composition/[id]/$document is identical in meaning to GET [base]/Composition/$document?id=[id]\n* the id parameter SHALL NOT be used if the operation is requested on a particular composition (e.g.  GET [base]/Composition/[id]/$document?id=[id] is not allowed)\n* Servers are not required to support generating documents on Compositions located on another server",
        type: "uri",
      },
      {
        name: "persist",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "Whether to store the document at the bundle end-point (/Bundle) or not once it is generated. Value = true or false (default is for the server to decide). If the document is stored, it's location can be inferred from the Bundle.id, but it SHOULD be provided explicitly in the HTTP Location header in the response",
        type: "boolean",
      },
      {
        name: "graph",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "Canonical reference to a GraphDefinition. If a URL is provided, it is the canonical reference to a [GraphDefinition](graphdefinition.html) that it controls what resources are to be added to the bundle when building the document. The GraphDefinition can also specify profiles that apply to the various resources",
        type: "uri",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace ConceptMapClosure {
  export type Input = {
    name: fhirTypes.string;
    concept?: Array<fhirTypes.Coding>;
    version?: fhirTypes.string;
  };
  export type Output = fhirTypes.ConceptMap;
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "ConceptMap-closure",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Closure Table Maintenance</h2><p>OPERATION: Closure Table Maintenance</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/ConceptMap-closure</pre><div><p>This operation provides support for ongoing maintenance of a client-side <a href="https://en.wikipedia.org/wiki/Transitive_closure#In_graph_theory">transitive closure table</a> based on server-side terminological logic. For details of how this is used, see <a href="terminology-service.html#closure">Maintaining a Closure Table</a></p>\n</div><p>URL: [base]/$closure</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>name</td><td>1..1</td><td><a href="datatypes.html#string">string</a></td><td/><td><div><p>The name that defines the particular context for the subsumption based closure table</p>\n</div></td></tr><tr><td>IN</td><td>concept</td><td>0..*</td><td><a href="datatypes.html#Coding">Coding</a></td><td/><td><div><p>Concepts to add to the closure table</p>\n</div></td></tr><tr><td>IN</td><td>version</td><td>0..1</td><td><a href="datatypes.html#string">string</a></td><td/><td><div><p>A request to resynchronise - request to send all new entries since the nominated version was sent by the server</p>\n</div></td></tr><tr><td>OUT</td><td>return</td><td>1..1</td><td><a href="conceptmap.html">ConceptMap</a></td><td/><td><div><p>A list of new entries (code / system --&gt; code/system) that the client should add to its closure table. The only kind of entry mapping equivalences that can be returned are equal, specializes, subsumes and unmatched</p>\n</div></td></tr></table><div/></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 3,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/ConceptMap-closure",
    version: "4.0.1",
    name: "Closure Table Maintenance",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "This operation provides support for ongoing maintenance of a client-side [transitive closure table](https://en.wikipedia.org/wiki/Transitive_closure#In_graph_theory) based on server-side terminological logic. For details of how this is used, see [Maintaining a Closure Table](terminology-service.html#closure)",
    code: "closure",
    resource: ["ConceptMap"],
    system: true,
    type: false,
    instance: false,
    parameter: [
      {
        name: "name",
        use: "in",
        min: 1,
        max: "1",
        documentation:
          "The name that defines the particular context for the subsumption based closure table",
        type: "string",
      },
      {
        name: "concept",
        use: "in",
        min: 0,
        max: "*",
        documentation: "Concepts to add to the closure table",
        type: "Coding",
      },
      {
        name: "version",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "A request to resynchronise - request to send all new entries since the nominated version was sent by the server",
        type: "string",
      },
      {
        name: "return",
        use: "out",
        min: 1,
        max: "1",
        documentation:
          "A list of new entries (code / system --> code/system) that the client should add to its closure table. The only kind of entry mapping equivalences that can be returned are equal, specializes, subsumes and unmatched",
        type: "ConceptMap",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace ConceptMapTranslate {
  export type Input = {
    url?: fhirTypes.uri;
    conceptMap?: fhirTypes.ConceptMap;
    conceptMapVersion?: fhirTypes.string;
    code?: fhirTypes.code;
    system?: fhirTypes.uri;
    version?: fhirTypes.string;
    source?: fhirTypes.uri;
    coding?: fhirTypes.Coding;
    codeableConcept?: fhirTypes.CodeableConcept;
    target?: fhirTypes.uri;
    targetsystem?: fhirTypes.uri;
    dependency?: Array<{
      element?: fhirTypes.uri;
      concept?: fhirTypes.CodeableConcept;
    }>;
    reverse?: fhirTypes.boolean;
  };
  export type Output = {
    result: fhirTypes.boolean;
    message?: fhirTypes.string;
    match?: Array<{
      equivalence?: fhirTypes.code;
      concept?: fhirTypes.Coding;
      product?: Array<{ element?: fhirTypes.uri; concept?: fhirTypes.Coding }>;
      source?: fhirTypes.uri;
    }>;
  };
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "ConceptMap-translate",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Concept Translation</h2><p>OPERATION: Concept Translation</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/ConceptMap-translate</pre><div><p>Translate a code from one value set to another, based on the existing value set and concept maps resources, and/or other additional knowledge available to the server.</p>\n<p>One (and only one) of the in parameters (code, coding, codeableConcept) must be provided, to identify the code that is to be translated.</p>\n<p>The operation returns a set of parameters including a \'result\' for whether there is an acceptable match, and a list of possible matches. Note that the list of matches may include notes of codes for which mapping is specifically excluded, so implementers have to check the match.equivalence for each match</p>\n</div><p>URL: [base]/ConceptMap/$translate</p><p>URL: [base]/ConceptMap/[id]/$translate</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>url</td><td>0..1</td><td><a href="datatypes.html#uri">uri</a></td><td/><td><div><p>A canonical URL for a concept map. The server must know the concept map (e.g. it is defined explicitly in the server\'s concept maps, or it is defined implicitly by some code system known to the server.</p>\n</div></td></tr><tr><td>IN</td><td>conceptMap</td><td>0..1</td><td><a href="conceptmap.html">ConceptMap</a></td><td/><td><div><p>The concept map is provided directly as part of the request. Servers may choose not to accept concept maps in this fashion.</p>\n</div></td></tr><tr><td>IN</td><td>conceptMapVersion</td><td>0..1</td><td><a href="datatypes.html#string">string</a></td><td/><td><div><p>The identifier that is used to identify a specific version of the concept map to be used for the translation. This is an arbitrary value managed by the concept map author and is not expected to be globally unique. For example, it might be a timestamp (e.g. yyyymmdd) if a managed version is not available.</p>\n</div></td></tr><tr><td>IN</td><td>code</td><td>0..1</td><td><a href="datatypes.html#code">code</a></td><td/><td><div><p>The code that is to be translated. If a code is provided, a system must be provided</p>\n</div></td></tr><tr><td>IN</td><td>system</td><td>0..1</td><td><a href="datatypes.html#uri">uri</a></td><td/><td><div><p>The system for the code that is to be translated</p>\n</div></td></tr><tr><td>IN</td><td>version</td><td>0..1</td><td><a href="datatypes.html#string">string</a></td><td/><td><div><p>The version of the system, if one was provided in the source data</p>\n</div></td></tr><tr><td>IN</td><td>source</td><td>0..1</td><td><a href="datatypes.html#uri">uri</a></td><td/><td><div><p>Identifies the value set used when the concept (system/code pair) was chosen. May be a logical id, or an absolute or relative location. The source value set is an optional parameter because in some cases, the client cannot know what the source value set is. However, without a source value set, the server may be unable to safely identify an applicable concept map, and would return an error. For this reason, a source value set SHOULD always be provided. Note that servers may be able to identify an appropriate concept map without a source value set if there is a full mapping for the entire code system in the concept map, or by manual intervention</p>\n</div></td></tr><tr><td>IN</td><td>coding</td><td>0..1</td><td><a href="datatypes.html#Coding">Coding</a></td><td/><td><div><p>A coding to translate</p>\n</div></td></tr><tr><td>IN</td><td>codeableConcept</td><td>0..1</td><td><a href="datatypes.html#CodeableConcept">CodeableConcept</a></td><td/><td><div><p>A full codeableConcept to validate. The server can translate any of the coding values (e.g. existing translations) as it chooses</p>\n</div></td></tr><tr><td>IN</td><td>target</td><td>0..1</td><td><a href="datatypes.html#uri">uri</a></td><td/><td><div><p>Identifies the value set in which a translation is sought. May be a logical id, or an absolute or relative location. If there\'s no target specified, the server should return all known translations, along with their source</p>\n</div></td></tr><tr><td>IN</td><td>targetsystem</td><td>0..1</td><td><a href="datatypes.html#uri">uri</a></td><td/><td><div><p>identifies a target code system in which a mapping is sought. This parameter is an alternative to the target parameter - only one is required. Searching for any translation to a target code system irrespective of the context (e.g. target valueset) may lead to unsafe results, and it is at the discretion of the server to decide when to support this operation</p>\n</div></td></tr><tr><td>IN</td><td>dependency</td><td>0..*</td><td></td><td/><td><div><p>Another element that may help produce the correct mapping</p>\n</div></td></tr><tr><td>IN</td><td>dependency.element</td><td>0..1</td><td><a href="datatypes.html#uri">uri</a></td><td/><td><div><p>The element for this dependency</p>\n</div></td></tr><tr><td>IN</td><td>dependency.concept</td><td>0..1</td><td><a href="datatypes.html#CodeableConcept">CodeableConcept</a></td><td/><td><div><p>The value for this dependency</p>\n</div></td></tr><tr><td>IN</td><td>reverse</td><td>0..1</td><td><a href="datatypes.html#boolean">boolean</a></td><td/><td><div><p>if this is true, then the operation should return all the codes that might be mapped to this code. This parameter reverses the meaning of the source and target parameters</p>\n</div></td></tr><tr><td>OUT</td><td>result</td><td>1..1</td><td><a href="datatypes.html#boolean">boolean</a></td><td/><td><div><p>True if the concept could be translated successfully. The value can only be true if at least one returned match has an equivalence which is not  unmatched or disjoint</p>\n</div></td></tr><tr><td>OUT</td><td>message</td><td>0..1</td><td><a href="datatypes.html#string">string</a></td><td/><td><div><p>Error details, for display to a human. If this is provided when result = true, the message carries hints and warnings (e.g. a note that the matches could be improved by providing additional detail)</p>\n</div></td></tr><tr><td>OUT</td><td>match</td><td>0..*</td><td></td><td/><td><div><p>A concept in the target value set with an equivalence. Note that there may be multiple matches of equal or differing equivalence, and the matches may include equivalence values that mean that there is no match</p>\n</div></td></tr><tr><td>OUT</td><td>match.equivalence</td><td>0..1</td><td><a href="datatypes.html#code">code</a></td><td/><td><div><p>A code indicating the equivalence of the translation, using values from <a href="valueset-concept-map-equivalence.html">ConceptMapEquivalence</a></p>\n</div></td></tr><tr><td>OUT</td><td>match.concept</td><td>0..1</td><td><a href="datatypes.html#Coding">Coding</a></td><td/><td><div><p>The translation outcome. Note that this would never have userSelected = true, since the process of translations implies that the user is not selecting the code (and only the client could know differently)</p>\n</div></td></tr><tr><td>OUT</td><td>match.product</td><td>0..*</td><td></td><td/><td><div><p>Another element that is the product of this mapping</p>\n</div></td></tr><tr><td>OUT</td><td>match.product.element</td><td>0..1</td><td><a href="datatypes.html#uri">uri</a></td><td/><td><div><p>The element for this product</p>\n</div></td></tr><tr><td>OUT</td><td>match.product.concept</td><td>0..1</td><td><a href="datatypes.html#Coding">Coding</a></td><td/><td><div><p>The value for this product</p>\n</div></td></tr><tr><td>OUT</td><td>match.source</td><td>0..1</td><td><a href="datatypes.html#uri">uri</a></td><td/><td><div><p>The canonical reference to the concept map from which this mapping comes from</p>\n</div></td></tr></table><div/></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 3,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/ConceptMap-translate",
    version: "4.0.1",
    name: "Concept Translation",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "Translate a code from one value set to another, based on the existing value set and concept maps resources, and/or other additional knowledge available to the server. \r\n\r\n One (and only one) of the in parameters (code, coding, codeableConcept) must be provided, to identify the code that is to be translated.  \r\n\r\n The operation returns a set of parameters including a 'result' for whether there is an acceptable match, and a list of possible matches. Note that the list of matches may include notes of codes for which mapping is specifically excluded, so implementers have to check the match.equivalence for each match",
    code: "translate",
    resource: ["ConceptMap"],
    system: false,
    type: true,
    instance: true,
    parameter: [
      {
        name: "url",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "A canonical URL for a concept map. The server must know the concept map (e.g. it is defined explicitly in the server's concept maps, or it is defined implicitly by some code system known to the server.",
        type: "uri",
      },
      {
        name: "conceptMap",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The concept map is provided directly as part of the request. Servers may choose not to accept concept maps in this fashion.",
        type: "ConceptMap",
      },
      {
        name: "conceptMapVersion",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The identifier that is used to identify a specific version of the concept map to be used for the translation. This is an arbitrary value managed by the concept map author and is not expected to be globally unique. For example, it might be a timestamp (e.g. yyyymmdd) if a managed version is not available.",
        type: "string",
      },
      {
        name: "code",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The code that is to be translated. If a code is provided, a system must be provided",
        type: "code",
      },
      {
        name: "system",
        use: "in",
        min: 0,
        max: "1",
        documentation: "The system for the code that is to be translated",
        type: "uri",
      },
      {
        name: "version",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The version of the system, if one was provided in the source data",
        type: "string",
      },
      {
        name: "source",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "Identifies the value set used when the concept (system/code pair) was chosen. May be a logical id, or an absolute or relative location. The source value set is an optional parameter because in some cases, the client cannot know what the source value set is. However, without a source value set, the server may be unable to safely identify an applicable concept map, and would return an error. For this reason, a source value set SHOULD always be provided. Note that servers may be able to identify an appropriate concept map without a source value set if there is a full mapping for the entire code system in the concept map, or by manual intervention",
        type: "uri",
      },
      {
        name: "coding",
        use: "in",
        min: 0,
        max: "1",
        documentation: "A coding to translate",
        type: "Coding",
      },
      {
        name: "codeableConcept",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "A full codeableConcept to validate. The server can translate any of the coding values (e.g. existing translations) as it chooses",
        type: "CodeableConcept",
      },
      {
        name: "target",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "Identifies the value set in which a translation is sought. May be a logical id, or an absolute or relative location. If there's no target specified, the server should return all known translations, along with their source",
        type: "uri",
      },
      {
        name: "targetsystem",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "identifies a target code system in which a mapping is sought. This parameter is an alternative to the target parameter - only one is required. Searching for any translation to a target code system irrespective of the context (e.g. target valueset) may lead to unsafe results, and it is at the discretion of the server to decide when to support this operation",
        type: "uri",
      },
      {
        name: "dependency",
        use: "in",
        min: 0,
        max: "*",
        documentation:
          "Another element that may help produce the correct mapping",
        part: [
          {
            name: "element",
            use: "in",
            min: 0,
            max: "1",
            documentation: "The element for this dependency",
            type: "uri",
          },
          {
            name: "concept",
            use: "in",
            min: 0,
            max: "1",
            documentation: "The value for this dependency",
            type: "CodeableConcept",
          },
        ],
      },
      {
        name: "reverse",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "if this is true, then the operation should return all the codes that might be mapped to this code. This parameter reverses the meaning of the source and target parameters",
        type: "boolean",
      },
      {
        name: "result",
        use: "out",
        min: 1,
        max: "1",
        documentation:
          "True if the concept could be translated successfully. The value can only be true if at least one returned match has an equivalence which is not  unmatched or disjoint",
        type: "boolean",
      },
      {
        name: "message",
        use: "out",
        min: 0,
        max: "1",
        documentation:
          "Error details, for display to a human. If this is provided when result = true, the message carries hints and warnings (e.g. a note that the matches could be improved by providing additional detail)",
        type: "string",
      },
      {
        name: "match",
        use: "out",
        min: 0,
        max: "*",
        documentation:
          "A concept in the target value set with an equivalence. Note that there may be multiple matches of equal or differing equivalence, and the matches may include equivalence values that mean that there is no match",
        part: [
          {
            name: "equivalence",
            use: "out",
            min: 0,
            max: "1",
            documentation:
              "A code indicating the equivalence of the translation, using values from [ConceptMapEquivalence](valueset-concept-map-equivalence.html)",
            type: "code",
          },
          {
            name: "concept",
            use: "out",
            min: 0,
            max: "1",
            documentation:
              "The translation outcome. Note that this would never have userSelected = true, since the process of translations implies that the user is not selecting the code (and only the client could know differently)",
            type: "Coding",
          },
          {
            name: "product",
            use: "out",
            min: 0,
            max: "*",
            documentation:
              "Another element that is the product of this mapping",
            part: [
              {
                name: "element",
                use: "out",
                min: 0,
                max: "1",
                documentation: "The element for this product",
                type: "uri",
              },
              {
                name: "concept",
                use: "out",
                min: 0,
                max: "1",
                documentation: "The value for this product",
                type: "Coding",
              },
            ],
          },
          {
            name: "source",
            use: "out",
            min: 0,
            max: "1",
            documentation:
              "The canonical reference to the concept map from which this mapping comes from",
            type: "uri",
          },
        ],
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace CoverageEligibilityRequestSubmit {
  export type Input = { resource: fhirTypes.Resource };
  export type Output = fhirTypes.Resource;
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "CoverageEligibilityRequest-submit",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Submit an EligibilityRequest resource for assessment</h2><p>OPERATION: Submit an EligibilityRequest resource for assessment</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/CoverageEligibilityRequest-submit</pre><div><p>This operation is used to submit an EligibilityRequest for assessment either as a single EligibilityRequest resource instance or as a Bundle containing the EligibilityRequest and other referenced resources, or Bundle containing a batch of EligibilityRequest resources, either as single EligibilityRequests resources or Bundle resources, for processing. The only input parameter is the single EligibilityRequest or Bundle resource and the only output is a single EligibilityResponse, Bundle of EligibilityResponses or an OperationOutcome resource.</p>\n</div><p>URL: [base]/CoverageEligibilityRequest/$submit</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>resource</td><td>1..1</td><td><a href="resource.html">Resource</a></td><td/><td><div><p>An EligibilityRequest resource or Bundle of EligibilityRequests, either as individual EligibilityRequest resources or as Bundles each containing a single EligibilityRequest plus referenced resources.</p>\n</div></td></tr><tr><td>OUT</td><td>return</td><td>1..1</td><td><a href="resource.html">Resource</a></td><td/><td><div><p>An EligibilityResponse resource or Bundle of EligibilityResponse responses, either as individual EligibilityResponse resources or as Bundles each containing a single EligibilityResponse plus referenced resources.</p>\n</div></td></tr></table><div/></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 2,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/CoverageEligibilityRequest-submit",
    version: "4.0.1",
    name: "Submit an EligibilityRequest resource for assessment",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "This operation is used to submit an EligibilityRequest for assessment either as a single EligibilityRequest resource instance or as a Bundle containing the EligibilityRequest and other referenced resources, or Bundle containing a batch of EligibilityRequest resources, either as single EligibilityRequests resources or Bundle resources, for processing. The only input parameter is the single EligibilityRequest or Bundle resource and the only output is a single EligibilityResponse, Bundle of EligibilityResponses or an OperationOutcome resource.",
    code: "submit",
    resource: ["CoverageEligibilityRequest"],
    system: false,
    type: true,
    instance: false,
    parameter: [
      {
        name: "resource",
        use: "in",
        min: 1,
        max: "1",
        documentation:
          "An EligibilityRequest resource or Bundle of EligibilityRequests, either as individual EligibilityRequest resources or as Bundles each containing a single EligibilityRequest plus referenced resources.",
        type: "Resource",
      },
      {
        name: "return",
        use: "out",
        min: 1,
        max: "1",
        documentation:
          "An EligibilityResponse resource or Bundle of EligibilityResponse responses, either as individual EligibilityResponse resources or as Bundles each containing a single EligibilityResponse plus referenced resources.",
        type: "Resource",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace EncounterEverything {
  export type Input = {
    _since?: fhirTypes.instant;
    _type?: Array<fhirTypes.code>;
    _count?: fhirTypes.integer;
  };
  export type Output = fhirTypes.Bundle;
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "Encounter-everything",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Fetch Encounter Record</h2><p>OPERATION: Fetch Encounter Record</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/Encounter-everything</pre><div><p>This operation is used to return all the information related to an encounter described in the resource on which this operation is invoked. The response is a bundle of type &quot;searchset&quot;. At a minimum, the encounter resource itself is returned, along with any other resources that the server has available for the given encounter for the user. The server also returns whatever resources are needed to support the records - e.g. linked practitioners, locations, organizations etc. The principle intended use for this operation is to provide a patient with access to their record, or to allow a client to retrieve everything for an encounter for efficient display).</p>\n<p>The server SHOULD return all resources it has that:</p>\n<ul>\n<li>are included in the encounter compartment for the identified encounter (have a reference to the encounter)</li>\n<li>are referenced by the standard extenstion for associating an encounter (where no reference element exists) http://hl7.org/fhir/StructureDefinition/encounter-associatedEncounter</li>\n<li>the server believes are relevant to the context of the encounter for any other reason (internally defined/decided)</li>\n<li>any resource referenced by the above, including binaries and attachments (to make a more complete package)</li>\n</ul>\n<p>In the US Realm, at a mimimum, the resources returned SHALL include all the data covered by the meaningful use common data elements (see <a href="http://hl7.org/fhir/us/daf">DAF</a> for further guidance). Other applicable implementation guides may make additional rules about the information that is returned.   Note that for many resources, the exact nature of the link to encounter can be ambiguous (e.g. for a DiagnosticReport, is it the encounter when it was initiated, or when it was reported?)</p>\n</div><p>URL: [base]/Encounter/[id]/$everything</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>_since</td><td>0..1</td><td><a href="datatypes.html#instant">instant</a></td><td/><td><div><p>Resources updated after this period will be included in the response. The intent of this parameter is to allow a client to request only records that have changed since the last request, based on either the return header time, or or (for asynchronous use), the transaction time</p>\n</div></td></tr><tr><td>IN</td><td>_type</td><td>0..*</td><td><a href="datatypes.html#code">code</a></td><td/><td><div><p>One or more parameters, each containing one or more comma-delimited FHIR resource types to include in the return resources. In the absense of any specified types, the server returns all resource types</p>\n</div></td></tr><tr><td>IN</td><td>_count</td><td>0..1</td><td><a href="datatypes.html#integer">integer</a></td><td/><td><div><p>See discussion below on the utility of paging through the results of the $everything operation</p>\n</div></td></tr><tr><td>OUT</td><td>return</td><td>1..1</td><td><a href="bundle.html">Bundle</a></td><td/><td><div><p>The bundle type is &quot;searchset&quot;</p>\n</div></td></tr></table><div><p>The key differences between this operation and simply searching the encounter compartment are:</p>\n<ul>\n<li>unless the client requests otherwise, the server returns the entire result set in a single bundle (rather than using paging)</li>\n<li>the server is responsible for determining what resources to return as included resources (rather than the client specifying which ones)</li>\n</ul>\n<p>This frees the client from needing to determine what it could or should ask for, particularly with regard to included resources. Servers should consider returning appropriate Provenance and AuditTrail on the returned resources, even though these are not directly part of the patient compartment.</p>\n<p>It is assumed that the server has identified and secured the context appropriately, and can either associate the authorization context with a single encounter, or determine whether the context has the rights to the nominated encounter, if there is one, or can determine an appropriate list of encouners to provide data for from the context of the request.  If there is no nominated encounter (GET /Encounter/$everything) and the context is not associated with a single encounter record, the actual list of encounters is all encounters that the user associated with the request has access to. In such cases, the server may choose to return an error rather than all the records. Specifying the relationship between the context, a user and encounter records is outside the scope of this specification (though see <a href="http://hl7.org/fhir/smart-app-launch">The Smart App Launch Implementation Guide</a>.</p>\n<p>When this operation is used to access multiple encounter records at once, the return bundle could be rather a lot of data; servers may choose to require that such requests are made <a href="async.html">asynchronously</a>, and associated with <a href="formats.html#bulk">bulk data formats</a>. Alternatively, clients may choose to page through the result set (or servers may require this). Paging through the results is done the same as for <a href="http.html#paging">Searching</a>, using the <a href="search.html#count">_count</a> parameter, and Bundle links. Implementers should note that paging will be slower than simply returning all the results at once (more network traffic, multiple latency delays) but may be required in order not to exhaust available memory reading or writing the whole response in a single package. Unlike searching, there is no inherent user-display order for the $everything operation. Servers might consider sorting the returned resources in descending order of last record update, but are not required to do so. Servers should consider returning appropriate Provenance and AuditTrail on the returned resources, even though these are not directly part of the patient compartment.</p>\n<p>The _since parameter is provided to support periodic queries to get additional information that has changed about the encounter since the last query. This means that the _since parameter is based on record time. The value of the _since parameter should be set to the time from the server. If using direct response, this is the timestamp in the response header. If using the async interface, this is the transaction timestamp in the json response. Servers should ensure that the timestamps a managed such that the client does not miss any changes. Clients should be able to handle getting the same response more than once in the case that the transaction falls on a time boundary. Clients should ensure that the other query parameters are constant to ensure a coherent set of records when doing periodic queries.</p>\n</div></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 2,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/Encounter-everything",
    version: "4.0.1",
    name: "Fetch Encounter Record",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      'This operation is used to return all the information related to an encounter described in the resource on which this operation is invoked. The response is a bundle of type "searchset". At a minimum, the encounter resource itself is returned, along with any other resources that the server has available for the given encounter for the user. The server also returns whatever resources are needed to support the records - e.g. linked practitioners, locations, organizations etc. The principle intended use for this operation is to provide a patient with access to their record, or to allow a client to retrieve everything for an encounter for efficient display).\r\rThe server SHOULD return all resources it has that:\r\r* are included in the encounter compartment for the identified encounter (have a reference to the encounter)\r* are referenced by the standard extenstion for associating an encounter (where no reference element exists) http://hl7.org/fhir/StructureDefinition/encounter-associatedEncounter\r* the server believes are relevant to the context of the encounter for any other reason (internally defined/decided)\r* any resource referenced by the above, including binaries and attachments (to make a more complete package)\r\rIn the US Realm, at a mimimum, the resources returned SHALL include all the data covered by the meaningful use common data elements (see [DAF](http://hl7.org/fhir/us/daf) for further guidance). Other applicable implementation guides may make additional rules about the information that is returned.   Note that for many resources, the exact nature of the link to encounter can be ambiguous (e.g. for a DiagnosticReport, is it the encounter when it was initiated, or when it was reported?)',
    code: "everything",
    comment:
      "The key differences between this operation and simply searching the encounter compartment are:  \n\n* unless the client requests otherwise, the server returns the entire result set in a single bundle (rather than using paging) \n* the server is responsible for determining what resources to return as included resources (rather than the client specifying which ones)\n\nThis frees the client from needing to determine what it could or should ask for, particularly with regard to included resources. Servers should consider returning appropriate Provenance and AuditTrail on the returned resources, even though these are not directly part of the patient compartment. \n\nIt is assumed that the server has identified and secured the context appropriately, and can either associate the authorization context with a single encounter, or determine whether the context has the rights to the nominated encounter, if there is one, or can determine an appropriate list of encouners to provide data for from the context of the request.  If there is no nominated encounter (GET /Encounter/$everything) and the context is not associated with a single encounter record, the actual list of encounters is all encounters that the user associated with the request has access to. In such cases, the server may choose to return an error rather than all the records. Specifying the relationship between the context, a user and encounter records is outside the scope of this specification (though see [The Smart App Launch Implementation Guide](http://hl7.org/fhir/smart-app-launch).   \n\nWhen this operation is used to access multiple encounter records at once, the return bundle could be rather a lot of data; servers may choose to require that such requests are made [asynchronously](async.html), and associated with [bulk data formats](formats.html#bulk). Alternatively, clients may choose to page through the result set (or servers may require this). Paging through the results is done the same as for [Searching](http.html#paging), using the [_count](search.html#count) parameter, and Bundle links. Implementers should note that paging will be slower than simply returning all the results at once (more network traffic, multiple latency delays) but may be required in order not to exhaust available memory reading or writing the whole response in a single package. Unlike searching, there is no inherent user-display order for the $everything operation. Servers might consider sorting the returned resources in descending order of last record update, but are not required to do so. Servers should consider returning appropriate Provenance and AuditTrail on the returned resources, even though these are not directly part of the patient compartment.\n\nThe _since parameter is provided to support periodic queries to get additional information that has changed about the encounter since the last query. This means that the _since parameter is based on record time. The value of the _since parameter should be set to the time from the server. If using direct response, this is the timestamp in the response header. If using the async interface, this is the transaction timestamp in the json response. Servers should ensure that the timestamps a managed such that the client does not miss any changes. Clients should be able to handle getting the same response more than once in the case that the transaction falls on a time boundary. Clients should ensure that the other query parameters are constant to ensure a coherent set of records when doing periodic queries.",
    resource: ["Encounter"],
    system: false,
    type: false,
    instance: true,
    parameter: [
      {
        name: "_since",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "Resources updated after this period will be included in the response. The intent of this parameter is to allow a client to request only records that have changed since the last request, based on either the return header time, or or (for asynchronous use), the transaction time",
        type: "instant",
      },
      {
        name: "_type",
        use: "in",
        min: 0,
        max: "*",
        documentation:
          "One or more parameters, each containing one or more comma-delimited FHIR resource types to include in the return resources. In the absense of any specified types, the server returns all resource types",
        type: "code",
      },
      {
        name: "_count",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "See discussion below on the utility of paging through the results of the $everything operation",
        type: "integer",
      },
      {
        name: "return",
        use: "out",
        min: 1,
        max: "1",
        documentation: 'The bundle type is "searchset"',
        type: "Bundle",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace GroupEverything {
  export type Input = {
    start?: fhirTypes.date;
    end?: fhirTypes.date;
    _since?: fhirTypes.instant;
    _type?: Array<fhirTypes.code>;
    _count?: fhirTypes.integer;
  };
  export type Output = fhirTypes.Bundle;
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "Group-everything",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Fetch a group of Patient Records</h2><p>OPERATION: Fetch a group of Patient Records</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/Group-everything</pre><div><p>This operation is used to return all the information related to one or more patients that are part of the group on which this operation is invoked. The response is a bundle of type &quot;searchset&quot;. At a minimum, the patient resource(s) itself is returned, along with any other resources that the server has that are related to the patient(s), and that are available for the given user. The server also returns whatever resources are needed to support the records - e.g. linked practitioners, medications, locations, organizations etc.   The intended use for this operation is for a provider or other user to perform a bulk data download.  The server SHOULD return at least all resources that it has that are in the patient compartment for the identified patient(s), and any resource referenced from those, including binaries and attachments. In the US Realm, at a mimimum, the resources returned SHALL include all the data covered by the meaningful use common data elements as defined in <a href="http://hl7.org/fhir/us/coref">US-Core</a>. Other applicable implementation guides may make additional rules about how much information that is returned.</p>\n</div><p>URL: [base]/Group/[id]/$everything</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>start</td><td>0..1</td><td><a href="datatypes.html#date">date</a></td><td/><td><div><p>The date range relates to care dates, not record currency dates - e.g. all records relating to care provided in a certain date range. If no start date is provided, all records prior to the end date are in scope.</p>\n</div></td></tr><tr><td>IN</td><td>end</td><td>0..1</td><td><a href="datatypes.html#date">date</a></td><td/><td><div><p>The date range relates to care dates, not record currency dates - e.g. all records relating to care provided in a certain date range. If no end date is provided, all records subsequent to the start date are in scope.</p>\n</div></td></tr><tr><td>IN</td><td>_since</td><td>0..1</td><td><a href="datatypes.html#instant">instant</a></td><td/><td><div><p>Resources updated after this period will be included in the response. The intent of this parameter is to allow a client to request only records that have changed since the last request, based on either the return header time, or or (for asynchronous use), the transaction time</p>\n</div></td></tr><tr><td>IN</td><td>_type</td><td>0..*</td><td><a href="datatypes.html#code">code</a></td><td/><td><div><p>One or more parameters, each containing one or more comma-delimited FHIR resource types to include in the return resources. In the absense of any specified types, the server returns all resource types</p>\n</div></td></tr><tr><td>IN</td><td>_count</td><td>0..1</td><td><a href="datatypes.html#integer">integer</a></td><td/><td><div><p>See discussion below on the utility of paging through the results of the $everything operation</p>\n</div></td></tr><tr><td>OUT</td><td>return</td><td>1..1</td><td><a href="bundle.html">Bundle</a></td><td/><td><div><p>The bundle type is &quot;searchset&quot;</p>\n</div></td></tr></table><div><p>The key differences between this operation and simply searching the group\'s patients compartment are:</p>\n<ul>\n<li>unless the client requests otherwise, the server returns the entire result set in a single bundle (rather than using paging)</li>\n<li>the server is responsible for determining what resources to return as included resources (rather than the client specifying which ones).</li>\n</ul>\n<p>This frees the client from needing to determine what it could or should ask for, particularly with regard to included resources.</p>\n<p>It is assumed that the server has identified and secured the context appropriately, and can either associate the authorization context with a particular group, or determine whether the context has the rights to the nominated group, if there is one, or can determine an appropriate list of groups to provide data for from the context of the request.   If there is no nominated group (GET /Group/$everything) and the context is not associated with a single group record, the actual list of groups is all groups that the user associated with the request has access to. In such cases, the server may choose to return an error rather than all the records (and is likely to do so, but not required to).  Specifying the relationship between the context, a user and groups is outside the scope of this specification (though see <a href="http://hl7.org/fhir/smart-app-launch">The Smart App Launch Implementation Guide</a>.</p>\n<p>The return bundle from this operation is usually rather a lot of data; servers typically choose to require that such requests are made <a href="async.html">asynchronously</a>, and associated with <a href="formats.html#bulk">bulk data formats</a>. Alternatively, clients may choose to page through the result set (or servers may require this). Paging through the results is done the same as for <a href="http.html#paging">Searching</a>, using the <a href="search.html#count">_count</a> parameter, and Bundle links. Implementers should note that paging will be slower than simply returning all the results at once (more network traffic, multiple latency delays) but may be required in order not to exhaust available memory reading or writing the whole response in a single package. Unlike searching, there is no inherent user-display order for the $everything operation. Servers might consider sorting the returned resources in descending order of last record update, but are not required to do so.</p>\n<p>The _since parameter is provided to support periodic queries to get additional information that has changed about the group since the last query. This means that the _since parameter is based on record time. The value of the _since parameter should be set to the time from the server. If using direct response, this is the timestamp in the response header. If using the async interface, this is the transaction timestamp in the json response. Servers should ensure that the timestamps a managed such that the client does not miss any changes. Clients should be able to handle getting the same response more than once in the case that the transaction falls on a time boundary. Clients should ensure that the other query parameters are constant to ensure a coherent set of records when doing periodic queries.</p>\n</div></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 1,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/Group-everything",
    version: "4.0.1",
    name: "Fetch a group of Patient Records",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      'This operation is used to return all the information related to one or more patients that are part of the group on which this operation is invoked. The response is a bundle of type "searchset". At a minimum, the patient resource(s) itself is returned, along with any other resources that the server has that are related to the patient(s), and that are available for the given user. The server also returns whatever resources are needed to support the records - e.g. linked practitioners, medications, locations, organizations etc.   The intended use for this operation is for a provider or other user to perform a bulk data download.  The server SHOULD return at least all resources that it has that are in the patient compartment for the identified patient(s), and any resource referenced from those, including binaries and attachments. In the US Realm, at a mimimum, the resources returned SHALL include all the data covered by the meaningful use common data elements as defined in [US-Core](http://hl7.org/fhir/us/coref). Other applicable implementation guides may make additional rules about how much information that is returned.',
    code: "everything",
    comment:
      "The key differences between this operation and simply searching the group's patients compartment are:    \n\n* unless the client requests otherwise, the server returns the entire result set in a single bundle (rather than using paging)  \n* the server is responsible for determining what resources to return as included resources (rather than the client specifying which ones). \n\nThis frees the client from needing to determine what it could or should ask for, particularly with regard to included resources.  \n\nIt is assumed that the server has identified and secured the context appropriately, and can either associate the authorization context with a particular group, or determine whether the context has the rights to the nominated group, if there is one, or can determine an appropriate list of groups to provide data for from the context of the request.   If there is no nominated group (GET /Group/$everything) and the context is not associated with a single group record, the actual list of groups is all groups that the user associated with the request has access to. In such cases, the server may choose to return an error rather than all the records (and is likely to do so, but not required to).  Specifying the relationship between the context, a user and groups is outside the scope of this specification (though see [The Smart App Launch Implementation Guide](http://hl7.org/fhir/smart-app-launch). \n\nThe return bundle from this operation is usually rather a lot of data; servers typically choose to require that such requests are made [asynchronously](async.html), and associated with [bulk data formats](formats.html#bulk). Alternatively, clients may choose to page through the result set (or servers may require this). Paging through the results is done the same as for [Searching](http.html#paging), using the [_count](search.html#count) parameter, and Bundle links. Implementers should note that paging will be slower than simply returning all the results at once (more network traffic, multiple latency delays) but may be required in order not to exhaust available memory reading or writing the whole response in a single package. Unlike searching, there is no inherent user-display order for the $everything operation. Servers might consider sorting the returned resources in descending order of last record update, but are not required to do so.\n\nThe _since parameter is provided to support periodic queries to get additional information that has changed about the group since the last query. This means that the _since parameter is based on record time. The value of the _since parameter should be set to the time from the server. If using direct response, this is the timestamp in the response header. If using the async interface, this is the transaction timestamp in the json response. Servers should ensure that the timestamps a managed such that the client does not miss any changes. Clients should be able to handle getting the same response more than once in the case that the transaction falls on a time boundary. Clients should ensure that the other query parameters are constant to ensure a coherent set of records when doing periodic queries.",
    resource: ["Group"],
    system: false,
    type: false,
    instance: true,
    parameter: [
      {
        name: "start",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The date range relates to care dates, not record currency dates - e.g. all records relating to care provided in a certain date range. If no start date is provided, all records prior to the end date are in scope.",
        type: "date",
      },
      {
        name: "end",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The date range relates to care dates, not record currency dates - e.g. all records relating to care provided in a certain date range. If no end date is provided, all records subsequent to the start date are in scope.",
        type: "date",
      },
      {
        name: "_since",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "Resources updated after this period will be included in the response. The intent of this parameter is to allow a client to request only records that have changed since the last request, based on either the return header time, or or (for asynchronous use), the transaction time",
        type: "instant",
      },
      {
        name: "_type",
        use: "in",
        min: 0,
        max: "*",
        documentation:
          "One or more parameters, each containing one or more comma-delimited FHIR resource types to include in the return resources. In the absense of any specified types, the server returns all resource types",
        type: "code",
      },
      {
        name: "_count",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "See discussion below on the utility of paging through the results of the $everything operation",
        type: "integer",
      },
      {
        name: "return",
        use: "out",
        min: 1,
        max: "1",
        documentation: 'The bundle type is "searchset"',
        type: "Bundle",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace LibraryDataRequirements {
  export type Input = { target?: fhirTypes.string };
  export type Output = fhirTypes.Library;
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "Library-data-requirements",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Data Requirements</h2><p>OPERATION: Data Requirements</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/Library-data-requirements</pre><div><p>The data-requirements operation aggregates and returns the parameters and data requirements for a resource and all its dependencies as a single module definition</p>\n</div><p>URL: [base]/$data-requirements</p><p>URL: [base]/Library/[id]/$data-requirements</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>target</td><td>0..1</td><td><a href="datatypes.html#string">string</a></td><td/><td><div><p>The target of the data requirements operation</p>\n</div></td></tr><tr><td>OUT</td><td>return</td><td>1..1</td><td><a href="library.html">Library</a></td><td/><td><div><p>The result of the requirements gathering</p>\n</div></td></tr></table><div><p>The effect of invoking this operation is to determine the aggregate set of data requirements and dependencies for a given target resource. The result is a Library resource with a type of module-definition that contains all the parameter definitions and data requirements of the target resource and any libraries referenced by it. Implementations SHOULD aggregate data requirements intelligently (i.e. by collapsing overlapping data requirements)</p>\n</div></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 2,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/Library-data-requirements",
    version: "4.0.1",
    name: "Data Requirements",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "The data-requirements operation aggregates and returns the parameters and data requirements for a resource and all its dependencies as a single module definition",
    code: "data-requirements",
    comment:
      "The effect of invoking this operation is to determine the aggregate set of data requirements and dependencies for a given target resource. The result is a Library resource with a type of module-definition that contains all the parameter definitions and data requirements of the target resource and any libraries referenced by it. Implementations SHOULD aggregate data requirements intelligently (i.e. by collapsing overlapping data requirements)",
    resource: ["Library"],
    system: true,
    type: false,
    instance: true,
    parameter: [
      {
        name: "target",
        use: "in",
        min: 0,
        max: "1",
        documentation: "The target of the data requirements operation",
        type: "string",
      },
      {
        name: "return",
        use: "out",
        min: 1,
        max: "1",
        documentation: "The result of the requirements gathering",
        type: "Library",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace ListFind {
  export type Input = { patient: fhirTypes.id; name: fhirTypes.code };
  export type Output = Record<string, never>;
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "List-find",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Find a functional list</h2><p>OPERATION: Find a functional list</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/List-find</pre><div><p>This operation allows a client to find an identified list for a particular function by its function. The operation takes two parameters, the identity of a patient, and the name of a functional list.     The list of defined functional lists can be found at <a href="lifecycle.html#lists">Current Resource Lists</a>. Applications are not required to support all the lists, and may define additional lists of their own.   If the system is able to locate a list that serves the identified purpose, it returns it as the body of the response with a 200 OK status. If the resource cannot be located, the server returns a 404 not found (optionally with an OperationOutcome resource)</p>\n</div><p>URL: [base]/List/$find</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>patient</td><td>1..1</td><td><a href="datatypes.html#id">id</a></td><td/><td><div><p>The id of a patient resource located on the server on which this operation is executed</p>\n</div></td></tr><tr><td>IN</td><td>name</td><td>1..1</td><td><a href="datatypes.html#code">code</a></td><td/><td><div><p>The code for the functional list that is being found</p>\n</div></td></tr></table><div><p>Note that servers may support searching by a functional list, and not support this operation that allows clients to find the list directly</p>\n</div></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 1,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/List-find",
    version: "4.0.1",
    name: "Find a functional list",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "This operation allows a client to find an identified list for a particular function by its function. The operation takes two parameters, the identity of a patient, and the name of a functional list.     The list of defined functional lists can be found at [Current Resource Lists](lifecycle.html#lists). Applications are not required to support all the lists, and may define additional lists of their own.   If the system is able to locate a list that serves the identified purpose, it returns it as the body of the response with a 200 OK status. If the resource cannot be located, the server returns a 404 not found (optionally with an OperationOutcome resource)",
    code: "find",
    comment:
      "Note that servers may support searching by a functional list, and not support this operation that allows clients to find the list directly",
    resource: ["List"],
    system: false,
    type: true,
    instance: false,
    parameter: [
      {
        name: "patient",
        use: "in",
        min: 1,
        max: "1",
        documentation:
          "The id of a patient resource located on the server on which this operation is executed",
        type: "id",
      },
      {
        name: "name",
        use: "in",
        min: 1,
        max: "1",
        documentation: "The code for the functional list that is being found",
        type: "code",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace MeasureCareGaps {
  export type Input = {
    periodStart: fhirTypes.date;
    periodEnd: fhirTypes.date;
    topic: fhirTypes.string;
    subject: fhirTypes.string;
  };
  export type Output = fhirTypes.Bundle;
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "Measure-care-gaps",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Care Gaps</h2><p>OPERATION: Care Gaps</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/Measure-care-gaps</pre><div><p>The care-gaps operation is used to determine gaps-in-care based on the results of quality measures</p>\n</div><p>URL: [base]/Measure/$care-gaps</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>periodStart</td><td>1..1</td><td><a href="datatypes.html#date">date</a></td><td/><td><div><p>The start of the measurement period. In keeping with the semantics of the date parameter used in the FHIR search operation, the period will start at the beginning of the period implied by the supplied timestamp. E.g. a value of 2014 would set the period s</p>\n</div></td></tr><tr><td>IN</td><td>periodEnd</td><td>1..1</td><td><a href="datatypes.html#date">date</a></td><td/><td><div><p>The end of the measurement period. The period will end at the end of the period implied by the supplied timestamp. E.g. a value of 2014 would set the period end to be 2014-12-31T23:59:59 inclusive</p>\n</div></td></tr><tr><td>IN</td><td>topic</td><td>1..1</td><td><a href="datatypes.html#string">string</a></td><td/><td><div><p>The topic to be used to determine which measures are considered for the care gaps report. Any measure with the given topic will be included in the report</p>\n</div></td></tr><tr><td>IN</td><td>subject</td><td>1..1</td><td><a href="datatypes.html#string">string</a><br/>(<a href="search.html#reference">reference</a>)</td><td/><td><div><p>Subject for which the care gaps report will be produced</p>\n</div></td></tr><tr><td>OUT</td><td>return</td><td>1..1</td><td><a href="bundle.html">Bundle</a></td><td/><td><div><p>The result of the care gaps report will be returned as a document bundle with a MeasureReport entry for each included measure</p>\n</div></td></tr></table><div><p>The effect of invoking this operation is to calculate a set of measures for a particular topic (e.g. Preventive Care and Screening) and return a document describing the results of each measure for the given subject. Note that it is up to the server to determine whether or not the generated care gaps report is persisted. If the server does not persist the results, the operation does not affect state and can be invoked with a GET</p>\n</div></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 2,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/Measure-care-gaps",
    version: "4.0.1",
    name: "Care Gaps",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "The care-gaps operation is used to determine gaps-in-care based on the results of quality measures",
    code: "care-gaps",
    comment:
      "The effect of invoking this operation is to calculate a set of measures for a particular topic (e.g. Preventive Care and Screening) and return a document describing the results of each measure for the given subject. Note that it is up to the server to determine whether or not the generated care gaps report is persisted. If the server does not persist the results, the operation does not affect state and can be invoked with a GET",
    resource: ["Measure"],
    system: false,
    type: true,
    instance: false,
    parameter: [
      {
        name: "periodStart",
        use: "in",
        min: 1,
        max: "1",
        documentation:
          "The start of the measurement period. In keeping with the semantics of the date parameter used in the FHIR search operation, the period will start at the beginning of the period implied by the supplied timestamp. E.g. a value of 2014 would set the period s",
        type: "date",
      },
      {
        name: "periodEnd",
        use: "in",
        min: 1,
        max: "1",
        documentation:
          "The end of the measurement period. The period will end at the end of the period implied by the supplied timestamp. E.g. a value of 2014 would set the period end to be 2014-12-31T23:59:59 inclusive",
        type: "date",
      },
      {
        name: "topic",
        use: "in",
        min: 1,
        max: "1",
        documentation:
          "The topic to be used to determine which measures are considered for the care gaps report. Any measure with the given topic will be included in the report",
        type: "string",
      },
      {
        name: "subject",
        use: "in",
        min: 1,
        max: "1",
        documentation:
          "Subject for which the care gaps report will be produced",
        type: "string",
        searchType: "reference",
      },
      {
        name: "return",
        use: "out",
        min: 1,
        max: "1",
        documentation:
          "The result of the care gaps report will be returned as a document bundle with a MeasureReport entry for each included measure",
        type: "Bundle",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace MeasureCollectData {
  export type Input = {
    periodStart: fhirTypes.date;
    periodEnd: fhirTypes.date;
    measure?: fhirTypes.string;
    subject?: fhirTypes.string;
    practitioner?: fhirTypes.string;
    lastReceivedOn?: fhirTypes.dateTime;
  };
  export type Output = {
    measureReport: fhirTypes.MeasureReport;
    resource?: Array<fhirTypes.Resource>;
  };
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "Measure-collect-data",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Collect Data</h2><p>OPERATION: Collect Data</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/Measure-collect-data</pre><div><p>The collect-data operation is used to collect the data-of-interest for the given measure.</p>\n</div><p>URL: [base]/Measure/$collect-data</p><p>URL: [base]/Measure/[id]/$collect-data</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>periodStart</td><td>1..1</td><td><a href="datatypes.html#date">date</a></td><td/><td><div><p>The start of the measurement period. In keeping with the semantics of the date parameter used in the FHIR search operation, the period will start at the beginning of the period implied by the supplied timestamp. E.g. a value of 2014 would set the period s</p>\n</div></td></tr><tr><td>IN</td><td>periodEnd</td><td>1..1</td><td><a href="datatypes.html#date">date</a></td><td/><td><div><p>The end of the measurement period. The period will end at the end of the period implied by the supplied timestamp. E.g. a value of 2014 would set the period end to be 2014-12-31T23:59:59 inclusive</p>\n</div></td></tr><tr><td>IN</td><td>measure</td><td>0..1</td><td><a href="datatypes.html#string">string</a><br/>(<a href="search.html#reference">reference</a>)</td><td/><td><div><p>The measure to evaluate. This parameter is only required when the operation is invoked on the resource type, it is not used when invoking the operation on a Measure instance</p>\n</div></td></tr><tr><td>IN</td><td>subject</td><td>0..1</td><td><a href="datatypes.html#string">string</a><br/>(<a href="search.html#reference">reference</a>)</td><td/><td><div><p>Subject for which the measure will be collected. If not specified, measure data will be collected for all subjects that meet the requirements of the measure. If specified, the measure will only be calculated for the referenced subject(s)</p>\n</div></td></tr><tr><td>IN</td><td>practitioner</td><td>0..1</td><td><a href="datatypes.html#string">string</a><br/>(<a href="search.html#reference">reference</a>)</td><td/><td><div><p>Practitioner for which the measure will be collected. If specified, measure data will be collected only for subjects that have a primary relationship to the identified practitioner</p>\n</div></td></tr><tr><td>IN</td><td>lastReceivedOn</td><td>0..1</td><td><a href="datatypes.html#dateTime">dateTime</a></td><td/><td><div><p>The date the results of this measure were last received. This parameter used to indicate when the last time data for this measure was collected. This information is used to support incremental data collection scenarios</p>\n</div></td></tr><tr><td>OUT</td><td>measureReport</td><td>1..1</td><td><a href="measurereport.html">MeasureReport</a></td><td/><td><div><p>A MeasureReport of type data-collection detailing the results of the operation</p>\n</div></td></tr><tr><td>OUT</td><td>resource</td><td>0..*</td><td><a href="resource.html">Resource</a></td><td/><td><div><p>The result resources that make up the data-of-interest for the measure</p>\n</div></td></tr></table><div><p>The effect of invoking this operation is to gather the data required to perform an evaluation of the measure. If the lastReceivedOn parameter is supplied, only data that is new or has been changed since the lastReceivedOn date is included in the response. Note that the resulting MeasureReport is a transient resource</p>\n</div></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 2,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/Measure-collect-data",
    version: "4.0.1",
    name: "Collect Data",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "The collect-data operation is used to collect the data-of-interest for the given measure.",
    code: "collect-data",
    comment:
      "The effect of invoking this operation is to gather the data required to perform an evaluation of the measure. If the lastReceivedOn parameter is supplied, only data that is new or has been changed since the lastReceivedOn date is included in the response. Note that the resulting MeasureReport is a transient resource",
    resource: ["Measure"],
    system: false,
    type: true,
    instance: true,
    parameter: [
      {
        name: "periodStart",
        use: "in",
        min: 1,
        max: "1",
        documentation:
          "The start of the measurement period. In keeping with the semantics of the date parameter used in the FHIR search operation, the period will start at the beginning of the period implied by the supplied timestamp. E.g. a value of 2014 would set the period s",
        type: "date",
      },
      {
        name: "periodEnd",
        use: "in",
        min: 1,
        max: "1",
        documentation:
          "The end of the measurement period. The period will end at the end of the period implied by the supplied timestamp. E.g. a value of 2014 would set the period end to be 2014-12-31T23:59:59 inclusive",
        type: "date",
      },
      {
        name: "measure",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The measure to evaluate. This parameter is only required when the operation is invoked on the resource type, it is not used when invoking the operation on a Measure instance",
        type: "string",
        searchType: "reference",
      },
      {
        name: "subject",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "Subject for which the measure will be collected. If not specified, measure data will be collected for all subjects that meet the requirements of the measure. If specified, the measure will only be calculated for the referenced subject(s)",
        type: "string",
        searchType: "reference",
      },
      {
        name: "practitioner",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "Practitioner for which the measure will be collected. If specified, measure data will be collected only for subjects that have a primary relationship to the identified practitioner",
        type: "string",
        searchType: "reference",
      },
      {
        name: "lastReceivedOn",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The date the results of this measure were last received. This parameter used to indicate when the last time data for this measure was collected. This information is used to support incremental data collection scenarios",
        type: "dateTime",
      },
      {
        name: "measureReport",
        use: "out",
        min: 1,
        max: "1",
        documentation:
          "A MeasureReport of type data-collection detailing the results of the operation",
        type: "MeasureReport",
      },
      {
        name: "resource",
        use: "out",
        min: 0,
        max: "*",
        documentation:
          "The result resources that make up the data-of-interest for the measure",
        type: "Resource",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace MeasureDataRequirements {
  export type Input = {
    periodStart: fhirTypes.date;
    periodEnd: fhirTypes.date;
  };
  export type Output = fhirTypes.Library;
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "Measure-data-requirements",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Data Requirements</h2><p>OPERATION: Data Requirements</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/Measure-data-requirements</pre><div><p>The data-requirements operation aggregates and returns the parameters and data requirements for the measure and all its dependencies as a single module definition</p>\n</div><p>URL: [base]/Measure/[id]/$data-requirements</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>periodStart</td><td>1..1</td><td><a href="datatypes.html#date">date</a></td><td/><td><div><p>The start of the measurement period. In keeping with the semantics of the date parameter used in the FHIR search operation, the period will start at the beginning of the period implied by the supplied timestamp. E.g. a value of 2014 would set the period start to be 2014-01-01T00:00:00 inclusive</p>\n</div></td></tr><tr><td>IN</td><td>periodEnd</td><td>1..1</td><td><a href="datatypes.html#date">date</a></td><td/><td><div><p>The end of the measurement period. The period will end at the end of the period implied by the supplied timestamp. E.g. a value of 2014 would set the period end to be 2014-12-31T23:59:59 inclusive</p>\n</div></td></tr><tr><td>OUT</td><td>return</td><td>1..1</td><td><a href="library.html">Library</a></td><td/><td><div><p>The result of the requirements gathering is a module-definition Library that describes the aggregate parameters, data requirements, and dependencies of the measure</p>\n</div></td></tr></table><div><p>The effect of invoking this operation is to determine the aggregate set of data requirements and dependencies for the measure. The result is a Library resource with a type of module-definition that contains all the parameter definitions and data requirements of the libraries referenced by the measures. Implementations SHOULD aggregate data requirements intelligently (i.e. by collapsing overlapping data requirements). This operation defines what resources are subsequently referenced in the evaluatedResources element of the MeasureReport when submitting measure data</p>\n</div></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 2,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/Measure-data-requirements",
    version: "4.0.1",
    name: "Data Requirements",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "The data-requirements operation aggregates and returns the parameters and data requirements for the measure and all its dependencies as a single module definition",
    code: "data-requirements",
    comment:
      "The effect of invoking this operation is to determine the aggregate set of data requirements and dependencies for the measure. The result is a Library resource with a type of module-definition that contains all the parameter definitions and data requirements of the libraries referenced by the measures. Implementations SHOULD aggregate data requirements intelligently (i.e. by collapsing overlapping data requirements). This operation defines what resources are subsequently referenced in the evaluatedResources element of the MeasureReport when submitting measure data",
    resource: ["Measure"],
    system: false,
    type: false,
    instance: true,
    parameter: [
      {
        name: "periodStart",
        use: "in",
        min: 1,
        max: "1",
        documentation:
          "The start of the measurement period. In keeping with the semantics of the date parameter used in the FHIR search operation, the period will start at the beginning of the period implied by the supplied timestamp. E.g. a value of 2014 would set the period start to be 2014-01-01T00:00:00 inclusive",
        type: "date",
      },
      {
        name: "periodEnd",
        use: "in",
        min: 1,
        max: "1",
        documentation:
          "The end of the measurement period. The period will end at the end of the period implied by the supplied timestamp. E.g. a value of 2014 would set the period end to be 2014-12-31T23:59:59 inclusive",
        type: "date",
      },
      {
        name: "return",
        use: "out",
        min: 1,
        max: "1",
        documentation:
          "The result of the requirements gathering is a module-definition Library that describes the aggregate parameters, data requirements, and dependencies of the measure",
        type: "Library",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace MeasureEvaluateMeasure {
  export type Input = {
    periodStart: fhirTypes.date;
    periodEnd: fhirTypes.date;
    measure?: fhirTypes.string;
    reportType?: fhirTypes.code;
    subject?: fhirTypes.string;
    practitioner?: fhirTypes.string;
    lastReceivedOn?: fhirTypes.dateTime;
  };
  export type Output = fhirTypes.MeasureReport;
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "Measure-evaluate-measure",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Evaluate Measure</h2><p>OPERATION: Evaluate Measure</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/Measure-evaluate-measure</pre><div><p>The evaluate-measure operation is used to calculate an eMeasure and obtain the results</p>\n</div><p>URL: [base]/Measure/$evaluate-measure</p><p>URL: [base]/Measure/[id]/$evaluate-measure</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>periodStart</td><td>1..1</td><td><a href="datatypes.html#date">date</a></td><td/><td><div><p>The start of the measurement period. In keeping with the semantics of the date parameter used in the FHIR search operation, the period will start at the beginning of the period implied by the supplied timestamp. E.g. a value of 2014 would set the period start to be 2014-01-01T00:00:00 inclusive</p>\n</div></td></tr><tr><td>IN</td><td>periodEnd</td><td>1..1</td><td><a href="datatypes.html#date">date</a></td><td/><td><div><p>The end of the measurement period. The period will end at the end of the period implied by the supplied timestamp. E.g. a value of 2014 would set the period end to be 2014-12-31T23:59:59 inclusive</p>\n</div></td></tr><tr><td>IN</td><td>measure</td><td>0..1</td><td><a href="datatypes.html#string">string</a><br/>(<a href="search.html#reference">reference</a>)</td><td/><td><div><p>The measure to evaluate. This parameter is only required when the operation is invoked on the resource type, it is not used when invoking the operation on a Measure instance</p>\n</div></td></tr><tr><td>IN</td><td>reportType</td><td>0..1</td><td><a href="datatypes.html#code">code</a></td><td/><td><div><p>The type of measure report: subject, subject-list, or population. If not specified, a default value of subject will be used if the subject parameter is supplied, otherwise, population will be used</p>\n</div></td></tr><tr><td>IN</td><td>subject</td><td>0..1</td><td><a href="datatypes.html#string">string</a><br/>(<a href="search.html#reference">reference</a>)</td><td/><td><div><p>Subject for which the measure will be calculated. If not specified, the measure will be calculated for all subjects that meet the requirements of the measure. If specified, the measure will only be calculated for the referenced subject(s)</p>\n</div></td></tr><tr><td>IN</td><td>practitioner</td><td>0..1</td><td><a href="datatypes.html#string">string</a><br/>(<a href="search.html#reference">reference</a>)</td><td/><td><div><p>Practitioner for which the measure will be calculated. If specified, the measure will be calculated only for subjects that have a primary relationship to the identified practitioner</p>\n</div></td></tr><tr><td>IN</td><td>lastReceivedOn</td><td>0..1</td><td><a href="datatypes.html#dateTime">dateTime</a></td><td/><td><div><p>The date the results of this measure were last received. This parameter is only valid for patient-level reports and is used to indicate when the last time a result for this patient was received. This information can be used to limit the set of resources returned for a patient-level report</p>\n</div></td></tr><tr><td>OUT</td><td>return</td><td>1..1</td><td><a href="measurereport.html">MeasureReport</a></td><td/><td><div><p>The results of the measure calculation. See the MeasureReport resource for a complete description of the output of this operation. Note that implementations may choose to return a MeasureReport with a status of pending to indicate that the report is still being generated. In this case, the client can use a polling method to continually request the MeasureReport until the status is updated to complete</p>\n</div></td></tr></table><div><p>The effect of invoking this operation is to calculate the measure for the given subject, or all subjects if no subject is supplied, and return the results as a MeasureReport resource of the appropriate type. Note that whether or not this operation affects the state of the server depends on whether the server persists the generated MeasureReport. If the MeasureReport is not persisted, this operation can be invoked with GET</p>\n</div></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 2,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/Measure-evaluate-measure",
    version: "4.0.1",
    name: "Evaluate Measure",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "The evaluate-measure operation is used to calculate an eMeasure and obtain the results",
    code: "evaluate-measure",
    comment:
      "The effect of invoking this operation is to calculate the measure for the given subject, or all subjects if no subject is supplied, and return the results as a MeasureReport resource of the appropriate type. Note that whether or not this operation affects the state of the server depends on whether the server persists the generated MeasureReport. If the MeasureReport is not persisted, this operation can be invoked with GET",
    resource: ["Measure"],
    system: false,
    type: true,
    instance: true,
    parameter: [
      {
        name: "periodStart",
        use: "in",
        min: 1,
        max: "1",
        documentation:
          "The start of the measurement period. In keeping with the semantics of the date parameter used in the FHIR search operation, the period will start at the beginning of the period implied by the supplied timestamp. E.g. a value of 2014 would set the period start to be 2014-01-01T00:00:00 inclusive",
        type: "date",
      },
      {
        name: "periodEnd",
        use: "in",
        min: 1,
        max: "1",
        documentation:
          "The end of the measurement period. The period will end at the end of the period implied by the supplied timestamp. E.g. a value of 2014 would set the period end to be 2014-12-31T23:59:59 inclusive",
        type: "date",
      },
      {
        name: "measure",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The measure to evaluate. This parameter is only required when the operation is invoked on the resource type, it is not used when invoking the operation on a Measure instance",
        type: "string",
        searchType: "reference",
      },
      {
        name: "reportType",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The type of measure report: subject, subject-list, or population. If not specified, a default value of subject will be used if the subject parameter is supplied, otherwise, population will be used",
        type: "code",
      },
      {
        name: "subject",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "Subject for which the measure will be calculated. If not specified, the measure will be calculated for all subjects that meet the requirements of the measure. If specified, the measure will only be calculated for the referenced subject(s)",
        type: "string",
        searchType: "reference",
      },
      {
        name: "practitioner",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "Practitioner for which the measure will be calculated. If specified, the measure will be calculated only for subjects that have a primary relationship to the identified practitioner",
        type: "string",
        searchType: "reference",
      },
      {
        name: "lastReceivedOn",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The date the results of this measure were last received. This parameter is only valid for patient-level reports and is used to indicate when the last time a result for this patient was received. This information can be used to limit the set of resources returned for a patient-level report",
        type: "dateTime",
      },
      {
        name: "return",
        use: "out",
        min: 1,
        max: "1",
        documentation:
          "The results of the measure calculation. See the MeasureReport resource for a complete description of the output of this operation. Note that implementations may choose to return a MeasureReport with a status of pending to indicate that the report is still being generated. In this case, the client can use a polling method to continually request the MeasureReport until the status is updated to complete",
        type: "MeasureReport",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace MeasureSubmitData {
  export type Input = {
    measureReport: fhirTypes.MeasureReport;
    resource?: Array<fhirTypes.Resource>;
  };
  export type Output = Record<string, never>;
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "Measure-submit-data",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Submit Data</h2><p>OPERATION: Submit Data</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/Measure-submit-data</pre><div><p>The submit-data operation is used to submit data-of-interest for a measure. There is no expectation that the submitted data represents all the data-of-interest, only that all the data submitted is relevant to the calculation of the measure for a particular subject or population</p>\n</div><p>URL: [base]/Measure/$submit-data</p><p>URL: [base]/Measure/[id]/$submit-data</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>measureReport</td><td>1..1</td><td><a href="measurereport.html">MeasureReport</a></td><td/><td><div><p>The measure report being submitted</p>\n</div></td></tr><tr><td>IN</td><td>resource</td><td>0..*</td><td><a href="resource.html">Resource</a></td><td/><td><div><p>The individual resources that make up the data-of-interest being submitted</p>\n</div></td></tr></table><div><p>The effect of invoking this operation is that the submitted data is posted to the receiving system and can be used for subsequent calculation of the relevant quality measure. The data-of-interest for a measure can be determined by examining the measure definition, or by invoking the $data-requirements operation</p>\n</div></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 2,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/Measure-submit-data",
    version: "4.0.1",
    name: "Submit Data",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "The submit-data operation is used to submit data-of-interest for a measure. There is no expectation that the submitted data represents all the data-of-interest, only that all the data submitted is relevant to the calculation of the measure for a particular subject or population",
    code: "submit-data",
    comment:
      "The effect of invoking this operation is that the submitted data is posted to the receiving system and can be used for subsequent calculation of the relevant quality measure. The data-of-interest for a measure can be determined by examining the measure definition, or by invoking the $data-requirements operation",
    resource: ["Measure"],
    system: false,
    type: true,
    instance: true,
    parameter: [
      {
        name: "measureReport",
        use: "in",
        min: 1,
        max: "1",
        documentation: "The measure report being submitted",
        type: "MeasureReport",
      },
      {
        name: "resource",
        use: "in",
        min: 0,
        max: "*",
        documentation:
          "The individual resources that make up the data-of-interest being submitted",
        type: "Resource",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace MedicinalProductEverything {
  export type Input = {
    _since?: fhirTypes.instant;
    _count?: fhirTypes.integer;
  };
  export type Output = fhirTypes.Bundle;
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "MedicinalProduct-everything",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Fetch Product Record</h2><p>OPERATION: Fetch Product Record</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/MedicinalProduct-everything</pre><div><p>This operation is used to return all the information related to one or more products described in the resource or context on which this operation is invoked. The response is a bundle of type &quot;searchset&quot;. At a minimum, the product resource(s) itself is returned, along with any other resources that the server has that are related to the products(s), and that are available for the given user. This is typically the marketing authorisations, ingredients, packages, therapeutic indications and so on. The server also returns whatever resources are needed to support the records - e.g. linked organizations, document references etc.</p>\n</div><p>URL: [base]/MedicinalProduct/$everything</p><p>URL: [base]/MedicinalProduct/[id]/$everything</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>_since</td><td>0..1</td><td><a href="datatypes.html#instant">instant</a></td><td/><td><div><p>Resources updated after this period will be included in the response. The intent of this parameter is to allow a client to request only records that have changed since the last request, based on either the return header time, or or (for asynchronous use), the transaction time</p>\n</div></td></tr><tr><td>IN</td><td>_count</td><td>0..1</td><td><a href="datatypes.html#integer">integer</a></td><td/><td><div><p>See discussion below on the utility of paging through the results of the $everything operation</p>\n</div></td></tr><tr><td>OUT</td><td>return</td><td>1..1</td><td><a href="bundle.html">Bundle</a></td><td/><td><div><p>The bundle type is &quot;searchset&quot;</p>\n</div></td></tr></table><div><p>The key differences between this operation and simply performing a search and using _include and _revinclude are:</p>\n<ul>\n<li>unless the client requests otherwise, the server returns the entire result set in a single bundle (rather than using paging)</li>\n<li>the server is responsible for determining what resources to return as included resources (rather than the client specifying which ones).</li>\n</ul>\n<p>This frees the client from needing to determine what it could or should ask for, particularly with regard to included resources. It also makes for a much shorter and easier to construct query string. Servers should consider returning appropriate Provenance and AuditTrail on the returned resources, even though these are not directly part of the product data.</p>\n<p>When this operation is used to access multiple product records at once, the return bundle could be rather a lot of data; servers may choose to require that such requests are made <a href="async.html">asynchronously</a>, and associated with <a href="formats.html#bulk">bulk data formats</a>. Alternatively, clients may choose to page through the result set (or servers may require this). Paging through the results is done the same as for <a href="http.html#paging">Searching</a>, using the <a href="search.html#count">_count</a> parameter, and Bundle links. Implementers should note that paging will be slower than simply returning all the results at once (more network traffic, multiple latency delays) but may be required in order not to exhaust available memory reading or writing the whole response in a single package. Unlike searching, there is no inherent user-display order for the $everything operation. Servers might consider sorting the returned resources in descending order of last record update, but are not required to do so.</p>\n<p>The _since parameter is provided to support periodic queries to get additional information that has changed about the product since the last query. This means that the _since parameter is based on record time. The value of the _since parameter should be set to the time from the server. If using direct response, this is the timestamp in the response header. If using the async interface, this is the transaction timestamp in the json response. Servers should ensure that the timestamps a managed such that the client does not miss any changes. Clients should be able to handle getting the same response more than once in the case that the transaction falls on a time boundary. Clients should ensure that the other query parameters are constant to ensure a coherent set of records when doing periodic queries.</p>\n</div></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 0,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/MedicinalProduct-everything",
    version: "4.0.1",
    name: "Fetch Product Record",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      'This operation is used to return all the information related to one or more products described in the resource or context on which this operation is invoked. The response is a bundle of type "searchset". At a minimum, the product resource(s) itself is returned, along with any other resources that the server has that are related to the products(s), and that are available for the given user. This is typically the marketing authorisations, ingredients, packages, therapeutic indications and so on. The server also returns whatever resources are needed to support the records - e.g. linked organizations, document references etc.',
    code: "everything",
    comment:
      "The key differences between this operation and simply performing a search and using _include and _revinclude are:    \n\n* unless the client requests otherwise, the server returns the entire result set in a single bundle (rather than using paging)  \n* the server is responsible for determining what resources to return as included resources (rather than the client specifying which ones). \n\nThis frees the client from needing to determine what it could or should ask for, particularly with regard to included resources. It also makes for a much shorter and easier to construct query string. Servers should consider returning appropriate Provenance and AuditTrail on the returned resources, even though these are not directly part of the product data. \n\nWhen this operation is used to access multiple product records at once, the return bundle could be rather a lot of data; servers may choose to require that such requests are made [asynchronously](async.html), and associated with [bulk data formats](formats.html#bulk). Alternatively, clients may choose to page through the result set (or servers may require this). Paging through the results is done the same as for [Searching](http.html#paging), using the [_count](search.html#count) parameter, and Bundle links. Implementers should note that paging will be slower than simply returning all the results at once (more network traffic, multiple latency delays) but may be required in order not to exhaust available memory reading or writing the whole response in a single package. Unlike searching, there is no inherent user-display order for the $everything operation. Servers might consider sorting the returned resources in descending order of last record update, but are not required to do so.\n\nThe _since parameter is provided to support periodic queries to get additional information that has changed about the product since the last query. This means that the _since parameter is based on record time. The value of the _since parameter should be set to the time from the server. If using direct response, this is the timestamp in the response header. If using the async interface, this is the transaction timestamp in the json response. Servers should ensure that the timestamps a managed such that the client does not miss any changes. Clients should be able to handle getting the same response more than once in the case that the transaction falls on a time boundary. Clients should ensure that the other query parameters are constant to ensure a coherent set of records when doing periodic queries.",
    resource: ["MedicinalProduct"],
    system: false,
    type: true,
    instance: true,
    parameter: [
      {
        name: "_since",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "Resources updated after this period will be included in the response. The intent of this parameter is to allow a client to request only records that have changed since the last request, based on either the return header time, or or (for asynchronous use), the transaction time",
        type: "instant",
      },
      {
        name: "_count",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "See discussion below on the utility of paging through the results of the $everything operation",
        type: "integer",
      },
      {
        name: "return",
        use: "out",
        min: 1,
        max: "1",
        documentation: 'The bundle type is "searchset"',
        type: "Bundle",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace MessageHeaderProcessMessage {
  export type Input = {
    content: fhirTypes.Bundle;
    async?: fhirTypes.boolean;
    "response-url"?: fhirTypes.url;
  };
  export type Output = fhirTypes.Bundle;
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "MessageHeader-process-message",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Process Message</h2><p>OPERATION: Process Message</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/MessageHeader-process-message</pre><div><p>This operation accepts a message, processes it according to the definition of the event in the message header, and returns one or more response messages.</p>\n<p>In addition to processing the message event, a server may choose to retain all or some the resources and make them available on a RESTful interface, but is not required to do so.</p>\n</div><p>URL: [base]/$process-message</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>content</td><td>1..1</td><td><a href="bundle.html">Bundle</a></td><td/><td><div><p>The message to process (or, if using asynchronous messaging, it may be a response message to accept)</p>\n</div></td></tr><tr><td>IN</td><td>async</td><td>0..1</td><td><a href="datatypes.html#boolean">boolean</a></td><td/><td><div><p>If \'true\' the message is processed using the asynchronous messaging pattern</p>\n</div></td></tr><tr><td>IN</td><td>response-url</td><td>0..1</td><td><a href="datatypes.html#url">url</a></td><td/><td><div><p>A URL to submit response messages to, if asynchronous messaging is being used, and if the MessageHeader.source.endpoint is not the appropriate place to submit responses</p>\n</div></td></tr><tr><td>OUT</td><td>return</td><td>0..1</td><td><a href="bundle.html">Bundle</a></td><td/><td><div><p>A response message, if synchronous messaging is being used (mandatory in this case). For asynchronous messaging, there is no return value</p>\n</div></td></tr></table><div><p>This operation does not use the parameters resource; the parameters &quot;async&quot; and &quot;response-url&quot; always go in the URL, if they are used, and the &quot;content&quot; parameter is always the body of the HTTP message.</p>\n<p>When processing messages, a server may return one of several status codes:</p>\n<ul>\n<li><strong>200 OK</strong>: Indicates that the message has been fully processed.  If an application-level response is expected for the submitted message, that response SHALL be returned as the body of the 200 response.</li>\n<li><strong>202 Accepted</strong>: Indicates that the receiving system has accepted custody of the message</li>\n<li><strong>204 No Content</strong>: Indicates that the message has been fully processed and would normally have had an application-level response, but because of instructions from the sender (e.g. the <a href="extension-messageheader-response-request.html">messageheader-response-request</a> extension), no response is being provided</li>\n<li><strong>300+</strong>: Indicates that the message was not successfully processed.  The server MAY return an <a href="operationoutcome.html">OperationOutcome</a> with additional information, and SHOULD do so if the response code is 400 or greater.&lt;br/&gt;\nThe client SHALL interpret a 4xx response to indicate that there is no point resubmitting the unaltered message, and a 5xx response to indicate an unexpected error occurred on the part of the server, with the implication that it may be appropriate to resubmit the original message. Doing so SHOULD NOT result in a duplicate message response. Repeated failures indicate either a fatal problem with the submission or a problem with the receiving application.</li>\n</ul>\n<p>The following rules apply when using $process-message:</p>\n<ul>\n<li>The operation only accepts POST transactions - any other HTTP method will result in an HTTP error</li>\n<li>The request content type submitted is always <a href="bundle.html">Bundle</a> with type &quot;message&quot; containing a <a href="messageheader.html">Message Header</a> resource as the first resource</li>\n<li>The response content type returned is always <a href="bundle.html">Bundle</a> with type &quot;message&quot; containing a <a href="messageheader.html">Message Header</a> resource as the first resource, or an HTTP error</li>\n<li>If the response is an error, the body SHOULD be an <a href="operationoutcome.html">Errors &amp;mp; Warning</a> resource with full details</li>\n<li>The mailbox may be authenticated using standard HTTP authentication methods, including OAuth</li>\n</ul>\n<p>The $process-message operation can be used by any HTTP end-point that accepts FHIR messages, not just FHIR RESTful servers.</p>\n<p>In order to ensure consistency of processing, the <a href="messaging.html#reliable">logical rules regarding processing of Bundle.id and message id</a> SHALL be followed when messages are processed using this operation.</p>\n<p>The $process-message operation may be used synchronously, or asynchronously.</p>\n<p>The following rules apply when using the $process-message operation synchronously:</p>\n<ul>\n<li>The URL (http://server/base/$process-message) has no parameters</li>\n<li>It is an error if the sender POSTs a message that requires multiple response messages</li>\n<li>Servers SHALL accept multiple concurrent message submissions and process them correctly (they are allowed to process them sequentially internally, but multiple concurrent submissions is not an error in its own right)</li>\n</ul>\n<p>The following rules apply when using the $process-message operation asynchronously:</p>\n<ul>\n<li>The URL has at least one parameter: http://server/base/$process-message?async=true</li>\n<li>The server acknowledges the message with a 200 OK with no body, or returns an HTTP error if the message cannot be processed</li>\n<li>Accepting the message means that the server has understood the message enough to know where to respond</li>\n<li>An <a href="operationoutcome.html">OperationOutcome</a> SHOULD be returned in either case</li>\n<li>By default, the server responds by invoking the $process-message using the sender\'s stated end-point in the message: POST [MessageHeader.source.endpoint]/$process-messages]</li>\n<li>Since the source end-point may be manipulated by message transfer engines, an alternative response address may be specified using the parameter &quot;response-url&quot;: http://server/base/$process-message?async=true&amp;amp;response-url=http://server2.com/base/anything.  The endpoint at the specified URL SHALL implement the signature of the $process-message operation (parameter async=true, accept a Bundle, return a 200 OK or an error)</li>\n<li>The server submits response messages to the appropriate end-point with the parameter async=true. There is no response message for the response messages</li>\n</ul>\n</div></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 4,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/MessageHeader-process-message",
    version: "4.0.1",
    name: "Process Message",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "This operation accepts a message, processes it according to the definition of the event in the message header, and returns one or more response messages.  \n\nIn addition to processing the message event, a server may choose to retain all or some the resources and make them available on a RESTful interface, but is not required to do so.",
    code: "process-message",
    comment:
      'This operation does not use the parameters resource; the parameters "async" and "response-url" always go in the URL, if they are used, and the "content" parameter is always the body of the HTTP message.\n\nWhen processing messages, a server may return one of several status codes:\n* **200 OK**: Indicates that the message has been fully processed.  If an application-level response is expected for the submitted message, that response SHALL be returned as the body of the 200 response.\n* **202 Accepted**: Indicates that the receiving system has accepted custody of the message\n* **204 No Content**: Indicates that the message has been fully processed and would normally have had an application-level response, but because of instructions from the sender (e.g. the [messageheader-response-request](extension-messageheader-response-request.html) extension), no response is being provided\n* **300+**: Indicates that the message was not successfully processed.  The server MAY return an [OperationOutcome](operationoutcome.html) with additional information, and SHOULD do so if the response code is 400 or greater.<br/>\n    The client SHALL interpret a 4xx response to indicate that there is no point resubmitting the unaltered message, and a 5xx response to indicate an unexpected error occurred on the part of the server, with the implication that it may be appropriate to resubmit the original message. Doing so SHOULD NOT result in a duplicate message response. Repeated failures indicate either a fatal problem with the submission or a problem with the receiving application.\n\nThe following rules apply when using $process-message:\n\n* The operation only accepts POST transactions - any other HTTP method will result in an HTTP error\n* The request content type submitted is always [Bundle](bundle.html) with type "message" containing a [Message Header](messageheader.html) resource as the first resource\n* The response content type returned is always [Bundle](bundle.html) with type "message" containing a [Message Header](messageheader.html) resource as the first resource, or an HTTP error\n* If the response is an error, the body SHOULD be an [Errors &mp; Warning](operationoutcome.html) resource with full details\n* The mailbox may be authenticated using standard HTTP authentication methods, including OAuth\n\nThe $process-message operation can be used by any HTTP end-point that accepts FHIR messages, not just FHIR RESTful servers.\n\nIn order to ensure consistency of processing, the [logical rules regarding processing of Bundle.id and message id](messaging.html#reliable) SHALL be followed when messages are processed using this operation.\n\nThe $process-message operation may be used synchronously, or asynchronously.\n\nThe following rules apply when using the $process-message operation synchronously:\n\n* The URL (http://server/base/$process-message) has no parameters\n* It is an error if the sender POSTs a message that requires multiple response messages\n* Servers SHALL accept multiple concurrent message submissions and process them correctly (they are allowed to process them sequentially internally, but multiple concurrent submissions is not an error in its own right)\n\nThe following rules apply when using the $process-message operation asynchronously:\n\n* The URL has at least one parameter: http://server/base/$process-message?async=true\n* The server acknowledges the message with a 200 OK with no body, or returns an HTTP error if the message cannot be processed\n* Accepting the message means that the server has understood the message enough to know where to respond\n* An [OperationOutcome](operationoutcome.html) SHOULD be returned in either case\n* By default, the server responds by invoking the $process-message using the sender\'s stated end-point in the message: POST [MessageHeader.source.endpoint]/$process-messages]\n* Since the source end-point may be manipulated by message transfer engines, an alternative response address may be specified using the parameter "response-url": http://server/base/$process-message?async=true&amp;response-url=http://server2.com/base/anything.  The endpoint at the specified URL SHALL implement the signature of the $process-message operation (parameter async=true, accept a Bundle, return a 200 OK or an error)\n* The server submits response messages to the appropriate end-point with the parameter async=true. There is no response message for the response messages',
    resource: ["MessageHeader"],
    system: true,
    type: false,
    instance: false,
    parameter: [
      {
        name: "content",
        use: "in",
        min: 1,
        max: "1",
        documentation:
          "The message to process (or, if using asynchronous messaging, it may be a response message to accept)",
        type: "Bundle",
      },
      {
        name: "async",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "If 'true' the message is processed using the asynchronous messaging pattern",
        type: "boolean",
      },
      {
        name: "response-url",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "A URL to submit response messages to, if asynchronous messaging is being used, and if the MessageHeader.source.endpoint is not the appropriate place to submit responses",
        type: "url",
      },
      {
        name: "return",
        use: "out",
        min: 0,
        max: "1",
        documentation:
          "A response message, if synchronous messaging is being used (mandatory in this case). For asynchronous messaging, there is no return value",
        type: "Bundle",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace NamingSystemPreferredId {
  export type Input = { id: fhirTypes.string; type: fhirTypes.code };
  export type Output = { result: fhirTypes.string };
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "NamingSystem-preferred-id",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Fetch Preferred it</h2><p>OPERATION: Fetch Preferred it</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/NamingSystem-preferred-id</pre><div><p>This operation returns the preferred identifiers for identifiers, and terminologies. The operation takes 2 parameters:</p>\n<ul>\n<li>a system identifier - either a URI, an OID, or a v2 table 0396 (other) code</li>\n<li>a code for what kind of identifier is desired (URI, OID, v2 table 0396 identifier)</li>\n</ul>\n<p>and returns either the requested identifier, or an HTTP errors response with an OperationOutcome because either the provided identifier was not recognized, or the requested identiifer type is not known.</p>\n<p>The principle use of this operation is when converting between v2, CDA and FHIR Identifier/CX/II and CodeableConcepts/C(N/W)E/CD but the operation may also find use when converting metadata such as profiles.</p>\n</div><p>URL: [base]/NamingSystem/$preferred-id</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>id</td><td>1..1</td><td><a href="datatypes.html#string">string</a></td><td/><td><div><p>The server parses the provided id to see what type it is (mary a URI, an OID as a URI, a plain OID, or a v2 table 0396 code). If the server can\'t tell what type of identifier it is, it can try it as multiple types. It is an error if more than one system matches the provided identifier</p>\n</div></td></tr><tr><td>IN</td><td>type</td><td>1..1</td><td><a href="datatypes.html#code">code</a></td><td><a href="valueset-namingsystem-identifier-type.html">http://hl7.org/fhir/ValueSet/namingsystem-identifier-type|4.0.1</a> (Required)</td><td/></tr><tr><td>OUT</td><td>result</td><td>1..1</td><td><a href="datatypes.html#string">string</a></td><td/><td><div><p>OIDs are return as plain OIDs (not the URI form).</p>\n</div></td></tr></table><div><p>Servers handle this request by finding the provided identifier in their known naming systems, and returning the requested identifier type (<a href="namingsystem-definitions.html#NamingSystem.uniqueId.type">NamingSystem.uniqueId.type</a>). If there are multiple possible identifiers of the specified type (e.g. multiple OIDs) the server returns an error.</p>\n<p>If the server wishes, it can also look through all code systems and value sets it knows about when attempting to find the requested identifier</p>\n</div></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 1,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/NamingSystem-preferred-id",
    version: "4.0.1",
    name: "Fetch Preferred it",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "This operation returns the preferred identifiers for identifiers, and terminologies. The operation takes 2 parameters:     \n\n* a system identifier - either a URI, an OID, or a v2 table 0396 (other) code  \n* a code for what kind of identifier is desired (URI, OID, v2 table 0396 identifier)    \n\nand returns either the requested identifier, or an HTTP errors response with an OperationOutcome because either the provided identifier was not recognized, or the requested identiifer type is not known.    \n\nThe principle use of this operation is when converting between v2, CDA and FHIR Identifier/CX/II and CodeableConcepts/C(N/W)E/CD but the operation may also find use when converting metadata such as profiles.",
    code: "preferred-id",
    comment:
      "Servers handle this request by finding the provided identifier in their known naming systems, and returning the requested identifier type ([NamingSystem.uniqueId.type](namingsystem-definitions.html#NamingSystem.uniqueId.type)). If there are multiple possible identifiers of the specified type (e.g. multiple OIDs) the server returns an error.    \n\nIf the server wishes, it can also look through all code systems and value sets it knows about when attempting to find the requested identifier",
    resource: ["NamingSystem"],
    system: false,
    type: true,
    instance: false,
    parameter: [
      {
        name: "id",
        use: "in",
        min: 1,
        max: "1",
        documentation:
          "The server parses the provided id to see what type it is (mary a URI, an OID as a URI, a plain OID, or a v2 table 0396 code). If the server can't tell what type of identifier it is, it can try it as multiple types. It is an error if more than one system matches the provided identifier",
        type: "string",
      },
      {
        name: "type",
        use: "in",
        min: 1,
        max: "1",
        type: "code",
        binding: {
          strength: "required",
          valueSet:
            "http://hl7.org/fhir/ValueSet/namingsystem-identifier-type|4.0.1",
        },
      },
      {
        name: "result",
        use: "out",
        min: 1,
        max: "1",
        documentation: "OIDs are return as plain OIDs (not the URI form).",
        type: "string",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace ObservationLastn {
  export type Input = { max?: fhirTypes.positiveInt };
  export type Output = fhirTypes.Bundle;
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "Observation-lastn",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: "<div xmlns=\"http://www.w3.org/1999/xhtml\"><h2>Last N Observations Query</h2><p>OPERATION: Last N Observations Query</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/Observation-lastn</pre><div><p>The <em>lastn query</em> meets the common need for searching for the most recent or last n=number of observations for a subject. For example, retrieving the last 5 temperatures for a patient to view trends or fetching the most recent laboratory results or vitals signs. To ask a server to return the last n=number of observations, the <em>lastn</em> query uses the <a href=\"observation.html#search\">normal search parameters</a> defined for the Observation resource.  However, rather than their normal use, they are interpreted as inputs - i.e.. instead of requiring that the resources literally contain the search parameters, they are passed to a server algorithm of some kind that uses them to determine the most appropriate matches.</p>\n<p>The request for a lastn query SHALL include:</p>\n<ul>\n<li>A <code>$lastn</code> operation parameter</li>\n<li>A subject using either the <code>patient</code> or <code>subject</code>  search parameter</li>\n<li>A <code>category</code> parameter and/or a search parameter that contains a code element in its FHIRpath expression.  ( e.g., <code>code</code> or <code>code-value-concept</code>)</li>\n</ul>\n<p>The request for a lastn query MAY include:</p>\n<ul>\n<li>Other Observation search parameters and modifiers</li>\n</ul>\n<p>The response from a lastn query is a set of observations:</p>\n<ul>\n<li>Filtered by additional parameters\n<ul>\n<li>If not explicitly filtered by status then will include statuses of 'entered-in-error'</li>\n</ul>\n</li>\n<li>'GROUP BY' <code>Observation.code</code>\n<ul>\n<li>Codes SHALL be considered equivalent if the <code>coding.value</code> <em>and</em> <code>coding.system</code> are the same.</li>\n<li>Text only codes SHALL be treated and grouped based on the text.</li>\n<li>For codes with translations (multiple codings), the code translations are assumed to be equal and the grouping by code SHALL follow the transitive property of equality.</li>\n</ul>\n</li>\n</ul>\n<p>for example:</p>\n<table class=\"grid\">\n<thead>\n<tr><th>Observation.code for observation a</th><th>Observation.code for observation b</th><th>Observation.code for observation c</th><th>number of groups [codes/text in each group]</th></tr>\n</thead>\n<tbody>\n<tr><td>a</td><td>b</td><td>c</td><td>3 [a],[b],[c]</td></tr>\n<tr><td>a</td><td>b</td><td>a,c</td><td>2 [a.c],[b]</td></tr>\n<tr><td>a</td><td>b</td><td>a,b</td><td>1 [a,b]</td></tr>\n<tr><td>'textM'</td><td>'Text'</td><td>'t e x t'</td><td>3 ['text'],['Text'],['t e x t']</td></tr>\n</tbody>\n</table>\n<ul>\n<li>Sorted from most recent to the oldest</li>\n<li>Limited to the number of requested responses per group specified by the optional <em>max</em> query parameter\n<ul>\n<li>In case of a tie - when the effective times for &gt;1 Observations are equal - both will be returned.  Therefore, more Observations may be returned than is specified in <em>max</em>.  For example, 4 Observations instead of 3 if the 3rd and 4th most recent observation had the same effective time.</li>\n</ul>\n</li>\n<li>If no maximum number is given then only the most recent Observation in each group is returned.</li>\n</ul>\n<p>The set of returned observations should represent distinct real world observations and not the same observation with changes in status or versions. If there are no matches, the <em>lastn</em> query SHALL return an empty search set with no error, but may include an operation outcome with further advice.</p>\n</div><p>URL: [base]/Observation/$lastn</p><p>Parameters</p><table class=\"grid\"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>max</td><td>0..1</td><td><a href=\"datatypes.html#positiveInt\">positiveInt</a></td><td/><td><div><p><code>max</code> is  an optional input parameter to the <em>lastn</em> query operation.  It is used to specify the maximum number of Observations to return from each group. For example for the query &quot;Fetch the last 3 results for all vitals for a patient&quot; <code>max</code> = 3.</p>\n</div></td></tr><tr><td>OUT</td><td>return</td><td>1..1</td><td><a href=\"bundle.html\">Bundle</a></td><td/><td><div><p>The set of most recent N Observations that match the <em>lastn</em> query search criteria.</p>\n</div></td></tr></table><div><p>The key differences between this query operation and simply searching Observation using the combination of <code>_count</code> and <code>_sort</code> parameters are:</p>\n<ul>\n<li>The <em>lastn</em> query returns <strong>only</strong> the last N resource grouped by code.  Using the _count query method doesn't restrict the total matches so you may need to page through several &quot;A&quot; Observations  before getting to Observation &quot;B&quot;.</li>\n<li>The server is responsible for grouping the observations by codes.  This frees the client from needing to determine which codes she should ask for.</li>\n</ul>\n<p>This operation cannot be performed on observations that the user is not authorized to see.  It is assumed that the server has identified and secured the context appropriately, and can either associate the authorization context with a single patient, or determine whether the context has the rights to the nominated patient, if there is one. If there is no nominated patient (e.g. the operation is invoked at the system level) and the context is not associated with a single patient record, then the server should return an error. Specifying the relationship between the context, a user and patient records is outside the scope of this specification.</p>\n</div></div>",
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 3,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/Observation-lastn",
    version: "4.0.1",
    name: "Last N Observations Query",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "The *lastn query* meets the common need for searching for the most recent or last n=number of observations for a subject. For example, retrieving the last 5 temperatures for a patient to view trends or fetching the most recent laboratory results or vitals signs. To ask a server to return the last n=number of observations, the *lastn* query uses the [normal search parameters](observation.html#search) defined for the Observation resource.  However, rather than their normal use, they are interpreted as inputs - i.e.. instead of requiring that the resources literally contain the search parameters, they are passed to a server algorithm of some kind that uses them to determine the most appropriate matches.\n\nThe request for a lastn query SHALL include:\n\n* A `$lastn` operation parameter\n*  A subject using either the `patient` or `subject`  search parameter\n*  A `category` parameter and/or a search parameter that contains a code element in its FHIRpath expression.  ( e.g., `code` or `code-value-concept`)\n\nThe request for a lastn query MAY include:\n\n* Other Observation search parameters and modifiers\n\nThe response from a lastn query is a set of observations:\n\n*  Filtered by additional parameters\n   * If not explicitly filtered by status then will include statuses of 'entered-in-error'\n* 'GROUP BY' `Observation.code`\n   * Codes SHALL be considered equivalent if the `coding.value` *and* `coding.system` are the same.\n   * Text only codes SHALL be treated and grouped based on the text.\n   * For codes with translations (multiple codings), the code translations are assumed to be equal and the grouping by code SHALL follow the transitive property of equality.\n\nfor example:\n\n|Observation.code for observation a|Observation.code for observation b|Observation.code for observation c|number of groups [codes/text in each group]|    \n|---|---|---|---|     \n|a|b|c | 3 [a],[b],[c]|    \n|a|b|a,c | 2 [a.c],[b]|     \n|a|b|a,b | 1 [a,b]|    \n|'textM'|'Text'|'t e x t'|3 ['text'],['Text'],['t e x t']|\n\n* Sorted from most recent to the oldest\n* Limited to the number of requested responses per group specified by the optional *max* query parameter\n  * In case of a tie - when the effective times for >1 Observations are equal - both will be returned.  Therefore, more Observations may be returned than is specified in *max*.  For example, 4 Observations instead of 3 if the 3rd and 4th most recent observation had the same effective time.\n* If no maximum number is given then only the most recent Observation in each group is returned.\n\nThe set of returned observations should represent distinct real world observations and not the same observation with changes in status or versions. If there are no matches, the *lastn* query SHALL return an empty search set with no error, but may include an operation outcome with further advice.",
    code: "lastn",
    comment:
      'The key differences between this query operation and simply searching Observation using the combination of `_count` and `_sort` parameters are:\r\r* The *lastn* query returns **only** the last N resource grouped by code.  Using the _count query method doesn\'t restrict the total matches so you may need to page through several "A" Observations  before getting to Observation "B".\r* The server is responsible for grouping the observations by codes.  This frees the client from needing to determine which codes she should ask for.\r\rThis operation cannot be performed on observations that the user is not authorized to see.  It is assumed that the server has identified and secured the context appropriately, and can either associate the authorization context with a single patient, or determine whether the context has the rights to the nominated patient, if there is one. If there is no nominated patient (e.g. the operation is invoked at the system level) and the context is not associated with a single patient record, then the server should return an error. Specifying the relationship between the context, a user and patient records is outside the scope of this specification.',
    resource: ["Observation"],
    system: false,
    type: true,
    instance: false,
    parameter: [
      {
        name: "max",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          '`max` is  an optional input parameter to the *lastn* query operation.  It is used to specify the maximum number of Observations to return from each group. For example for the query "Fetch the last 3 results for all vitals for a patient" `max` = 3.',
        type: "positiveInt",
      },
      {
        name: "return",
        use: "out",
        min: 1,
        max: "1",
        documentation:
          "The set of most recent N Observations that match the *lastn* query search criteria.",
        type: "Bundle",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace ObservationStats {
  export type Input = {
    subject: fhirTypes.uri;
    code?: Array<fhirTypes.string>;
    system?: fhirTypes.uri;
    coding?: Array<fhirTypes.Coding>;
    duration?: fhirTypes.decimal;
    period?: fhirTypes.Period;
    statistic: Array<fhirTypes.code>;
    include?: fhirTypes.boolean;
    limit?: fhirTypes.positiveInt;
  };
  export type Output = {
    statistics: Array<fhirTypes.Observation>;
    source?: Array<fhirTypes.Observation>;
  };
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "Observation-stats",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Observation Statistics</h2><p>OPERATION: Observation Statistics</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/Observation-stats</pre><div><p>The Statistics operation performs a set of statistical calculations on a set of clinical measurements such as a blood pressure as stored on the server.  This operation evaluates <a href="observation.html">Observation</a> resources having valueQuantity elements that have UCUM unit codes. Observations with a status of \'entered-in-error\' will be excluded from the calculations.</p>\n<p>The set of Observations is defined by 4 parameters:</p>\n<ul>\n<li>the subject of the observations for which the statistics are being generated (<code>subject</code>)</li>\n<li>which observations to generate statistics for (<code>code</code> and <code>system</code>, or <code>coding</code>)</li>\n<li>the time period over which to generate statistics \'duration<code>or</code>period`)</li>\n<li>the set of statistical analyses to return (<code>statistic</code>)</li>\n</ul>\n<p>Possible statistical analyses (see <a href="valueset-observation-statistics.html">StatisticsCode</a>):</p>\n<ul>\n<li><strong>average</strong> (&quot;Average&quot;): The <a href="https://en.wikipedia.org/wiki/Arithmetic_mean">mean</a> of N measurements over the stated period.</li>\n<li><strong>maximum</strong> (&quot;Maximum&quot;): The <a href="https://en.wikipedia.org/wiki/Maximal_element">maximum</a> value of N measurements over the stated period.</li>\n<li><strong>minimum</strong> (&quot;Minimum&quot;): The <a href="https://en.wikipedia.org/wiki/Minimal_element">minimum</a> value of N measurements over the stated period.</li>\n<li><strong>count</strong> (&quot;Count&quot;): The [number] of valid measurements over the stated period that contributed to the other statistical outputs.</li>\n<li><strong>total-count</strong> (&quot;Total Count&quot;): The total [number] of valid measurements over the stated period, including observations that were ignored because they did not contain valid result values.</li>\n<li><strong>median</strong> (&quot;Median&quot;): The <a href="https://en.wikipedia.org/wiki/Median">median</a> of N measurements over the stated period.</li>\n<li><strong>std-dev</strong> (&quot;Standard Deviation&quot;): The <a href="https://en.wikipedia.org/wiki/Standard_deviation">standard deviation</a> of N measurements over the stated period.</li>\n<li><strong>sum</strong> (&quot;Sum&quot;): The <a href="https://en.wikipedia.org/wiki/Summation">sum</a> of N measurements over the stated period.</li>\n<li><strong>variance</strong> (&quot;Variance&quot;): The <a href="https://en.wikipedia.org/wiki/Variance">variance</a> of N measurements over the stated period.</li>\n<li><strong>20-percent</strong> (&quot;20th Percentile&quot;): The 20th <a href="https://en.wikipedia.org/wiki/Percentile">Percentile</a> of N measurements over the stated period.</li>\n<li><strong>80-percent</strong> (&quot;80th Percentile&quot;): The 80th <a href="https://en.wikipedia.org/wiki/Percentile">Percentile</a> of N measurements over the stated period.</li>\n<li><strong>4-lower</strong> (&quot;Lower Quartile&quot;): The lower <a href="https://en.wikipedia.org/wiki/Quartile">Quartile</a> Boundary of N measurements over the stated period.</li>\n<li><strong>4-upper</strong> (&quot;Upper Quartile&quot;): The upper <a href="https://en.wikipedia.org/wiki/Quartile">Quartile</a> Boundary of N measurements over the stated period.</li>\n<li><strong>4-dev</strong> (&quot;Quartile Deviation&quot;): The difference between the upper and lower <a href="https://en.wikipedia.org/wiki/Quartile">Quartiles</a> is called the Interquartile range. (IQR = Q3-Q1) Quartile deviation or Semi-interquartile range is one-half the difference between the first and the third quartiles.</li>\n<li><strong>5-1</strong> (&quot;1st Quintile&quot;): The lowest of four values that divide the N measurements into a frequency distribution of five classes with each containing one fifth of the total population.</li>\n<li><strong>5-2</strong> (&quot;2nd Quintile&quot;): The second of four values that divide the N measurements into a frequency distribution of five classes with each containing one fifth of the total population.</li>\n<li><strong>5-3</strong> (&quot;3rd Quintile&quot;): The third of four values that divide the N measurements into a frequency distribution of five classes with each containing one fifth of the total population.</li>\n<li><strong>5-4</strong> (&quot;4th Quintile&quot;): The fourth of four values that divide the N measurements into a frequency distribution of five classes with each containing one fifth of the total population.</li>\n<li><strong>skew</strong> (&quot;Skew&quot;): Skewness is a measure of the asymmetry of the probability distribution of a real-valued random variable about its mean. The skewness value can be positive or negative, or even undefined.  Source: <a href="https://en.wikipedia.org/wiki/Skewness">Wikipedia</a>.</li>\n<li><strong>kurtosis</strong> (&quot;Kurtosis&quot;): Kurtosis  is a measure of the &quot;tailedness&quot; of the probability distribution of a real-valued random variable.   Source: <a href="https://en.wikipedia.org/wiki/Kurtosis">Wikipedia</a>.</li>\n<li><strong>regression</strong> (&quot;Regression&quot;): Linear regression is an approach for modeling two-dimensional sample points with one independent variable and one dependent variable (conventionally, the x and y coordinates in a Cartesian coordinate system) and finds a linear function (a non-vertical straight line) that, as accurately as possible, predicts the dependent variable values as a function of the independent variables. Source: <a href="https://en.wikipedia.org/wiki/Simple_linear_regression">Wikipedia</a>  This Statistic code will return both a gradient and an intercept value.</li>\n</ul>\n<p>If successful, the operation returns an Observation resource for each code with the results of the statistical calculations as component value pairs where the component code = the statistical code. The Observation also contains the input parameters <code>patient</code>,<code>code</code> and <code>duration</code> parameters. If unsuccessful, an <a href="operationoutcome.html">OperationOutcome</a> with an error message will be returned.</p>\n<p>The client can request that all the observations on which the statistics are based be returned as well, using the include parameter. If an include parameter is specified, a limit may also be specified; the sources observations are subsetted at the server\'s discretion if count &gt; limit. This functionality is included with the intent of supporting graphical presentation</p>\n</div><p>URL: [base]/Observation/$stats</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>subject</td><td>1..1</td><td><a href="datatypes.html#uri">uri</a></td><td/><td><div><p>The subject of the relevant Observations, which has the value of the Observation.subject.reference. E.g. \'Patient/123\'. Reference can be to an absolute URL, but servers only perform stats on their own observations</p>\n</div></td></tr><tr><td>IN</td><td>code</td><td>0..*</td><td><a href="datatypes.html#string">string</a></td><td/><td><div><p>The test code(s) upon which the statistics are being performed. Provide along with a system, or as a coding. For example, the LOINC code  =\n2339-0 (Glucose [Mass/​volume] in Blood) will evaluate all relevant Observations with this code in <code>Observation.code</code> and <code>Observation.component.code</code>. For LOINC codes that are panels, e.g., 85354-9(Blood pressure panel with all children optional), the stats operation returns statistics for each of the individual panel measurements.  That means it will include and evaluate all values grouped by code for all the individual observations that are: 1) referenced in   <code>.related</code> for <code>.related.type</code> = \'has-member\'  and 2) component observations in <code>Observation.component</code>.</p>\n</div></td></tr><tr><td>IN</td><td>system</td><td>0..1</td><td><a href="datatypes.html#uri">uri</a></td><td/><td><div><p>The system for the code(s). Or provide a coding instead</p>\n</div></td></tr><tr><td>IN</td><td>coding</td><td>0..*</td><td><a href="datatypes.html#Coding">Coding</a></td><td/><td><div><p>The test code upon which the statistics are being performed, as a Coding</p>\n</div></td></tr><tr><td>IN</td><td>duration</td><td>0..1</td><td><a href="datatypes.html#decimal">decimal</a></td><td/><td><div><p>The time period of interest given as hours.  For example, the duration = &quot;1&quot; represents the last hour - the time period from on hour ago to now</p>\n</div></td></tr><tr><td>IN</td><td>period</td><td>0..1</td><td><a href="datatypes.html#Period">Period</a></td><td/><td><div><p>The time period over which the calculations to be performed, if a duration is not provided</p>\n</div></td></tr><tr><td>IN</td><td>statistic</td><td>1..*</td><td><a href="datatypes.html#code">code</a></td><td/><td><div><p>average|max|min|count  The statistical operations to be performed on the relevant operations. Multiple statistics operations can be specified. These codes are defined <a href="valueset-observation-statistics.html">here</a></p>\n</div></td></tr><tr><td>IN</td><td>include</td><td>0..1</td><td><a href="datatypes.html#boolean">boolean</a></td><td/><td><div><p>Whether to return the observations on which the statistics are based</p>\n</div></td></tr><tr><td>IN</td><td>limit</td><td>0..1</td><td><a href="datatypes.html#positiveInt">positiveInt</a></td><td/><td><div><p>If an include parameter is specified, a limit may also be specified to limit the number of source Observations returned.  If the include paramter is absent or equal to &quot;false&quot; the limit parameter SHALL be ignored by the server</p>\n</div></td></tr><tr><td>OUT</td><td>statistics</td><td>1..*</td><td><a href="observation.html">Observation</a></td><td/><td><div><p>A set of observations, one observation for each code, each containing one component for each statistic. The Observation.component.code contains the statistic, and is relative to the Observation.code and cannot be interpreted independently.  The Observation will also contain a subject, effectivePeriod, and code reflecting the input parameters.  The status is fixed to <code>final</code>.</p>\n</div></td></tr><tr><td>OUT</td><td>source</td><td>0..*</td><td><a href="observation.html">Observation</a></td><td/><td><div><p>Source observations on which the statistics are based</p>\n</div></td></tr></table><div><p>If <a href="extensibility.html#modifierExtension">modifier extensions</a> are present in the Observation, they must be accounted for by implementers.  A modifier extension may affect the observation.value in a way that it should be excluded from the from the calculations.</p>\n<p>This operation cannot be performed on observations that the user is not authorized to see.  It is assumed that the server has identified and secured the context appropriately, and can either associate the authorization context with a single patient, or determine whether the context has the rights to the nominated patient, if there is one. If there is no nominated patient (e.g. the operation is invoked at the system level) and the context is not associated with a single patient record, then the server should return an error. Specifying the relationship between the context, a user and patient records is outside the scope of this specification.</p>\n</div></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 3,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/Observation-stats",
    version: "4.0.1",
    name: "Observation Statistics",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      'The Statistics operation performs a set of statistical calculations on a set of clinical measurements such as a blood pressure as stored on the server.  This operation evaluates [Observation](observation.html) resources having valueQuantity elements that have UCUM unit codes. Observations with a status of \'entered-in-error\' will be excluded from the calculations.  \r\rThe set of Observations is defined by 4 parameters:\r\r*  the subject of the observations for which the statistics are being generated (`subject`)\r* which observations to generate statistics for (`code` and `system`, or `coding`)\r* the time period over which to generate statistics \'duration` or `period`)\r* the set of statistical analyses to return (`statistic`)\r\rPossible statistical analyses (see [StatisticsCode](valueset-observation-statistics.html)):\r\r - **average** ("Average"): The [mean](https://en.wikipedia.org/wiki/Arithmetic_mean) of N measurements over the stated period.\r\n - **maximum** ("Maximum"): The [maximum](https://en.wikipedia.org/wiki/Maximal_element) value of N measurements over the stated period.\r\n - **minimum** ("Minimum"): The [minimum](https://en.wikipedia.org/wiki/Minimal_element) value of N measurements over the stated period.\r\n - **count** ("Count"): The [number] of valid measurements over the stated period that contributed to the other statistical outputs.\r\n - **total-count** ("Total Count"): The total [number] of valid measurements over the stated period, including observations that were ignored because they did not contain valid result values.\r\n - **median** ("Median"): The [median](https://en.wikipedia.org/wiki/Median) of N measurements over the stated period.\r\n - **std-dev** ("Standard Deviation"): The [standard deviation](https://en.wikipedia.org/wiki/Standard_deviation) of N measurements over the stated period.\r\n - **sum** ("Sum"): The [sum](https://en.wikipedia.org/wiki/Summation) of N measurements over the stated period.\r\n - **variance** ("Variance"): The [variance](https://en.wikipedia.org/wiki/Variance) of N measurements over the stated period.\r\n - **20-percent** ("20th Percentile"): The 20th [Percentile](https://en.wikipedia.org/wiki/Percentile) of N measurements over the stated period.\r\n - **80-percent** ("80th Percentile"): The 80th [Percentile](https://en.wikipedia.org/wiki/Percentile) of N measurements over the stated period.\r\n - **4-lower** ("Lower Quartile"): The lower [Quartile](https://en.wikipedia.org/wiki/Quartile) Boundary of N measurements over the stated period.\r\n - **4-upper** ("Upper Quartile"): The upper [Quartile](https://en.wikipedia.org/wiki/Quartile) Boundary of N measurements over the stated period.\r\n - **4-dev** ("Quartile Deviation"): The difference between the upper and lower [Quartiles](https://en.wikipedia.org/wiki/Quartile) is called the Interquartile range. (IQR = Q3-Q1) Quartile deviation or Semi-interquartile range is one-half the difference between the first and the third quartiles.\r\n - **5-1** ("1st Quintile"): The lowest of four values that divide the N measurements into a frequency distribution of five classes with each containing one fifth of the total population.\r\n - **5-2** ("2nd Quintile"): The second of four values that divide the N measurements into a frequency distribution of five classes with each containing one fifth of the total population.\r\n - **5-3** ("3rd Quintile"): The third of four values that divide the N measurements into a frequency distribution of five classes with each containing one fifth of the total population.\r\n - **5-4** ("4th Quintile"): The fourth of four values that divide the N measurements into a frequency distribution of five classes with each containing one fifth of the total population.\r\n - **skew** ("Skew"): Skewness is a measure of the asymmetry of the probability distribution of a real-valued random variable about its mean. The skewness value can be positive or negative, or even undefined.  Source: [Wikipedia](https://en.wikipedia.org/wiki/Skewness).\r\n - **kurtosis** ("Kurtosis"): Kurtosis  is a measure of the "tailedness" of the probability distribution of a real-valued random variable.   Source: [Wikipedia](https://en.wikipedia.org/wiki/Kurtosis).\r\n - **regression** ("Regression"): Linear regression is an approach for modeling two-dimensional sample points with one independent variable and one dependent variable (conventionally, the x and y coordinates in a Cartesian coordinate system) and finds a linear function (a non-vertical straight line) that, as accurately as possible, predicts the dependent variable values as a function of the independent variables. Source: [Wikipedia](https://en.wikipedia.org/wiki/Simple_linear_regression)  This Statistic code will return both a gradient and an intercept value.\r\n\r\rIf successful, the operation returns an Observation resource for each code with the results of the statistical calculations as component value pairs where the component code = the statistical code. The Observation also contains the input parameters `patient`,`code` and `duration` parameters. If unsuccessful, an [OperationOutcome](operationoutcome.html) with an error message will be returned.\r\rThe client can request that all the observations on which the statistics are based be returned as well, using the include parameter. If an include parameter is specified, a limit may also be specified; the sources observations are subsetted at the server\'s discretion if count > limit. This functionality is included with the intent of supporting graphical presentation',
    code: "stats",
    comment:
      "If [modifier extensions](extensibility.html#modifierExtension) are present in the Observation, they must be accounted for by implementers.  A modifier extension may affect the observation.value in a way that it should be excluded from the from the calculations.\r\rThis operation cannot be performed on observations that the user is not authorized to see.  It is assumed that the server has identified and secured the context appropriately, and can either associate the authorization context with a single patient, or determine whether the context has the rights to the nominated patient, if there is one. If there is no nominated patient (e.g. the operation is invoked at the system level) and the context is not associated with a single patient record, then the server should return an error. Specifying the relationship between the context, a user and patient records is outside the scope of this specification.",
    resource: ["Observation"],
    system: false,
    type: true,
    instance: false,
    parameter: [
      {
        name: "subject",
        use: "in",
        min: 1,
        max: "1",
        documentation:
          "The subject of the relevant Observations, which has the value of the Observation.subject.reference. E.g. 'Patient/123'. Reference can be to an absolute URL, but servers only perform stats on their own observations",
        type: "uri",
      },
      {
        name: "code",
        use: "in",
        min: 0,
        max: "*",
        documentation:
          "The test code(s) upon which the statistics are being performed. Provide along with a system, or as a coding. For example, the LOINC code  = \r2339-0 (Glucose [Mass/​volume] in Blood) will evaluate all relevant Observations with this code in `Observation.code` and `Observation.component.code`. For LOINC codes that are panels, e.g., 85354-9(Blood pressure panel with all children optional), the stats operation returns statistics for each of the individual panel measurements.  That means it will include and evaluate all values grouped by code for all the individual observations that are: 1) referenced in   `.related` for `.related.type` = 'has-member'  and 2) component observations in `Observation.component`.",
        type: "string",
      },
      {
        name: "system",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The system for the code(s). Or provide a coding instead",
        type: "uri",
      },
      {
        name: "coding",
        use: "in",
        min: 0,
        max: "*",
        documentation:
          "The test code upon which the statistics are being performed, as a Coding",
        type: "Coding",
      },
      {
        name: "duration",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          'The time period of interest given as hours.  For example, the duration = "1" represents the last hour - the time period from on hour ago to now',
        type: "decimal",
      },
      {
        name: "period",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The time period over which the calculations to be performed, if a duration is not provided",
        type: "Period",
      },
      {
        name: "statistic",
        use: "in",
        min: 1,
        max: "*",
        documentation:
          "average|max|min|count  The statistical operations to be performed on the relevant operations. Multiple statistics operations can be specified. These codes are defined [here](valueset-observation-statistics.html)",
        type: "code",
      },
      {
        name: "include",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "Whether to return the observations on which the statistics are based",
        type: "boolean",
      },
      {
        name: "limit",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          'If an include parameter is specified, a limit may also be specified to limit the number of source Observations returned.  If the include paramter is absent or equal to "false" the limit parameter SHALL be ignored by the server',
        type: "positiveInt",
      },
      {
        name: "statistics",
        use: "out",
        min: 1,
        max: "*",
        documentation:
          "A set of observations, one observation for each code, each containing one component for each statistic. The Observation.component.code contains the statistic, and is relative to the Observation.code and cannot be interpreted independently.  The Observation will also contain a subject, effectivePeriod, and code reflecting the input parameters.  The status is fixed to `final`.",
        type: "Observation",
      },
      {
        name: "source",
        use: "out",
        min: 0,
        max: "*",
        documentation: "Source observations on which the statistics are based",
        type: "Observation",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace PatientEverything {
  export type Input = {
    start?: fhirTypes.date;
    end?: fhirTypes.date;
    _since?: fhirTypes.instant;
    _type?: Array<fhirTypes.code>;
    _count?: fhirTypes.integer;
  };
  export type Output = fhirTypes.Bundle;
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "Patient-everything",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Fetch Patient Record</h2><p>OPERATION: Fetch Patient Record</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/Patient-everything</pre><div><p>This operation is used to return all the information related to one or more patients described in the resource or context on which this operation is invoked. The response is a bundle of type &quot;searchset&quot;. At a minimum, the patient resource(s) itself is returned, along with any other resources that the server has that are related to the patient(s), and that are available for the given user. The server also returns whatever resources are needed to support the records - e.g. linked practitioners, medications, locations, organizations etc.</p>\n<p>The intended use for this operation is to provide a patient with access to their entire record (e.g. &quot;Blue Button&quot;), or for provider or other user to perform a bulk data download.  The server SHOULD return at least all resources that it has that are in the patient compartment for the identified patient(s), and any resource referenced from those, including binaries and attachments. In the US Realm, at a minimum, the resources returned SHALL include all the data covered by the meaningful use common data elements as defined in the US Core Implementation Guide. Other applicable implementation guides may make additional rules about how much information that is returned.</p>\n</div><p>URL: [base]/Patient/$everything</p><p>URL: [base]/Patient/[id]/$everything</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>start</td><td>0..1</td><td><a href="datatypes.html#date">date</a></td><td/><td><div><p>The date range relates to care dates, not record currency dates - e.g. all records relating to care provided in a certain date range. If no start date is provided, all records prior to the end date are in scope.</p>\n</div></td></tr><tr><td>IN</td><td>end</td><td>0..1</td><td><a href="datatypes.html#date">date</a></td><td/><td><div><p>The date range relates to care dates, not record currency dates - e.g. all records relating to care provided in a certain date range. If no end date is provided, all records subsequent to the start date are in scope.</p>\n</div></td></tr><tr><td>IN</td><td>_since</td><td>0..1</td><td><a href="datatypes.html#instant">instant</a></td><td/><td><div><p>Resources updated after this period will be included in the response. The intent of this parameter is to allow a client to request only records that have changed since the last request, based on either the return header time, or or (for asynchronous use), the transaction time</p>\n</div></td></tr><tr><td>IN</td><td>_type</td><td>0..*</td><td><a href="datatypes.html#code">code</a></td><td/><td><div><p>One or more parameters, each containing one or more comma-delimited FHIR resource types to include in the return resources. In the absence of any specified types, the server returns all resource types</p>\n</div></td></tr><tr><td>IN</td><td>_count</td><td>0..1</td><td><a href="datatypes.html#integer">integer</a></td><td/><td><div><p>See discussion below on the utility of paging through the results of the $everything operation</p>\n</div></td></tr><tr><td>OUT</td><td>return</td><td>1..1</td><td><a href="bundle.html">Bundle</a></td><td/><td><div><p>The bundle type is &quot;searchset&quot;</p>\n</div></td></tr></table><div><p>The key differences between this operation and simply searching the patient compartment are:</p>\n<ul>\n<li>unless the client requests otherwise, the server returns the entire result set in a single bundle (rather than using paging)</li>\n<li>the server is responsible for determining what resources to return as included resources (rather than the client specifying which ones).</li>\n</ul>\n<p>This frees the client from needing to determine what it could or should ask for, particularly with regard to included resources. Servers should consider returning appropriate Provenance and AuditTrail on the returned resources, even though these are not directly part of the patient compartment.</p>\n<p>It is assumed that the server has identified and secured the context appropriately, and can either associate the authorization context with a single patient, or determine whether the context has the rights to the nominated patient, if there is one, or can determine an appropriate list of patients to provide data for from the context of the request.   If there is no nominated patient (GET /Patient/$everything) and the context is not associated with a single patient record, the actual list of patients is all patients that the user associated with the request has access to. This may be all patients in the family that the patient has access to, or it may be all patients that a care provider has access to, or all patients on the entire record system. In such cases, the server may choose to return an error rather than all the records.  Specifying the relationship between the context, a user and patient records is outside the scope of this specification (though see <a href="http://hl7.org/fhir/smart-app-launch">The Smart App Launch Implementation Guide</a>.</p>\n<p>When this operation is used to access multiple patient records at once, the return bundle could be rather a lot of data; servers may choose to require that such requests are made <a href="async.html">asynchronously</a>, and associated with <a href="formats.html#bulk">bulk data formats</a>. Alternatively, clients may choose to page through the result set (or servers may require this). Paging through the results is done the same as for <a href="http.html#paging">Searching</a>, using the <a href="search.html#count">_count</a> parameter, and Bundle links. Implementers should note that paging will be slower than simply returning all the results at once (more network traffic, multiple latency delays) but may be required in order not to exhaust available memory reading or writing the whole response in a single package. Unlike searching, there is no inherent user-display order for the $everything operation. Servers might consider sorting the returned resources in descending order of last record update, but are not required to do so.</p>\n<p>The _since parameter is provided to support periodic queries to get additional information that has changed about the patient since the last query. This means that the _since parameter is based on record time. The value of the _since parameter should be set to the time from the server. If using direct response, this is the timestamp in the response header. If using the async interface, this is the transaction timestamp in the json response. Servers should ensure that the timestamps a managed such that the client does not miss any changes. Clients should be able to handle getting the same response more than once in the case that the transaction falls on a time boundary. Clients should ensure that the other query parameters are constant to ensure a coherent set of records when doing periodic queries.</p>\n</div></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 5,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/Patient-everything",
    version: "4.0.1",
    name: "Fetch Patient Record",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      'This operation is used to return all the information related to one or more patients described in the resource or context on which this operation is invoked. The response is a bundle of type "searchset". At a minimum, the patient resource(s) itself is returned, along with any other resources that the server has that are related to the patient(s), and that are available for the given user. The server also returns whatever resources are needed to support the records - e.g. linked practitioners, medications, locations, organizations etc.   \n\nThe intended use for this operation is to provide a patient with access to their entire record (e.g. "Blue Button"), or for provider or other user to perform a bulk data download.  The server SHOULD return at least all resources that it has that are in the patient compartment for the identified patient(s), and any resource referenced from those, including binaries and attachments. In the US Realm, at a minimum, the resources returned SHALL include all the data covered by the meaningful use common data elements as defined in the US Core Implementation Guide. Other applicable implementation guides may make additional rules about how much information that is returned.',
    code: "everything",
    comment:
      "The key differences between this operation and simply searching the patient compartment are:    \n\n* unless the client requests otherwise, the server returns the entire result set in a single bundle (rather than using paging)  \n* the server is responsible for determining what resources to return as included resources (rather than the client specifying which ones). \n\nThis frees the client from needing to determine what it could or should ask for, particularly with regard to included resources. Servers should consider returning appropriate Provenance and AuditTrail on the returned resources, even though these are not directly part of the patient compartment. \n\nIt is assumed that the server has identified and secured the context appropriately, and can either associate the authorization context with a single patient, or determine whether the context has the rights to the nominated patient, if there is one, or can determine an appropriate list of patients to provide data for from the context of the request.   If there is no nominated patient (GET /Patient/$everything) and the context is not associated with a single patient record, the actual list of patients is all patients that the user associated with the request has access to. This may be all patients in the family that the patient has access to, or it may be all patients that a care provider has access to, or all patients on the entire record system. In such cases, the server may choose to return an error rather than all the records.  Specifying the relationship between the context, a user and patient records is outside the scope of this specification (though see [The Smart App Launch Implementation Guide](http://hl7.org/fhir/smart-app-launch). \n\nWhen this operation is used to access multiple patient records at once, the return bundle could be rather a lot of data; servers may choose to require that such requests are made [asynchronously](async.html), and associated with [bulk data formats](formats.html#bulk). Alternatively, clients may choose to page through the result set (or servers may require this). Paging through the results is done the same as for [Searching](http.html#paging), using the [_count](search.html#count) parameter, and Bundle links. Implementers should note that paging will be slower than simply returning all the results at once (more network traffic, multiple latency delays) but may be required in order not to exhaust available memory reading or writing the whole response in a single package. Unlike searching, there is no inherent user-display order for the $everything operation. Servers might consider sorting the returned resources in descending order of last record update, but are not required to do so.\n\nThe _since parameter is provided to support periodic queries to get additional information that has changed about the patient since the last query. This means that the _since parameter is based on record time. The value of the _since parameter should be set to the time from the server. If using direct response, this is the timestamp in the response header. If using the async interface, this is the transaction timestamp in the json response. Servers should ensure that the timestamps a managed such that the client does not miss any changes. Clients should be able to handle getting the same response more than once in the case that the transaction falls on a time boundary. Clients should ensure that the other query parameters are constant to ensure a coherent set of records when doing periodic queries.",
    resource: ["Patient"],
    system: false,
    type: true,
    instance: true,
    parameter: [
      {
        name: "start",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The date range relates to care dates, not record currency dates - e.g. all records relating to care provided in a certain date range. If no start date is provided, all records prior to the end date are in scope.",
        type: "date",
      },
      {
        name: "end",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The date range relates to care dates, not record currency dates - e.g. all records relating to care provided in a certain date range. If no end date is provided, all records subsequent to the start date are in scope.",
        type: "date",
      },
      {
        name: "_since",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "Resources updated after this period will be included in the response. The intent of this parameter is to allow a client to request only records that have changed since the last request, based on either the return header time, or or (for asynchronous use), the transaction time",
        type: "instant",
      },
      {
        name: "_type",
        use: "in",
        min: 0,
        max: "*",
        documentation:
          "One or more parameters, each containing one or more comma-delimited FHIR resource types to include in the return resources. In the absence of any specified types, the server returns all resource types",
        type: "code",
      },
      {
        name: "_count",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "See discussion below on the utility of paging through the results of the $everything operation",
        type: "integer",
      },
      {
        name: "return",
        use: "out",
        min: 1,
        max: "1",
        documentation: 'The bundle type is "searchset"',
        type: "Bundle",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace PatientMatch {
  export type Input = {
    resource: fhirTypes.Resource;
    onlyCertainMatches?: fhirTypes.boolean;
    count?: fhirTypes.integer;
  };
  export type Output = fhirTypes.Bundle;
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "Patient-match",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Find patient matches using MPI based logic</h2><p>OPERATION: Find patient matches using MPI based logic</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/Patient-match</pre><div><p>A Master Patient Index (<a href="http://en.wikipedia.org/wiki/Enterprise_master_patient_index">MPI</a> ) is a service used to manage patient identification in a context where multiple patient databases exist. Healthcare applications and middleware use the MPI to match patients between the databases, and to store new patient details as they are encountered. MPIs are highly specialized applications, often tailored extensively to the institution\'s particular mix of patients. MPIs can also be run on a regional and national basis.</p>\n<p>To ask an MPI to match a patient, clients use the &quot;$match&quot; operation, which accepts a patient resource which may be only partially complete. The data provided is interpreted as an MPI input and processed by an algorithm of some kind that uses the data to determine the most appropriate matches in the patient set.  Note that different MPI matching algorithms have different required inputs. The generic $match operation does not specify any particular algorithm, nor a minimum set of information that must be provided when asking for an MPI match operation to be performed, but many implementations will have a set of minimum information, which may be declared in their definition of the $match operation by specifying a profile on the resource parameter, indicating which properties are required in the search.  The patient resource submitted to the operation does not have to be complete, nor does it need to pass validation (i.e. mandatory fields don\'t need to be populated), but it does have to be a valid instance, as it is used as the reference data to match against.</p>\n</div><p>URL: [base]/Patient/$match</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>resource</td><td>1..1</td><td><a href="resource.html">Resource</a></td><td/><td><div><p>Use this to provide an entire set of patient details for the MPI to match against (e.g. POST a patient record to Patient/$match).</p>\n</div></td></tr><tr><td>IN</td><td>onlyCertainMatches</td><td>0..1</td><td><a href="datatypes.html#boolean">boolean</a></td><td/><td><div><p>If there are multiple potential matches, then the match should not return the results with this flag set to true.  When false, the server may return multiple results with each result graded accordingly.</p>\n</div></td></tr><tr><td>IN</td><td>count</td><td>0..1</td><td><a href="datatypes.html#integer">integer</a></td><td/><td><div><p>The maximum number of records to return. If no value is provided, the server decides how many matches to return. Note that clients should be careful when using this, as it may prevent probable - and valid - matches from being returned</p>\n</div></td></tr><tr><td>OUT</td><td>return</td><td>1..1</td><td><a href="bundle.html">Bundle</a></td><td/><td><div><p>A bundle contain a set of Patient records that represent possible matches, optionally it may also contain an OperationOutcome with further information about the search results (such as warnings or information messages, such as a count of records that were close but eliminated)  If the operation was unsuccessful, then an OperationOutcome may be returned along with a BadRequest status Code (e.g. security issue, or insufficient properties in patient fragment - check against profile)</p>\n</div></td></tr></table><div><p>The response from an &quot;mpi&quot; query is a bundle containing patient records, ordered from most likely to least likely. If there are no patient matches, the MPI SHALL return an empty search set with no error, but may include an operation outcome with further advice regarding patient selection. All patient records SHALL have a search score from 0 to 1, where 1 is the most certain match, along with an extension &quot;<a href="extension-match-grade.html">match-grade</a>&quot; that indicates the MPI\'s position on the match quality.</p>\n</div></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 5,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/Patient-match",
    version: "4.0.1",
    name: "Find patient matches using MPI based logic",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "A Master Patient Index ([MPI](http://en.wikipedia.org/wiki/Enterprise_master_patient_index) ) is a service used to manage patient identification in a context where multiple patient databases exist. Healthcare applications and middleware use the MPI to match patients between the databases, and to store new patient details as they are encountered. MPIs are highly specialized applications, often tailored extensively to the institution's particular mix of patients. MPIs can also be run on a regional and national basis.  \n\nTo ask an MPI to match a patient, clients use the \"$match\" operation, which accepts a patient resource which may be only partially complete. The data provided is interpreted as an MPI input and processed by an algorithm of some kind that uses the data to determine the most appropriate matches in the patient set.  Note that different MPI matching algorithms have different required inputs. The generic $match operation does not specify any particular algorithm, nor a minimum set of information that must be provided when asking for an MPI match operation to be performed, but many implementations will have a set of minimum information, which may be declared in their definition of the $match operation by specifying a profile on the resource parameter, indicating which properties are required in the search.  The patient resource submitted to the operation does not have to be complete, nor does it need to pass validation (i.e. mandatory fields don't need to be populated), but it does have to be a valid instance, as it is used as the reference data to match against.",
    code: "match",
    comment:
      'The response from an "mpi" query is a bundle containing patient records, ordered from most likely to least likely. If there are no patient matches, the MPI SHALL return an empty search set with no error, but may include an operation outcome with further advice regarding patient selection. All patient records SHALL have a search score from 0 to 1, where 1 is the most certain match, along with an extension "[match-grade](extension-match-grade.html)" that indicates the MPI\'s position on the match quality.',
    resource: ["Patient"],
    system: false,
    type: true,
    instance: false,
    parameter: [
      {
        name: "resource",
        use: "in",
        min: 1,
        max: "1",
        documentation:
          "Use this to provide an entire set of patient details for the MPI to match against (e.g. POST a patient record to Patient/$match).",
        type: "Resource",
      },
      {
        name: "onlyCertainMatches",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "If there are multiple potential matches, then the match should not return the results with this flag set to true.  When false, the server may return multiple results with each result graded accordingly.",
        type: "boolean",
      },
      {
        name: "count",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The maximum number of records to return. If no value is provided, the server decides how many matches to return. Note that clients should be careful when using this, as it may prevent probable - and valid - matches from being returned",
        type: "integer",
      },
      {
        name: "return",
        use: "out",
        min: 1,
        max: "1",
        documentation:
          "A bundle contain a set of Patient records that represent possible matches, optionally it may also contain an OperationOutcome with further information about the search results (such as warnings or information messages, such as a count of records that were close but eliminated)  If the operation was unsuccessful, then an OperationOutcome may be returned along with a BadRequest status Code (e.g. security issue, or insufficient properties in patient fragment - check against profile)",
        type: "Bundle",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace PlanDefinitionApply {
  export type Input = {
    planDefinition?: fhirTypes.PlanDefinition;
    subject: Array<fhirTypes.string>;
    encounter?: fhirTypes.string;
    practitioner?: fhirTypes.string;
    organization?: fhirTypes.string;
    userType?: fhirTypes.CodeableConcept;
    userLanguage?: fhirTypes.CodeableConcept;
    userTaskContext?: fhirTypes.CodeableConcept;
    setting?: fhirTypes.CodeableConcept;
    settingContext?: fhirTypes.CodeableConcept;
  };
  export type Output = fhirTypes.CarePlan;
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "PlanDefinition-apply",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Apply</h2><p>OPERATION: Apply</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/PlanDefinition-apply</pre><div><p>The apply operation applies a PlanDefinition to a given context</p>\n</div><p>URL: [base]/PlanDefinition/$apply</p><p>URL: [base]/PlanDefinition/[id]/$apply</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>planDefinition</td><td>0..1</td><td><a href="plandefinition.html">PlanDefinition</a></td><td/><td><div><p>The plan definition to be applied. If the operation is invoked at the instance level, this parameter is not allowed; if the operation is invoked at the type level, this parameter is required</p>\n</div></td></tr><tr><td>IN</td><td>subject</td><td>1..*</td><td><a href="datatypes.html#string">string</a><br/>(<a href="search.html#reference">reference</a>)</td><td/><td><div><p>The subject(s) that is/are the target of the plan to be applied. The subject may be a Patient, Practitioner, Organization, Location, Device, or Group. Subjects provided in this parameter will be resolved as the subject of the PlanDefinition based on the type of the subject. If multiple subjects of the same type are provided, the behavior is implementation-defined</p>\n</div></td></tr><tr><td>IN</td><td>encounter</td><td>0..1</td><td><a href="datatypes.html#string">string</a><br/>(<a href="search.html#reference">reference</a>)</td><td/><td><div><p>The encounter in context, if any</p>\n</div></td></tr><tr><td>IN</td><td>practitioner</td><td>0..1</td><td><a href="datatypes.html#string">string</a><br/>(<a href="search.html#reference">reference</a>)</td><td/><td><div><p>The practitioner applying the plan definition</p>\n</div></td></tr><tr><td>IN</td><td>organization</td><td>0..1</td><td><a href="datatypes.html#string">string</a><br/>(<a href="search.html#reference">reference</a>)</td><td/><td><div><p>The organization applying the plan definition</p>\n</div></td></tr><tr><td>IN</td><td>userType</td><td>0..1</td><td><a href="datatypes.html#CodeableConcept">CodeableConcept</a></td><td/><td><div><p>The type of user initiating the request, e.g. patient, healthcare provider, or specific type of healthcare provider (physician, nurse, etc.)</p>\n</div></td></tr><tr><td>IN</td><td>userLanguage</td><td>0..1</td><td><a href="datatypes.html#CodeableConcept">CodeableConcept</a></td><td/><td><div><p>Preferred language of the person using the system</p>\n</div></td></tr><tr><td>IN</td><td>userTaskContext</td><td>0..1</td><td><a href="datatypes.html#CodeableConcept">CodeableConcept</a></td><td/><td><div><p>The task the system user is performing, e.g. laboratory results review, medication list review, etc. This information can be used to tailor decision support outputs, such as recommended information resources</p>\n</div></td></tr><tr><td>IN</td><td>setting</td><td>0..1</td><td><a href="datatypes.html#CodeableConcept">CodeableConcept</a></td><td/><td><div><p>The current setting of the request (inpatient, outpatient, etc.)</p>\n</div></td></tr><tr><td>IN</td><td>settingContext</td><td>0..1</td><td><a href="datatypes.html#CodeableConcept">CodeableConcept</a></td><td/><td><div><p>Additional detail about the setting of the request, if any</p>\n</div></td></tr><tr><td>OUT</td><td>return</td><td>1..1</td><td><a href="careplan.html">CarePlan</a></td><td/><td><div><p>The CarePlan that is the result of applying the plan definition</p>\n</div></td></tr></table><div><p>The result of this operation is a CarePlan resource with a single activity represented by a RequestGroup. The RequestGroup will have actions for each of the applicable actions in the plan based on evaluating the applicability condition in context. For each applicable action, the definition is applied as described in the $apply operation of the ActivityDefinition resource, and the resulting resource is added as an activity to the CarePlan. If the ActivityDefinition includes library references, those libraries will be available to the evaluated expressions. If those libraries have parameters, those parameters will be bound by name to the parameters given to the operation. In addition, parameters to the $apply operation are available within dynamicValue expressions as context variables, accessible by the name of the parameter, prefixed with a percent (%) symbol. For a more detailed description, refer to the PlanDefinition and ActivityDefinition resource documentation</p>\n</div></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 2,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/PlanDefinition-apply",
    version: "4.0.1",
    name: "Apply",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "The apply operation applies a PlanDefinition to a given context",
    code: "apply",
    comment:
      "The result of this operation is a CarePlan resource with a single activity represented by a RequestGroup. The RequestGroup will have actions for each of the applicable actions in the plan based on evaluating the applicability condition in context. For each applicable action, the definition is applied as described in the $apply operation of the ActivityDefinition resource, and the resulting resource is added as an activity to the CarePlan. If the ActivityDefinition includes library references, those libraries will be available to the evaluated expressions. If those libraries have parameters, those parameters will be bound by name to the parameters given to the operation. In addition, parameters to the $apply operation are available within dynamicValue expressions as context variables, accessible by the name of the parameter, prefixed with a percent (%) symbol. For a more detailed description, refer to the PlanDefinition and ActivityDefinition resource documentation",
    resource: ["PlanDefinition"],
    system: false,
    type: true,
    instance: true,
    parameter: [
      {
        name: "planDefinition",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The plan definition to be applied. If the operation is invoked at the instance level, this parameter is not allowed; if the operation is invoked at the type level, this parameter is required",
        type: "PlanDefinition",
      },
      {
        name: "subject",
        use: "in",
        min: 1,
        max: "*",
        documentation:
          "The subject(s) that is/are the target of the plan to be applied. The subject may be a Patient, Practitioner, Organization, Location, Device, or Group. Subjects provided in this parameter will be resolved as the subject of the PlanDefinition based on the type of the subject. If multiple subjects of the same type are provided, the behavior is implementation-defined",
        type: "string",
        searchType: "reference",
      },
      {
        name: "encounter",
        use: "in",
        min: 0,
        max: "1",
        documentation: "The encounter in context, if any",
        type: "string",
        searchType: "reference",
      },
      {
        name: "practitioner",
        use: "in",
        min: 0,
        max: "1",
        documentation: "The practitioner applying the plan definition",
        type: "string",
        searchType: "reference",
      },
      {
        name: "organization",
        use: "in",
        min: 0,
        max: "1",
        documentation: "The organization applying the plan definition",
        type: "string",
        searchType: "reference",
      },
      {
        name: "userType",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The type of user initiating the request, e.g. patient, healthcare provider, or specific type of healthcare provider (physician, nurse, etc.)",
        type: "CodeableConcept",
      },
      {
        name: "userLanguage",
        use: "in",
        min: 0,
        max: "1",
        documentation: "Preferred language of the person using the system",
        type: "CodeableConcept",
      },
      {
        name: "userTaskContext",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The task the system user is performing, e.g. laboratory results review, medication list review, etc. This information can be used to tailor decision support outputs, such as recommended information resources",
        type: "CodeableConcept",
      },
      {
        name: "setting",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The current setting of the request (inpatient, outpatient, etc.)",
        type: "CodeableConcept",
      },
      {
        name: "settingContext",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "Additional detail about the setting of the request, if any",
        type: "CodeableConcept",
      },
      {
        name: "return",
        use: "out",
        min: 1,
        max: "1",
        documentation:
          "The CarePlan that is the result of applying the plan definition",
        type: "CarePlan",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace PlanDefinitionDataRequirements {
  export type Input = Record<string, never>;
  export type Output = fhirTypes.Library;
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "PlanDefinition-data-requirements",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Data Requirements</h2><p>OPERATION: Data Requirements</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/PlanDefinition-data-requirements</pre><div><p>The data-requirements operation aggregates and returns the parameters and data requirements for the plan definition and all its dependencies as a single module definition library</p>\n</div><p>URL: [base]/PlanDefinition/[id]/$data-requirements</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>OUT</td><td>return</td><td>1..1</td><td><a href="library.html">Library</a></td><td/><td><div><p>The result of the requirements gathering is a module-definition Library that describes the aggregate parameters, data requirements, and dependencies of the plan definition</p>\n</div></td></tr></table><div><p>The effect of invoking this operation is to determine the aggregate set of data requirements and dependencies for the plan definition. The result is a Library resource with a type of module-definition that contains all the parameter definitions and data requirements of the plan definition and any libraries referenced by it. Implementations SHOULD aggregate data requirements intelligently (i.e. by collapsing overlapping data requirements)</p>\n</div></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 2,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/PlanDefinition-data-requirements",
    version: "4.0.1",
    name: "Data Requirements",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "The data-requirements operation aggregates and returns the parameters and data requirements for the plan definition and all its dependencies as a single module definition library",
    code: "data-requirements",
    comment:
      "The effect of invoking this operation is to determine the aggregate set of data requirements and dependencies for the plan definition. The result is a Library resource with a type of module-definition that contains all the parameter definitions and data requirements of the plan definition and any libraries referenced by it. Implementations SHOULD aggregate data requirements intelligently (i.e. by collapsing overlapping data requirements)",
    resource: ["PlanDefinition"],
    system: false,
    type: false,
    instance: true,
    parameter: [
      {
        name: "return",
        use: "out",
        min: 1,
        max: "1",
        documentation:
          "The result of the requirements gathering is a module-definition Library that describes the aggregate parameters, data requirements, and dependencies of the plan definition",
        type: "Library",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace ResourceConvert {
  export type Input = { input: fhirTypes.Resource };
  export type Output = { output: fhirTypes.Resource };
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "Resource-convert",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Convert from one form to another</h2><p>OPERATION: Convert from one form to another</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/Resource-convert</pre><div><p>This operation takes a resource in one form, and returns to in another form. Both input and output are a single resource. The primary use of this operation is to convert between formats (e.g. (XML -&gt; JSON or vice versa)</p>\n</div><p>URL: [base]/$convert</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>input</td><td>1..1</td><td><a href="resource.html">Resource</a></td><td/><td><div><p>The resource that is to be converted</p>\n</div></td></tr><tr><td>OUT</td><td>output</td><td>1..1</td><td><a href="resource.html">Resource</a></td><td/><td><div><p>The resource after conversion</p>\n</div></td></tr></table><div><p>While the primary use of this operation is simple - converting a resource from one format to another, there are many potential uses including:</p>\n<ul>\n<li>converting resources from one version to another</li>\n<li>restructuring information in a resource (e.g. moving method into/out of Observation.code)</li>\n<li>extracting data from a questionnaire</li>\n<li>converting CDA documents or v2 messages (as a binary resource) to a bundle (or vice versa) (or even openEHR or openMHealth).</li>\n</ul>\n<p>These variants would all be associated with parameters that define and control these kind of conversions, though such parameters are not defined at this time. In the absence of any parameters, simple format conversion is all that will occur.</p>\n<p>For this reason, implementers should be aware that:</p>\n<ul>\n<li>the output resource type may be different from the input resource (particularly, it might be a bundle)</li>\n<li>binary resources may be represented directly using some other content-type (i.e. just post the content directly)</li>\n</ul>\n<p>Implementers are encouraged to provide feedback to HL7 about their use of this operation</p>\n</div></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 1,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "draft",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/Resource-convert",
    version: "4.0.1",
    name: "Convert from one form to another",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "This operation takes a resource in one form, and returns to in another form. Both input and output are a single resource. The primary use of this operation is to convert between formats (e.g. (XML -> JSON or vice versa)",
    code: "convert",
    comment:
      "While the primary use of this operation is simple - converting a resource from one format to another, there are many potential uses including:\n\n* converting resources from one version to another\n* restructuring information in a resource (e.g. moving method into/out of Observation.code)\n* extracting data from a questionnaire\n* converting CDA documents or v2 messages (as a binary resource) to a bundle (or vice versa) (or even openEHR or openMHealth). \n\nThese variants would all be associated with parameters that define and control these kind of conversions, though such parameters are not defined at this time. In the absence of any parameters, simple format conversion is all that will occur.\n\nFor this reason, implementers should be aware that:\n\n* the output resource type may be different from the input resource (particularly, it might be a bundle)\n* binary resources may be represented directly using some other content-type (i.e. just post the content directly)\n\nImplementers are encouraged to provide feedback to HL7 about their use of this operation",
    resource: ["Resource"],
    system: true,
    type: false,
    instance: false,
    parameter: [
      {
        name: "input",
        use: "in",
        min: 1,
        max: "1",
        documentation: "The resource that is to be converted",
        type: "Resource",
      },
      {
        name: "output",
        use: "out",
        min: 1,
        max: "1",
        documentation: "The resource after conversion",
        type: "Resource",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace ResourceGraph {
  export type Input = { graph: fhirTypes.uri };
  export type Output = { result: fhirTypes.Bundle };
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "Resource-graph",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Return a graph of resources</h2><p>OPERATION: Return a graph of resources</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/Resource-graph</pre><div><p>Return an entire graph of resources based on a <a href="graphdefinition.html">GraphDefinition</a>. The operation is invoked on a specific instance of a resource, and the graph definition tells the server what other resources to return in the same packaage</p>\n</div><p>URL: [base]/Resource/[id]/$graph</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>graph</td><td>1..1</td><td><a href="datatypes.html#uri">uri</a></td><td/><td><div><p>Servers MAY choose to allow any graph definition to be specified, but MAY require that the client choose a graph definition from a specific list of known supported definitions. The server is not required to support a formal definition of the graph on the end point</p>\n</div></td></tr><tr><td>OUT</td><td>result</td><td>1..1</td><td><a href="bundle.html">Bundle</a></td><td/><td><div><p>The set of resources that were in the graph based on the provided definition</p>\n</div></td></tr></table><div/></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 1,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/Resource-graph",
    version: "4.0.1",
    name: "Return a graph of resources",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "Return an entire graph of resources based on a [GraphDefinition](graphdefinition.html). The operation is invoked on a specific instance of a resource, and the graph definition tells the server what other resources to return in the same packaage",
    code: "graph",
    resource: ["Resource"],
    system: false,
    type: false,
    instance: true,
    parameter: [
      {
        name: "graph",
        use: "in",
        min: 1,
        max: "1",
        documentation:
          "Servers MAY choose to allow any graph definition to be specified, but MAY require that the client choose a graph definition from a specific list of known supported definitions. The server is not required to support a formal definition of the graph on the end point",
        type: "uri",
      },
      {
        name: "result",
        use: "out",
        min: 1,
        max: "1",
        documentation:
          "The set of resources that were in the graph based on the provided definition",
        type: "Bundle",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace ResourceGraphql {
  export type Input = { query: fhirTypes.string };
  export type Output = { result: fhirTypes.Binary };
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "Resource-graphql",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Execute a graphql statement</h2><p>OPERATION: Execute a graphql statement</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/Resource-graphql</pre><div><p>Execute a graphql statement on a since resource or against the entire system. See the <a href="graphql.html">Using GraphQL with FHIR</a> page for further details.</p>\n<p>For the purposes of graphQL compatibility, this operation can also be invoked using a POST with the graphQL as the body, or a JSON body (see <a href="http://graphql.org/">graphQL spec</a> for details)</p>\n</div><p>URL: [base]/$graphql</p><p>URL: [base]/Resource/[id]/$graphql</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>query</td><td>1..1</td><td><a href="datatypes.html#string">string</a></td><td/><td/></tr><tr><td>OUT</td><td>result</td><td>1..1</td><td><a href="binary.html">Binary</a></td><td/><td><div><p>The content is always returned as application/json; this SHOULD be specified in the Accept header</p>\n</div></td></tr></table><div/></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 1,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/Resource-graphql",
    version: "4.0.1",
    name: "Execute a graphql statement",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "Execute a graphql statement on a since resource or against the entire system. See the [Using GraphQL with FHIR](graphql.html) page for further details.\n\nFor the purposes of graphQL compatibility, this operation can also be invoked using a POST with the graphQL as the body, or a JSON body (see [graphQL spec](http://graphql.org/) for details)",
    code: "graphql",
    resource: ["Resource"],
    system: true,
    type: false,
    instance: true,
    parameter: [
      { name: "query", use: "in", min: 1, max: "1", type: "string" },
      {
        name: "result",
        use: "out",
        min: 1,
        max: "1",
        documentation:
          "The content is always returned as application/json; this SHOULD be specified in the Accept header",
        type: "Binary",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace ResourceMeta {
  export type Input = Record<string, never>;
  export type Output = { return: fhirTypes.Meta };
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "Resource-meta",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Access a list of profiles, tags, and security labels</h2><p>OPERATION: Access a list of profiles, tags, and security labels</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/Resource-meta</pre><div><p>This operation retrieves a summary of the profiles, tags, and security labels for the given scope; e.g. for each scope:</p>\n<ul>\n<li>system-wide: a list of all profiles, tags and security labels in use by the system</li>\n<li>resource-type level: A list of all profiles, tags, and security labels for the resource type</li>\n<li>individual resource level: A list of all profiles, tags, and security labels for the current version of the resource.  Also, as a special case, this operation (and other meta operations) can be performed on a historical version of a resource)</li>\n</ul>\n</div><p>URL: [base]/$meta</p><p>URL: [base]/Resource/$meta</p><p>URL: [base]/Resource/[id]/$meta</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>OUT</td><td>return</td><td>1..1</td><td><a href="resource.html#Meta">Meta</a></td><td/><td><div><p>The meta returned by the operation</p>\n</div></td></tr></table><div><p>At the system and type levels, the $meta operation is used to get a summary of all the labels that are in use across the system. The principal use for this operation is to support search e.g. what tags can be searched for. At these levels, the meta will not contain versionId, lastUpdated etc. Systems are not obligated to implement the operation at this level (and should return a 4xx error if they don\'t). At the resource and historical entry level, the $meta operation returns the same meta as would be returned by accessing the resource directly. This can be used to allow a system to get access to the meta-information for the resource without accessing the resource itself, e.g. for security reasons</p>\n</div></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 3,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/Resource-meta",
    version: "4.0.1",
    name: "Access a list of profiles, tags, and security labels",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "This operation retrieves a summary of the profiles, tags, and security labels for the given scope; e.g. for each scope:  \n\n* system-wide: a list of all profiles, tags and security labels in use by the system \n* resource-type level: A list of all profiles, tags, and security labels for the resource type \n* individual resource level: A list of all profiles, tags, and security labels for the current version of the resource.  Also, as a special case, this operation (and other meta operations) can be performed on a historical version of a resource)",
    code: "meta",
    comment:
      "At the system and type levels, the $meta operation is used to get a summary of all the labels that are in use across the system. The principal use for this operation is to support search e.g. what tags can be searched for. At these levels, the meta will not contain versionId, lastUpdated etc. Systems are not obligated to implement the operation at this level (and should return a 4xx error if they don't). At the resource and historical entry level, the $meta operation returns the same meta as would be returned by accessing the resource directly. This can be used to allow a system to get access to the meta-information for the resource without accessing the resource itself, e.g. for security reasons",
    resource: ["Resource"],
    system: true,
    type: true,
    instance: true,
    parameter: [
      {
        name: "return",
        use: "out",
        min: 1,
        max: "1",
        documentation: "The meta returned by the operation",
        type: "Meta",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace ResourceMetaAdd {
  export type Input = { meta: fhirTypes.Meta };
  export type Output = { return: fhirTypes.Meta };
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "Resource-meta-add",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Add profiles, tags, and security labels to a resource</h2><p>OPERATION: Add profiles, tags, and security labels to a resource</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/Resource-meta-add</pre><div><p>This operation takes a meta, and adds the profiles, tags, and security labels found in it to the nominated resource</p>\n</div><p>URL: [base]/Resource/[id]/$meta-add</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>meta</td><td>1..1</td><td><a href="resource.html#Meta">Meta</a></td><td/><td><div><p>Profiles, tags, and security labels to add to the existing resource. Note that profiles, tags, and security labels are sets, and duplicates are not created.  The identity of a tag or security label is the system+code. When matching existing tags during adding, version and display are ignored. For profiles, matching is based on the full URL</p>\n</div></td></tr><tr><td>OUT</td><td>return</td><td>1..1</td><td><a href="resource.html#Meta">Meta</a></td><td/><td><div><p>Resulting meta for the resource</p>\n</div></td></tr></table><div/></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 3,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/Resource-meta-add",
    version: "4.0.1",
    name: "Add profiles, tags, and security labels to a resource",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "This operation takes a meta, and adds the profiles, tags, and security labels found in it to the nominated resource",
    code: "meta-add",
    resource: ["Resource"],
    system: false,
    type: false,
    instance: true,
    parameter: [
      {
        name: "meta",
        use: "in",
        min: 1,
        max: "1",
        documentation:
          "Profiles, tags, and security labels to add to the existing resource. Note that profiles, tags, and security labels are sets, and duplicates are not created.  The identity of a tag or security label is the system+code. When matching existing tags during adding, version and display are ignored. For profiles, matching is based on the full URL",
        type: "Meta",
      },
      {
        name: "return",
        use: "out",
        min: 1,
        max: "1",
        documentation: "Resulting meta for the resource",
        type: "Meta",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace ResourceMetaDelete {
  export type Input = { meta: fhirTypes.Meta };
  export type Output = { return: fhirTypes.Meta };
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "Resource-meta-delete",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Delete profiles, tags, and security labels for a resource</h2><p>OPERATION: Delete profiles, tags, and security labels for a resource</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/Resource-meta-delete</pre><div><p>This operation takes a meta, and deletes the profiles, tags, and security labels found in it from the nominated resource</p>\n</div><p>URL: [base]/Resource/[id]/$meta-delete</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>meta</td><td>1..1</td><td><a href="resource.html#Meta">Meta</a></td><td/><td><div><p>Profiles, tags, and security labels to delete from the existing resource. It is not an error if these tags, profiles, and labels do not exist.  The identity of a tag or security label is the system+code. When matching existing tags during deletion, version and display are ignored. For profiles, matching is based on the full URL</p>\n</div></td></tr><tr><td>OUT</td><td>return</td><td>1..1</td><td><a href="resource.html#Meta">Meta</a></td><td/><td><div><p>Resulting meta for the resource</p>\n</div></td></tr></table><div/></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 3,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/Resource-meta-delete",
    version: "4.0.1",
    name: "Delete profiles, tags, and security labels for a resource",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "This operation takes a meta, and deletes the profiles, tags, and security labels found in it from the nominated resource",
    code: "meta-delete",
    resource: ["Resource"],
    system: false,
    type: false,
    instance: true,
    parameter: [
      {
        name: "meta",
        use: "in",
        min: 1,
        max: "1",
        documentation:
          "Profiles, tags, and security labels to delete from the existing resource. It is not an error if these tags, profiles, and labels do not exist.  The identity of a tag or security label is the system+code. When matching existing tags during deletion, version and display are ignored. For profiles, matching is based on the full URL",
        type: "Meta",
      },
      {
        name: "return",
        use: "out",
        min: 1,
        max: "1",
        documentation: "Resulting meta for the resource",
        type: "Meta",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace ResourceValidate {
  export type Input = {
    resource?: fhirTypes.Resource;
    mode?: fhirTypes.code;
    profile?: fhirTypes.uri;
  };
  export type Output = fhirTypes.OperationOutcome;
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "Resource-validate",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Validate a resource</h2><p>OPERATION: Validate a resource</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/Resource-validate</pre><div><p>The validate operation checks whether the attached content would be acceptable either generally, as a create, an update or as a delete to an existing resource.  The action the server takes depends on the mode parameter:</p>\n<ul>\n<li>[mode not provided]: The server checks the content of the resource against any schema, constraint rules, and other general terminology rules</li>\n<li>create: The server checks the content, and then checks that the content would be acceptable as a create (e.g. that the content would not violate any uniqueness constraints)</li>\n<li>update: The server checks the content, and then checks that it would accept it as an update against the nominated specific resource (e.g. that there are no changes to immutable fields the server does not allow to change, and checking version integrity if appropriate)</li>\n<li>delete: The server ignores the content, and checks that the nominated resource is allowed to be deleted (e.g. checking referential integrity rules)</li>\n</ul>\n<p>Modes update and delete can only be used when the operation is invoked at the resource instance level.   The return from this operation is an <a href="operationoutcome.html">OperationOutcome</a></p>\n<p>Note that this operation is not the only way to validate resources - see <a href="validation.html">Validating Resources</a> for further information.</p>\n</div><p>URL: [base]/Resource/$validate</p><p>URL: [base]/Resource/[id]/$validate</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>resource</td><td>0..1</td><td><a href="resource.html">Resource</a></td><td/><td><div><p>Must be present unless the mode is &quot;delete&quot;</p>\n</div></td></tr><tr><td>IN</td><td>mode</td><td>0..1</td><td><a href="datatypes.html#code">code</a></td><td><a href="valueset-resource-validation-mode.html">http://hl7.org/fhir/ValueSet/resource-validation-mode|4.0.1</a> (Required)</td><td><div><p>Default is \'no action\'; (e.g. general validation)</p>\n</div></td></tr><tr><td>IN</td><td>profile</td><td>0..1</td><td><a href="datatypes.html#uri">uri</a></td><td/><td><div><p>If this is nominated, then the resource is validated against this specific profile. If a profile is nominated, and the server cannot validate against the nominated profile, it SHALL return an error</p>\n</div></td></tr><tr><td>OUT</td><td>return</td><td>1..1</td><td><a href="operationoutcome.html">OperationOutcome</a></td><td/><td><div><p>If the operation outcome does not list any errors, and a mode was specified, then this is an indication that the operation would be expected to succeed (excepting for transactional integrity issues, see below)</p>\n</div></td></tr></table><div><p>This operation may be used during design and development to validate application design. It can also be used at run-time. One possible use might be that a client asks the server whether a proposed update is valid as the user is editing a dialog and displays an updated error to the user. The operation can be used as part of a light-weight two phase commit protocol but there is no expectation that the server will hold the content of the resource after this operation is used, or that the server guarantees to successfully perform an actual create, update or delete after the validation operation completes.</p>\n<p>This operation returns a 200 OK whether or not the resource is valid. A 4xx or 5xx error means that the validation itself could not be performed, and it is unknown whether the resource is valid or not.</p>\n<p>Note: the correct behaviour of validation with regard to language (especially for Coding.display) is currently undefined, and further development and testing may lead to specific requirements or recommendations in subsequent releases</p>\n<p>Future versions of this specifcation may add additional validation parameters. A candidate list is maintained with the <a href="https://confluence.hl7.org/display/FHIR/Using+the+FHIR+Validator">FHIR Validator Documentation</a></p>\n</div></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 5,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "normative",
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-normative-version",
        valueCode: "4.0.1",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/Resource-validate",
    version: "4.0.1",
    name: "Validate a resource",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "The validate operation checks whether the attached content would be acceptable either generally, as a create, an update or as a delete to an existing resource.  The action the server takes depends on the mode parameter:  \n\n* [mode not provided]: The server checks the content of the resource against any schema, constraint rules, and other general terminology rules \n* create: The server checks the content, and then checks that the content would be acceptable as a create (e.g. that the content would not violate any uniqueness constraints) \n* update: The server checks the content, and then checks that it would accept it as an update against the nominated specific resource (e.g. that there are no changes to immutable fields the server does not allow to change, and checking version integrity if appropriate) \n* delete: The server ignores the content, and checks that the nominated resource is allowed to be deleted (e.g. checking referential integrity rules)  \n\nModes update and delete can only be used when the operation is invoked at the resource instance level.   The return from this operation is an [OperationOutcome](operationoutcome.html)\n\nNote that this operation is not the only way to validate resources - see [Validating Resources](validation.html) for further information.",
    code: "validate",
    comment:
      "This operation may be used during design and development to validate application design. It can also be used at run-time. One possible use might be that a client asks the server whether a proposed update is valid as the user is editing a dialog and displays an updated error to the user. The operation can be used as part of a light-weight two phase commit protocol but there is no expectation that the server will hold the content of the resource after this operation is used, or that the server guarantees to successfully perform an actual create, update or delete after the validation operation completes.\n\nThis operation returns a 200 OK whether or not the resource is valid. A 4xx or 5xx error means that the validation itself could not be performed, and it is unknown whether the resource is valid or not.\n\nNote: the correct behaviour of validation with regard to language (especially for Coding.display) is currently undefined, and further development and testing may lead to specific requirements or recommendations in subsequent releases\n\nFuture versions of this specifcation may add additional validation parameters. A candidate list is maintained with the [FHIR Validator Documentation](https://confluence.hl7.org/display/FHIR/Using+the+FHIR+Validator)",
    resource: ["Resource"],
    system: false,
    type: true,
    instance: true,
    parameter: [
      {
        name: "resource",
        use: "in",
        min: 0,
        max: "1",
        documentation: 'Must be present unless the mode is "delete"',
        type: "Resource",
      },
      {
        name: "mode",
        use: "in",
        min: 0,
        max: "1",
        documentation: "Default is 'no action'; (e.g. general validation)",
        type: "code",
        binding: {
          strength: "required",
          valueSet:
            "http://hl7.org/fhir/ValueSet/resource-validation-mode|4.0.1",
        },
      },
      {
        name: "profile",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "If this is nominated, then the resource is validated against this specific profile. If a profile is nominated, and the server cannot validate against the nominated profile, it SHALL return an error",
        type: "uri",
      },
      {
        name: "return",
        use: "out",
        min: 1,
        max: "1",
        documentation:
          "If the operation outcome does not list any errors, and a mode was specified, then this is an indication that the operation would be expected to succeed (excepting for transactional integrity issues, see below)",
        type: "OperationOutcome",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace StructureDefinitionQuestionnaire {
  export type Input = {
    identifier?: fhirTypes.canonical;
    profile?: fhirTypes.string;
    url?: fhirTypes.canonical;
    supportedOnly?: fhirTypes.boolean;
  };
  export type Output = fhirTypes.Questionnaire;
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "StructureDefinition-questionnaire",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Build Questionnaire</h2><p>OPERATION: Build Questionnaire</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/StructureDefinition-questionnaire</pre><div><p>Generates a <a href="questionnaire.html">Questionnaire</a> instance  based on a specified <a href="structuredefinition.html">StructureDefinition</a>, creating questions for each core element or extension element found in the <a href="structuredefinition.html">StructureDefinition</a>.</p>\n<p>If the operation is not called at the instance level, one of the <em>identifier</em>, <em>profile</em> or <em>url</em> \'in\' parameters must be provided. If more than one is specified, servers may raise an error or may resolve with the parameter of their choice. If called at the instance level, these parameters will be ignored. The response will contain a <a href="questionnaire.html">Questionnaire</a> instance based on the specified <a href="structuredefinition.html">StructureDefinition</a> and/or an <a href="operationoutcome.html">OperationOutcome</a> resource with errors or warnings.  Nested groups are used to handle complex structures and data types.  If the \'supportedOnly\' parameter is set to true, only those elements marked as &quot;must support&quot; will be included.</p>\n<p>This operation is intended to enable auto-generation of simple interfaces for arbitrary profiles.  The \'questionnaire\' approach to data entry has limitations that will make it less optimal than custom-defined interfaces.  However, this function may be useful for simple applications or for systems that wish to support &quot;non-core&quot; resources with minimal development effort.</p>\n</div><p>URL: [base]/StructureDefinition/$questionnaire</p><p>URL: [base]/StructureDefinition/[id]/$questionnaire</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>identifier</td><td>0..1</td><td><a href="datatypes.html#canonical">canonical</a></td><td/><td><div><p>A logical identifier (i.e. \'StructureDefinition.identifier\'\'). The server must know the StructureDefinition or be able to retrieve it from other known repositories.</p>\n</div></td></tr><tr><td>IN</td><td>profile</td><td>0..1</td><td><a href="datatypes.html#string">string</a><br/>(<a href="search.html#token">token</a>)</td><td/><td><div><p>The <a href="structuredefinition.html">StructureDefinition</a> is provided directly as part of the request. Servers may choose not to accept profiles in this fashion</p>\n</div></td></tr><tr><td>IN</td><td>url</td><td>0..1</td><td><a href="datatypes.html#canonical">canonical</a></td><td/><td><div><p>The StructureDefinition\'s official URL (i.e. \'StructureDefinition.url\'). The server must know the StructureDefinition or be able to retrieve it from other known repositories.</p>\n</div></td></tr><tr><td>IN</td><td>supportedOnly</td><td>0..1</td><td><a href="datatypes.html#boolean">boolean</a></td><td/><td><div><p>If true, the questionnaire will only include those elements marked as &quot;mustSupport=\'true\'&quot; in the StructureDefinition.</p>\n</div></td></tr><tr><td>OUT</td><td>return</td><td>1..1</td><td><a href="questionnaire.html">Questionnaire</a></td><td/><td><div><p>The questionnaire form generated based on the StructureDefinition.</p>\n</div></td></tr></table><div><p><strong>Open Issue</strong>: Ideally, extensions should be populated in the generated <a href="questionnaire.html">Questionnaire</a> that will support taking <a href="questionnaireresponse.html">QuestionnaireResponse</a> resources generated from the Questionnaire and turning them back into the appropriate resources.</p>\n</div></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 5,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/StructureDefinition-questionnaire",
    version: "4.0.1",
    name: "Build Questionnaire",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "Generates a [Questionnaire](questionnaire.html) instance  based on a specified [StructureDefinition](structuredefinition.html), creating questions for each core element or extension element found in the [StructureDefinition](structuredefinition.html).    \n\nIf the operation is not called at the instance level, one of the *identifier*, *profile* or *url* 'in' parameters must be provided. If more than one is specified, servers may raise an error or may resolve with the parameter of their choice. If called at the instance level, these parameters will be ignored. The response will contain a [Questionnaire](questionnaire.html) instance based on the specified [StructureDefinition](structuredefinition.html) and/or an [OperationOutcome](operationoutcome.html) resource with errors or warnings.  Nested groups are used to handle complex structures and data types.  If the 'supportedOnly' parameter is set to true, only those elements marked as \"must support\" will be included.  \n\nThis operation is intended to enable auto-generation of simple interfaces for arbitrary profiles.  The 'questionnaire' approach to data entry has limitations that will make it less optimal than custom-defined interfaces.  However, this function may be useful for simple applications or for systems that wish to support \"non-core\" resources with minimal development effort.",
    code: "questionnaire",
    comment:
      "**Open Issue**: Ideally, extensions should be populated in the generated [Questionnaire](questionnaire.html) that will support taking [QuestionnaireResponse](questionnaireresponse.html) resources generated from the Questionnaire and turning them back into the appropriate resources.",
    resource: ["StructureDefinition"],
    system: false,
    type: true,
    instance: true,
    parameter: [
      {
        name: "identifier",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "A logical identifier (i.e. 'StructureDefinition.identifier''). The server must know the StructureDefinition or be able to retrieve it from other known repositories.",
        type: "canonical",
        targetProfile: [
          "http://hl7.org/fhir/StructureDefinition/StructureDefinition",
        ],
      },
      {
        name: "profile",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The [StructureDefinition](structuredefinition.html) is provided directly as part of the request. Servers may choose not to accept profiles in this fashion",
        type: "string",
        searchType: "token",
      },
      {
        name: "url",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The StructureDefinition's official URL (i.e. 'StructureDefinition.url'). The server must know the StructureDefinition or be able to retrieve it from other known repositories.",
        type: "canonical",
        targetProfile: [
          "http://hl7.org/fhir/StructureDefinition/StructureDefinition",
        ],
      },
      {
        name: "supportedOnly",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "If true, the questionnaire will only include those elements marked as \"mustSupport='true'\" in the StructureDefinition.",
        type: "boolean",
      },
      {
        name: "return",
        use: "out",
        min: 1,
        max: "1",
        documentation:
          "The questionnaire form generated based on the StructureDefinition.",
        type: "Questionnaire",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace StructureDefinitionSnapshot {
  export type Input = {
    definition?: fhirTypes.StructureDefinition;
    url?: fhirTypes.string;
  };
  export type Output = fhirTypes.StructureDefinition;
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "StructureDefinition-snapshot",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Generate Snapshot</h2><p>OPERATION: Generate Snapshot</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/StructureDefinition-snapshot</pre><div><p>Generates a <a href="structuredefinition.html">StructureDefinition</a> instance  with  a snapshot, based on a differential in a specified <a href="structuredefinition.html">StructureDefinition</a>.</p>\n<p>If the operation is not called at the instance level, either <em>definition</em> or <em>url</em> \'in\' parameters must be provided. If more than one is specified, servers may raise an error or may resolve with the parameter of their choice. If called at the instance level, these parameters will be ignored.</p>\n</div><p>URL: [base]/StructureDefinition/$snapshot</p><p>URL: [base]/StructureDefinition/[id]/$snapshot</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>definition</td><td>0..1</td><td><a href="structuredefinition.html">StructureDefinition</a></td><td/><td><div><p>The <a href="structuredefinition.html">StructureDefinition</a> is provided directly as part of the request. Servers may choose not to accept profiles in this fashion</p>\n</div></td></tr><tr><td>IN</td><td>url</td><td>0..1</td><td><a href="datatypes.html#string">string</a><br/>(<a href="search.html#token">token</a>)</td><td/><td><div><p>The StructureDefinition\'s canonical URL (i.e. \'StructureDefinition.url\'). The server must know the structure definition, or be able to retrieve it from other known repositories.</p>\n</div></td></tr><tr><td>OUT</td><td>return</td><td>1..1</td><td><a href="structuredefinition.html">StructureDefinition</a></td><td/><td><div><p>The structure definition with a snapshot</p>\n</div></td></tr></table><div/></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 5,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/StructureDefinition-snapshot",
    version: "4.0.1",
    name: "Generate Snapshot",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "Generates a [StructureDefinition](structuredefinition.html) instance  with  a snapshot, based on a differential in a specified [StructureDefinition](structuredefinition.html).   \n\nIf the operation is not called at the instance level, either *definition* or *url* 'in' parameters must be provided. If more than one is specified, servers may raise an error or may resolve with the parameter of their choice. If called at the instance level, these parameters will be ignored.",
    code: "snapshot",
    resource: ["StructureDefinition"],
    system: false,
    type: true,
    instance: true,
    parameter: [
      {
        name: "definition",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The [StructureDefinition](structuredefinition.html) is provided directly as part of the request. Servers may choose not to accept profiles in this fashion",
        type: "StructureDefinition",
      },
      {
        name: "url",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The StructureDefinition's canonical URL (i.e. 'StructureDefinition.url'). The server must know the structure definition, or be able to retrieve it from other known repositories.",
        type: "string",
        searchType: "token",
      },
      {
        name: "return",
        use: "out",
        min: 1,
        max: "1",
        documentation: "The structure definition with a snapshot",
        type: "StructureDefinition",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace StructureMapTransform {
  export type Input = { source?: fhirTypes.uri; content: fhirTypes.Resource };
  export type Output = fhirTypes.Resource;
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "StructureMap-transform",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Model Instance Transformation</h2><p>OPERATION: Model Instance Transformation</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/StructureMap-transform</pre><div><p>The transform operation takes input content, applies a structure map transform, and then returns the output.</p>\n</div><p>URL: [base]/StructureMap/$transform</p><p>URL: [base]/StructureMap/[id]/$transform</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>source</td><td>0..1</td><td><a href="datatypes.html#uri">uri</a></td><td/><td><div><p>The structure map to apply. This is only needed if the operation is invoked at the resource level. If the $transform operation is invoked on a particular structure map, this will be ignored by the server</p>\n</div></td></tr><tr><td>IN</td><td>content</td><td>1..1</td><td><a href="resource.html">Resource</a></td><td/><td><div><p>The logical content to transform</p>\n</div></td></tr><tr><td>OUT</td><td>return</td><td>1..1</td><td><a href="resource.html">Resource</a></td><td/><td><div><p>The result of the transform</p>\n</div></td></tr></table><div><p>The input and return are specified as \'Resources\'. In most usage of the $transform operation, either the input or return content is not a valid FHIR resource. In these cases, the return type is actually a <a href="binary.html">Binary</a> resource. For this operation, the Binary resources may be encoded directly, using a mime-type, as shown in the example. Note: this specification does not yet address the means by which the servers may know the correct mime types for the various content involved</p>\n</div></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 2,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "trial-use",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/StructureMap-transform",
    version: "4.0.1",
    name: "Model Instance Transformation",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "The transform operation takes input content, applies a structure map transform, and then returns the output.",
    code: "transform",
    comment:
      "The input and return are specified as 'Resources'. In most usage of the $transform operation, either the input or return content is not a valid FHIR resource. In these cases, the return type is actually a [Binary](binary.html) resource. For this operation, the Binary resources may be encoded directly, using a mime-type, as shown in the example. Note: this specification does not yet address the means by which the servers may know the correct mime types for the various content involved",
    resource: ["StructureMap"],
    system: false,
    type: true,
    instance: true,
    parameter: [
      {
        name: "source",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The structure map to apply. This is only needed if the operation is invoked at the resource level. If the $transform operation is invoked on a particular structure map, this will be ignored by the server",
        type: "uri",
      },
      {
        name: "content",
        use: "in",
        min: 1,
        max: "1",
        documentation: "The logical content to transform",
        type: "Resource",
      },
      {
        name: "return",
        use: "out",
        min: 1,
        max: "1",
        documentation: "The result of the transform",
        type: "Resource",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace ValueSetExpand {
  export type Input = {
    url?: fhirTypes.uri;
    valueSet?: fhirTypes.ValueSet;
    valueSetVersion?: fhirTypes.string;
    context?: fhirTypes.uri;
    contextDirection?: fhirTypes.code;
    filter?: fhirTypes.string;
    date?: fhirTypes.dateTime;
    offset?: fhirTypes.integer;
    count?: fhirTypes.integer;
    includeDesignations?: fhirTypes.boolean;
    designation?: Array<fhirTypes.string>;
    includeDefinition?: fhirTypes.boolean;
    activeOnly?: fhirTypes.boolean;
    excludeNested?: fhirTypes.boolean;
    excludeNotForUI?: fhirTypes.boolean;
    excludePostCoordinated?: fhirTypes.boolean;
    displayLanguage?: fhirTypes.code;
    "exclude-system"?: Array<fhirTypes.canonical>;
    "system-version"?: Array<fhirTypes.canonical>;
    "check-system-version"?: Array<fhirTypes.canonical>;
    "force-system-version"?: Array<fhirTypes.canonical>;
  };
  export type Output = fhirTypes.ValueSet;
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "ValueSet-expand",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Value Set Expansion</h2><p>OPERATION: Value Set Expansion</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/ValueSet-expand</pre><div><p>The definition of a value set is used to create a simple collection of codes suitable for use for data entry or validation.</p>\n<p>If the operation is not called at the instance level, one of the in parameters url, context or valueSet must be provided.  An expanded value set will be returned, or an OperationOutcome with an error message.</p>\n</div><p>URL: [base]/ValueSet/$expand</p><p>URL: [base]/ValueSet/[id]/$expand</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>url</td><td>0..1</td><td><a href="datatypes.html#uri">uri</a></td><td/><td><div><p>A canonical reference to a value set. The server must know the value set (e.g. it is defined explicitly in the server\'s value sets, or it is defined implicitly by some code system known to the server</p>\n</div></td></tr><tr><td>IN</td><td>valueSet</td><td>0..1</td><td><a href="valueset.html">ValueSet</a></td><td/><td><div><p>The value set is provided directly as part of the request. Servers may choose not to accept value sets in this fashion</p>\n</div></td></tr><tr><td>IN</td><td>valueSetVersion</td><td>0..1</td><td><a href="datatypes.html#string">string</a></td><td/><td><div><p>The identifier that is used to identify a specific version of the value set to be used when generating the expansion. This is an arbitrary value managed by the value set author and is not expected to be globally unique. For example, it might be a timestamp (e.g. yyyymmdd) if a managed version is not available.</p>\n</div></td></tr><tr><td>IN</td><td>context</td><td>0..1</td><td><a href="datatypes.html#uri">uri</a></td><td/><td><div><p>The context of the value set, so that the server can resolve this to a value set to expand. The recommended format for this URI is [Structure Definition URL]#[name or path into structure definition] e.g. http://hl7.org/fhir/StructureDefinition/observation-hspc-height-hspcheight#Observation.interpretation. Other forms may be used but are not defined. This form is only usable if the terminology server also has access to the conformance registry that the server is using, but can be used to delegate the mapping from an application context to a binding at run-time</p>\n</div></td></tr><tr><td>IN</td><td>contextDirection</td><td>0..1</td><td><a href="datatypes.html#code">code</a></td><td/><td><div><p>If a context is provided, a context direction may also be provided. Valid values are:</p>\n<ul>\n<li>\'incoming\': the codes a client can use for PUT/POST operations,  and</li>\n<li>\'outgoing\', the codes a client might receive from the server.</li>\n</ul>\n<p>The purpose is to inform the server whether to use the value set associated with the context for reading or writing purposes (note: for most elements, this is the same value set, but there are a few elements where the reading and writing value sets are different)</p>\n</div></td></tr><tr><td>IN</td><td>filter</td><td>0..1</td><td><a href="datatypes.html#string">string</a></td><td/><td><div><p>A text filter that is applied to restrict the codes that are returned (this is useful in a UI context). The interpretation of this is delegated to the server in order to allow to determine the most optimal search approach for the context. The server can document the way this parameter works in <a href="terminologycapabilities.html">TerminologyCapabilities</a>..expansion.textFilter. Typical usage of this parameter includes functionality like:</p>\n<ul>\n<li>using left matching e.g. &quot;acut ast&quot;</li>\n<li>allowing for wild cards such as %, &amp;, ?</li>\n<li>searching on definition as well as display(s)</li>\n<li>allowing for search conditions (and / or / exclusions)</li>\n</ul>\n<p>Text Search engines such as Lucene or Solr, long with their considerable functionality, might also be used. The optional text search might also be code system specific, and servers might have different implementations for different code systems</p>\n</div></td></tr><tr><td>IN</td><td>date</td><td>0..1</td><td><a href="datatypes.html#dateTime">dateTime</a></td><td/><td><div><p>The date for which the expansion should be generated.  if a date is provided, it means that the server should use the value set / code system definitions as they were on the given date, or return an error if this is not possible.  Normally, the date is the current conditions (which is the default value) but under some circumstances, systems need to generate an expansion as it would have been in the past. A typical example of this would be where code selection is constrained to the set of codes that were available when the patient was treated, not when the record is being edited. Note that which date is appropriate is a matter for implementation policy.</p>\n</div></td></tr><tr><td>IN</td><td>offset</td><td>0..1</td><td><a href="datatypes.html#integer">integer</a></td><td/><td><div><p>Paging support - where to start if a subset is desired (default = 0). Offset is number of records (not number of pages)</p>\n</div></td></tr><tr><td>IN</td><td>count</td><td>0..1</td><td><a href="datatypes.html#integer">integer</a></td><td/><td><div><p>Paging support - how many codes should be provided in a partial page view. Paging only applies to flat expansions - servers ignore paging if the expansion is not flat.  If count = 0, the client is asking how large the expansion is. Servers SHOULD honor this request for hierarchical expansions as well, and simply return the overall count</p>\n</div></td></tr><tr><td>IN</td><td>includeDesignations</td><td>0..1</td><td><a href="datatypes.html#boolean">boolean</a></td><td/><td><div><p>Controls whether concept designations are to be included or excluded in value set expansions</p>\n</div></td></tr><tr><td>IN</td><td>designation</td><td>0..*</td><td><a href="datatypes.html#string">string</a></td><td/><td><div><p>A <a href="search.html#token">token</a> that specifies a system+code that is either a use or a language. Designations that match by language or use are included in the expansion. If no designation is specified, it is at the server discretion which designations to return</p>\n</div></td></tr><tr><td>IN</td><td>includeDefinition</td><td>0..1</td><td><a href="datatypes.html#boolean">boolean</a></td><td/><td><div><p>Controls whether the value set definition is included or excluded in value set expansions</p>\n</div></td></tr><tr><td>IN</td><td>activeOnly</td><td>0..1</td><td><a href="datatypes.html#boolean">boolean</a></td><td/><td><div><p>Controls whether inactive concepts are included or excluded in value set expansions. Note that if the value set explicitly specifies that inactive codes are included, this parameter can still remove them from a specific expansion, but this parameter cannot include them if the value set excludes them</p>\n</div></td></tr><tr><td>IN</td><td>excludeNested</td><td>0..1</td><td><a href="datatypes.html#boolean">boolean</a></td><td/><td><div><p>Controls whether or not the value set expansion nests codes or not (i.e. ValueSet.expansion.contains.contains)</p>\n</div></td></tr><tr><td>IN</td><td>excludeNotForUI</td><td>0..1</td><td><a href="datatypes.html#boolean">boolean</a></td><td/><td><div><p>Controls whether or not the value set expansion is assembled for a user interface use or not. Value sets intended for User Interface might include <a href="codesystem.html#status">\'abstract\' codes</a> or have nested contains with items with no code or abstract = true, with the sole purpose of helping a user navigate through the list efficiently, where as a value set not generated for UI use might be flat, and only contain the selectable codes in the value set. The exact implications of \'for UI\' depend on the code system, and what properties it exposes for a terminology server to use. In the FHIR Specification itself, the value set expansions are generated with excludeNotForUI = false, and the expansions used when generated schema / code etc, or performing validation, are all excludeNotForUI = true.</p>\n</div></td></tr><tr><td>IN</td><td>excludePostCoordinated</td><td>0..1</td><td><a href="datatypes.html#boolean">boolean</a></td><td/><td><div><p>Controls whether or not the value set expansion includes post coordinated codes</p>\n</div></td></tr><tr><td>IN</td><td>displayLanguage</td><td>0..1</td><td><a href="datatypes.html#code">code</a></td><td/><td><div><p>Specifies the language to be used for description in the expansions i.e. the language to be used for ValueSet.expansion.contains.display</p>\n</div></td></tr><tr><td>IN</td><td>exclude-system</td><td>0..*</td><td><a href="datatypes.html#canonical">canonical</a></td><td/><td><div><p>Code system, or a particular version of a code system to be excluded from the value set expansion. The format is the same as a canonical URL: [system]|[version] - e.g. http://loinc.org|2.56</p>\n</div></td></tr><tr><td>IN</td><td>system-version</td><td>0..*</td><td><a href="datatypes.html#canonical">canonical</a></td><td/><td><div><p>Specifies a version to use for a system, if the value set does not specify which one to use. The format is the same as a canonical URL: [system]|[version] - e.g. http://loinc.org|2.56</p>\n</div></td></tr><tr><td>IN</td><td>check-system-version</td><td>0..*</td><td><a href="datatypes.html#canonical">canonical</a></td><td/><td><div><p>Edge Case: Specifies a version to use for a system. If a value set specifies a different version, an error is returned instead of the expansion. The format is the same as a canonical URL: [system]|[version] - e.g. http://loinc.org|2.56</p>\n</div></td></tr><tr><td>IN</td><td>force-system-version</td><td>0..*</td><td><a href="datatypes.html#canonical">canonical</a></td><td/><td><div><p>Edge Case: Specifies a version to use for a system. This parameter overrides any specified version in the value set (and any it depends on). The format is the same as a canonical URL: [system]|[version] - e.g. http://loinc.org|2.56. Note that this has obvious safety issues, in that it may result in a value set expansion giving a different list of codes that is both wrong and unsafe, and implementers should only use this capability reluctantly. It primarily exists to deal with situations where specifications have fallen into decay as time passes. If the value is override, the version used SHALL explicitly be represented in the expansion parameters</p>\n</div></td></tr><tr><td>OUT</td><td>return</td><td>1..1</td><td><a href="valueset.html">ValueSet</a></td><td/><td><div><p>The result of the expansion. Servers generating expansions SHOULD ensure that all the parameters that affect the contents of the expansion are recorded in the ValueSet.expansion.parameter list</p>\n</div></td></tr></table><div><p>The value set expansion returned by this query should be treated as a transient result that will change over time (whether it does or not depends on how the value set is specified), so applications should repeat the operation each time the value set is used.</p>\n<p>If the expansion is too large (at the discretion of the server), the server MAY return an error (OperationOutcome with code too-costly). Clients can work through large flat expansions in a set of pages (partial views of the full expansion) instead of just getting the full expansion in a single exchange by using offset and count parameters, or use the count parameter to request a subset of the expansion for limited purposes. Servers are not obliged to support paging, but if they do, SHALL support both the offset and count parameters. Hierarchical expansions are not subject to paging and servers simply return the entire expansion.</p>\n<p>Different servers may return different results from expanding a value set for the following reasons:</p>\n<ul>\n<li>The underlying code systems are different (e.g. different versions, possibly with different defined behavior)</li>\n<li>The server optimizes filter includes differently, such as sorting by code frequency</li>\n<li>Servers introduce arbitrary groups to assist a user to navigate the lists based either on extensions in the definition, or additional knowledge available to the server</li>\n</ul>\n<p>When a server cannot correctly expand a value set because it does not fully understand the code systems (e.g. it has the wrong version, or incomplete definitions) then it SHALL return an error. If the value set itself is unbounded due to the inclusion of post-coordinated value sets (e.g. SNOMED CT, UCUM), then the extension <a href="extension-valueset-unclosed.html">http://hl7.org/fhir/StructureDefinition/valueset-unclosed</a> can be used to indicate that the expansion is incomplete</p>\n</div></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 5,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "normative",
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-normative-version",
        valueCode: "4.0.1",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/ValueSet-expand",
    version: "4.0.1",
    name: "Value Set Expansion",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "The definition of a value set is used to create a simple collection of codes suitable for use for data entry or validation. \n\nIf the operation is not called at the instance level, one of the in parameters url, context or valueSet must be provided.  An expanded value set will be returned, or an OperationOutcome with an error message.",
    code: "expand",
    comment:
      "The value set expansion returned by this query should be treated as a transient result that will change over time (whether it does or not depends on how the value set is specified), so applications should repeat the operation each time the value set is used.  \n\nIf the expansion is too large (at the discretion of the server), the server MAY return an error (OperationOutcome with code too-costly). Clients can work through large flat expansions in a set of pages (partial views of the full expansion) instead of just getting the full expansion in a single exchange by using offset and count parameters, or use the count parameter to request a subset of the expansion for limited purposes. Servers are not obliged to support paging, but if they do, SHALL support both the offset and count parameters. Hierarchical expansions are not subject to paging and servers simply return the entire expansion.  \n\nDifferent servers may return different results from expanding a value set for the following reasons:  \n\n* The underlying code systems are different (e.g. different versions, possibly with different defined behavior) \n* The server optimizes filter includes differently, such as sorting by code frequency \n* Servers introduce arbitrary groups to assist a user to navigate the lists based either on extensions in the definition, or additional knowledge available to the server\n\nWhen a server cannot correctly expand a value set because it does not fully understand the code systems (e.g. it has the wrong version, or incomplete definitions) then it SHALL return an error. If the value set itself is unbounded due to the inclusion of post-coordinated value sets (e.g. SNOMED CT, UCUM), then the extension [http://hl7.org/fhir/StructureDefinition/valueset-unclosed](extension-valueset-unclosed.html) can be used to indicate that the expansion is incomplete",
    resource: ["ValueSet"],
    system: false,
    type: true,
    instance: true,
    parameter: [
      {
        name: "url",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "A canonical reference to a value set. The server must know the value set (e.g. it is defined explicitly in the server's value sets, or it is defined implicitly by some code system known to the server",
        type: "uri",
      },
      {
        name: "valueSet",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The value set is provided directly as part of the request. Servers may choose not to accept value sets in this fashion",
        type: "ValueSet",
      },
      {
        name: "valueSetVersion",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The identifier that is used to identify a specific version of the value set to be used when generating the expansion. This is an arbitrary value managed by the value set author and is not expected to be globally unique. For example, it might be a timestamp (e.g. yyyymmdd) if a managed version is not available.",
        type: "string",
      },
      {
        name: "context",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The context of the value set, so that the server can resolve this to a value set to expand. The recommended format for this URI is [Structure Definition URL]#[name or path into structure definition] e.g. http://hl7.org/fhir/StructureDefinition/observation-hspc-height-hspcheight#Observation.interpretation. Other forms may be used but are not defined. This form is only usable if the terminology server also has access to the conformance registry that the server is using, but can be used to delegate the mapping from an application context to a binding at run-time",
        type: "uri",
      },
      {
        name: "contextDirection",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "If a context is provided, a context direction may also be provided. Valid values are: \n\n* 'incoming': the codes a client can use for PUT/POST operations,  and \n* 'outgoing', the codes a client might receive from the server.\n\nThe purpose is to inform the server whether to use the value set associated with the context for reading or writing purposes (note: for most elements, this is the same value set, but there are a few elements where the reading and writing value sets are different)",
        type: "code",
      },
      {
        name: "filter",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          'A text filter that is applied to restrict the codes that are returned (this is useful in a UI context). The interpretation of this is delegated to the server in order to allow to determine the most optimal search approach for the context. The server can document the way this parameter works in [TerminologyCapabilities](terminologycapabilities.html)..expansion.textFilter. Typical usage of this parameter includes functionality like:\n\n* using left matching e.g. "acut ast"\n* allowing for wild cards such as %, &, ?\n* searching on definition as well as display(s)\n* allowing for search conditions (and / or / exclusions)\n\nText Search engines such as Lucene or Solr, long with their considerable functionality, might also be used. The optional text search might also be code system specific, and servers might have different implementations for different code systems',
        type: "string",
      },
      {
        name: "date",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The date for which the expansion should be generated.  if a date is provided, it means that the server should use the value set / code system definitions as they were on the given date, or return an error if this is not possible.  Normally, the date is the current conditions (which is the default value) but under some circumstances, systems need to generate an expansion as it would have been in the past. A typical example of this would be where code selection is constrained to the set of codes that were available when the patient was treated, not when the record is being edited. Note that which date is appropriate is a matter for implementation policy.",
        type: "dateTime",
      },
      {
        name: "offset",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "Paging support - where to start if a subset is desired (default = 0). Offset is number of records (not number of pages)",
        type: "integer",
      },
      {
        name: "count",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "Paging support - how many codes should be provided in a partial page view. Paging only applies to flat expansions - servers ignore paging if the expansion is not flat.  If count = 0, the client is asking how large the expansion is. Servers SHOULD honor this request for hierarchical expansions as well, and simply return the overall count",
        type: "integer",
      },
      {
        name: "includeDesignations",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "Controls whether concept designations are to be included or excluded in value set expansions",
        type: "boolean",
      },
      {
        name: "designation",
        use: "in",
        min: 0,
        max: "*",
        documentation:
          "A [token](search.html#token) that specifies a system+code that is either a use or a language. Designations that match by language or use are included in the expansion. If no designation is specified, it is at the server discretion which designations to return",
        type: "string",
      },
      {
        name: "includeDefinition",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "Controls whether the value set definition is included or excluded in value set expansions",
        type: "boolean",
      },
      {
        name: "activeOnly",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "Controls whether inactive concepts are included or excluded in value set expansions. Note that if the value set explicitly specifies that inactive codes are included, this parameter can still remove them from a specific expansion, but this parameter cannot include them if the value set excludes them",
        type: "boolean",
      },
      {
        name: "excludeNested",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "Controls whether or not the value set expansion nests codes or not (i.e. ValueSet.expansion.contains.contains)",
        type: "boolean",
      },
      {
        name: "excludeNotForUI",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "Controls whether or not the value set expansion is assembled for a user interface use or not. Value sets intended for User Interface might include ['abstract' codes](codesystem.html#status) or have nested contains with items with no code or abstract = true, with the sole purpose of helping a user navigate through the list efficiently, where as a value set not generated for UI use might be flat, and only contain the selectable codes in the value set. The exact implications of 'for UI' depend on the code system, and what properties it exposes for a terminology server to use. In the FHIR Specification itself, the value set expansions are generated with excludeNotForUI = false, and the expansions used when generated schema / code etc, or performing validation, are all excludeNotForUI = true.",
        type: "boolean",
      },
      {
        name: "excludePostCoordinated",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "Controls whether or not the value set expansion includes post coordinated codes",
        type: "boolean",
      },
      {
        name: "displayLanguage",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "Specifies the language to be used for description in the expansions i.e. the language to be used for ValueSet.expansion.contains.display",
        type: "code",
      },
      {
        name: "exclude-system",
        use: "in",
        min: 0,
        max: "*",
        documentation:
          "Code system, or a particular version of a code system to be excluded from the value set expansion. The format is the same as a canonical URL: [system]|[version] - e.g. http://loinc.org|2.56",
        type: "canonical",
      },
      {
        name: "system-version",
        use: "in",
        min: 0,
        max: "*",
        documentation:
          "Specifies a version to use for a system, if the value set does not specify which one to use. The format is the same as a canonical URL: [system]|[version] - e.g. http://loinc.org|2.56",
        type: "canonical",
      },
      {
        name: "check-system-version",
        use: "in",
        min: 0,
        max: "*",
        documentation:
          "Edge Case: Specifies a version to use for a system. If a value set specifies a different version, an error is returned instead of the expansion. The format is the same as a canonical URL: [system]|[version] - e.g. http://loinc.org|2.56",
        type: "canonical",
      },
      {
        name: "force-system-version",
        use: "in",
        min: 0,
        max: "*",
        documentation:
          "Edge Case: Specifies a version to use for a system. This parameter overrides any specified version in the value set (and any it depends on). The format is the same as a canonical URL: [system]|[version] - e.g. http://loinc.org|2.56. Note that this has obvious safety issues, in that it may result in a value set expansion giving a different list of codes that is both wrong and unsafe, and implementers should only use this capability reluctantly. It primarily exists to deal with situations where specifications have fallen into decay as time passes. If the value is override, the version used SHALL explicitly be represented in the expansion parameters",
        type: "canonical",
      },
      {
        name: "return",
        use: "out",
        min: 1,
        max: "1",
        documentation:
          "The result of the expansion. Servers generating expansions SHOULD ensure that all the parameters that affect the contents of the expansion are recorded in the ValueSet.expansion.parameter list",
        type: "ValueSet",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace ValueSetValidateCode {
  export type Input = {
    url?: fhirTypes.uri;
    context?: fhirTypes.uri;
    valueSet?: fhirTypes.ValueSet;
    valueSetVersion?: fhirTypes.string;
    code?: fhirTypes.code;
    system?: fhirTypes.uri;
    systemVersion?: fhirTypes.string;
    display?: fhirTypes.string;
    coding?: fhirTypes.Coding;
    codeableConcept?: fhirTypes.CodeableConcept;
    date?: fhirTypes.dateTime;
    abstract?: fhirTypes.boolean;
    displayLanguage?: fhirTypes.code;
  };
  export type Output = {
    result: fhirTypes.boolean;
    message?: fhirTypes.string;
    display?: fhirTypes.string;
  };
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "ValueSet-validate-code",
    meta: { lastUpdated: "2019-11-01T09:29:23.356+11:00" },
    text: {
      status: "generated",
      div: '<div xmlns="http://www.w3.org/1999/xhtml"><h2>Value Set based Validation</h2><p>OPERATION: Value Set based Validation</p><p>The official URL for this operation definition is: </p><pre>http://hl7.org/fhir/OperationDefinition/ValueSet-validate-code</pre><div><p>Validate that a coded value is in the set of codes allowed by a value set.</p>\n<p>If the operation is not called at the instance level, one of the in parameters url, context or  valueSet must be provided.  One (and only one) of the in parameters code, coding, or codeableConcept must be provided.  The operation returns a result (true / false), an error message, and the recommended display for the code</p>\n</div><p>URL: [base]/ValueSet/$validate-code</p><p>URL: [base]/ValueSet/[id]/$validate-code</p><p>Parameters</p><table class="grid"><tr><td><b>Use</b></td><td><b>Name</b></td><td><b>Cardinality</b></td><td><b>Type</b></td><td><b>Binding</b></td><td><b>Documentation</b></td></tr><tr><td>IN</td><td>url</td><td>0..1</td><td><a href="datatypes.html#uri">uri</a></td><td/><td><div><p>Value set Canonical URL. The server must know the value set (e.g. it is defined explicitly in the server\'s value sets, or it is defined implicitly by some code system known to the server</p>\n</div></td></tr><tr><td>IN</td><td>context</td><td>0..1</td><td><a href="datatypes.html#uri">uri</a></td><td/><td><div><p>The context of the value set, so that the server can resolve this to a value set to validate against. The recommended format for this URI is [Structure Definition URL]#[name or path into structure definition] e.g. http://hl7.org/fhir/StructureDefinition/observation-hspc-height-hspcheight#Observation.interpretation. Other forms may be used but are not defined. This form is only usable if the terminology server also has access to the conformance registry that the server is using, but can be used to delegate the mapping from an application context to a binding at run-time</p>\n</div></td></tr><tr><td>IN</td><td>valueSet</td><td>0..1</td><td><a href="valueset.html">ValueSet</a></td><td/><td><div><p>The value set is provided directly as part of the request. Servers may choose not to accept value sets in this fashion. This parameter is used when the client wants the server to expand a value set that is not stored on the server</p>\n</div></td></tr><tr><td>IN</td><td>valueSetVersion</td><td>0..1</td><td><a href="datatypes.html#string">string</a></td><td/><td><div><p>The identifier that is used to identify a specific version of the value set to be used when validating the code. This is an arbitrary value managed by the value set author and is not expected to be globally unique. For example, it might be a timestamp (e.g. yyyymmdd) if a managed version is not available.</p>\n</div></td></tr><tr><td>IN</td><td>code</td><td>0..1</td><td><a href="datatypes.html#code">code</a></td><td/><td><div><p>The code that is to be validated. If a code is provided, a system or a context must be provided (if a context is provided, then the server SHALL ensure that the code is not ambiguous without a system)</p>\n</div></td></tr><tr><td>IN</td><td>system</td><td>0..1</td><td><a href="datatypes.html#uri">uri</a></td><td/><td><div><p>The system for the code that is to be validated</p>\n</div></td></tr><tr><td>IN</td><td>systemVersion</td><td>0..1</td><td><a href="datatypes.html#string">string</a></td><td/><td><div><p>The version of the system, if one was provided in the source data</p>\n</div></td></tr><tr><td>IN</td><td>display</td><td>0..1</td><td><a href="datatypes.html#string">string</a></td><td/><td><div><p>The display associated with the code, if provided. If a display is provided a code must be provided. If no display is provided, the server cannot validate the display value, but may choose to return a recommended display name using the display parameter in the outcome. Whether displays are case sensitive is code system dependent</p>\n</div></td></tr><tr><td>IN</td><td>coding</td><td>0..1</td><td><a href="datatypes.html#Coding">Coding</a></td><td/><td><div><p>A coding to validate</p>\n</div></td></tr><tr><td>IN</td><td>codeableConcept</td><td>0..1</td><td><a href="datatypes.html#CodeableConcept">CodeableConcept</a></td><td/><td><div><p>A full codeableConcept to validate. The server returns true if one of the coding values is in the value set, and may also validate that the codings are not in conflict with each other if more than one is present</p>\n</div></td></tr><tr><td>IN</td><td>date</td><td>0..1</td><td><a href="datatypes.html#dateTime">dateTime</a></td><td/><td><div><p>The date for which the validation should be checked. Normally, this is the current conditions (which is the default values) but under some circumstances, systems need to validate that a correct code was used at some point in the past. A typical example of this would be where code selection is constrained to the set of codes that were available when the patient was treated, not when the record is being edited. Note that which date is appropriate is a matter for implementation policy.</p>\n</div></td></tr><tr><td>IN</td><td>abstract</td><td>0..1</td><td><a href="datatypes.html#boolean">boolean</a></td><td/><td><div><p>If this parameter has a value of true, the client is stating that the validation is being performed in a context where a concept designated as \'abstract\' is appropriate/allowed to be used, and the server should regard abstract codes as valid. If this parameter is false, abstract codes are not considered to be valid.</p>\n<p>Note that. \'abstract\' is a property defined by many HL7 code systems that indicates that the concept is a logical grouping concept that is not intended to be used asa \'concrete\' concept to in an actual patient/care/process record. This language is borrowed from Object Orienated theory where \'asbtract\' objects are never instantiated. However in the general record and terminology eco-system, there are many contexts where it is appropraite to use these codes e.g. as decision making criterion, or when editing value sets themselves. This parameter allows a client to indicate to the server that it is working in such a context.</p>\n</div></td></tr><tr><td>IN</td><td>displayLanguage</td><td>0..1</td><td><a href="datatypes.html#code">code</a></td><td/><td><div><p>Specifies the language to be used for description when validating the display property</p>\n</div></td></tr><tr><td>OUT</td><td>result</td><td>1..1</td><td><a href="datatypes.html#boolean">boolean</a></td><td/><td><div><p>True if the concept details supplied are valid</p>\n</div></td></tr><tr><td>OUT</td><td>message</td><td>0..1</td><td><a href="datatypes.html#string">string</a></td><td/><td><div><p>Error details, if result = false. If this is provided when result = true, the message carries hints and warnings</p>\n</div></td></tr><tr><td>OUT</td><td>display</td><td>0..1</td><td><a href="datatypes.html#string">string</a></td><td/><td><div><p>A valid display for the concept if the system wishes to display this to a user</p>\n</div></td></tr></table><div><p>Note: the correct behaviour of validation with regard to language for Coding.display items is currently undefined, and further development and testing may lead to specific requirements or recommendations in subsequent releases</p>\n</div></div>',
    },
    extension: [
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-fmm",
        valueInteger: 5,
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
        valueCode: "normative",
      },
      {
        url: "http://hl7.org/fhir/StructureDefinition/structuredefinition-normative-version",
        valueCode: "4.0.1",
      },
    ],
    url: "http://hl7.org/fhir/OperationDefinition/ValueSet-validate-code",
    version: "4.0.1",
    name: "Value Set based Validation",
    status: "draft",
    kind: "operation",
    date: "2019-11-01T09:29:23+11:00",
    publisher: "HL7 (FHIR Project)",
    contact: [
      {
        telecom: [
          { system: "url", value: "http://hl7.org/fhir" },
          { system: "email", value: "fhir@lists.hl7.org" },
        ],
      },
    ],
    description:
      "Validate that a coded value is in the set of codes allowed by a value set.\n\nIf the operation is not called at the instance level, one of the in parameters url, context or  valueSet must be provided.  One (and only one) of the in parameters code, coding, or codeableConcept must be provided.  The operation returns a result (true / false), an error message, and the recommended display for the code",
    code: "validate-code",
    comment:
      "Note: the correct behaviour of validation with regard to language for Coding.display items is currently undefined, and further development and testing may lead to specific requirements or recommendations in subsequent releases",
    resource: ["ValueSet"],
    system: false,
    type: true,
    instance: true,
    parameter: [
      {
        name: "url",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "Value set Canonical URL. The server must know the value set (e.g. it is defined explicitly in the server's value sets, or it is defined implicitly by some code system known to the server",
        type: "uri",
      },
      {
        name: "context",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The context of the value set, so that the server can resolve this to a value set to validate against. The recommended format for this URI is [Structure Definition URL]#[name or path into structure definition] e.g. http://hl7.org/fhir/StructureDefinition/observation-hspc-height-hspcheight#Observation.interpretation. Other forms may be used but are not defined. This form is only usable if the terminology server also has access to the conformance registry that the server is using, but can be used to delegate the mapping from an application context to a binding at run-time",
        type: "uri",
      },
      {
        name: "valueSet",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The value set is provided directly as part of the request. Servers may choose not to accept value sets in this fashion. This parameter is used when the client wants the server to expand a value set that is not stored on the server",
        type: "ValueSet",
      },
      {
        name: "valueSetVersion",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The identifier that is used to identify a specific version of the value set to be used when validating the code. This is an arbitrary value managed by the value set author and is not expected to be globally unique. For example, it might be a timestamp (e.g. yyyymmdd) if a managed version is not available.",
        type: "string",
      },
      {
        name: "code",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The code that is to be validated. If a code is provided, a system or a context must be provided (if a context is provided, then the server SHALL ensure that the code is not ambiguous without a system)",
        type: "code",
      },
      {
        name: "system",
        use: "in",
        min: 0,
        max: "1",
        documentation: "The system for the code that is to be validated",
        type: "uri",
      },
      {
        name: "systemVersion",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The version of the system, if one was provided in the source data",
        type: "string",
      },
      {
        name: "display",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The display associated with the code, if provided. If a display is provided a code must be provided. If no display is provided, the server cannot validate the display value, but may choose to return a recommended display name using the display parameter in the outcome. Whether displays are case sensitive is code system dependent",
        type: "string",
      },
      {
        name: "coding",
        use: "in",
        min: 0,
        max: "1",
        documentation: "A coding to validate",
        type: "Coding",
      },
      {
        name: "codeableConcept",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "A full codeableConcept to validate. The server returns true if one of the coding values is in the value set, and may also validate that the codings are not in conflict with each other if more than one is present",
        type: "CodeableConcept",
      },
      {
        name: "date",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The date for which the validation should be checked. Normally, this is the current conditions (which is the default values) but under some circumstances, systems need to validate that a correct code was used at some point in the past. A typical example of this would be where code selection is constrained to the set of codes that were available when the patient was treated, not when the record is being edited. Note that which date is appropriate is a matter for implementation policy.",
        type: "dateTime",
      },
      {
        name: "abstract",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "If this parameter has a value of true, the client is stating that the validation is being performed in a context where a concept designated as 'abstract' is appropriate/allowed to be used, and the server should regard abstract codes as valid. If this parameter is false, abstract codes are not considered to be valid.\n\nNote that. 'abstract' is a property defined by many HL7 code systems that indicates that the concept is a logical grouping concept that is not intended to be used asa 'concrete' concept to in an actual patient/care/process record. This language is borrowed from Object Orienated theory where 'asbtract' objects are never instantiated. However in the general record and terminology eco-system, there are many contexts where it is appropraite to use these codes e.g. as decision making criterion, or when editing value sets themselves. This parameter allows a client to indicate to the server that it is working in such a context.",
        type: "boolean",
      },
      {
        name: "displayLanguage",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "Specifies the language to be used for description when validating the display property",
        type: "code",
      },
      {
        name: "result",
        use: "out",
        min: 1,
        max: "1",
        documentation: "True if the concept details supplied are valid",
        type: "boolean",
      },
      {
        name: "message",
        use: "out",
        min: 0,
        max: "1",
        documentation:
          "Error details, if result = false. If this is provided when result = true, the message carries hints and warnings",
        type: "string",
      },
      {
        name: "display",
        use: "out",
        min: 0,
        max: "1",
        documentation:
          "A valid display for the concept if the system wishes to display this to a user",
        type: "string",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace OxidizedHealthDeployOperation {
  export type Input = {
    code: fhirTypes.string;
    environment?: Array<{ name: fhirTypes.string; value: fhirTypes.string }>;
  };
  export type Output = fhirTypes.OperationOutcome;
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "oxidized-health-deploy-operation",
    url: "https://oxidized-health.app/OperationDefinition/deploy-operation",
    version: "4.0.1",
    name: "Deploy an operations code to the server.",
    status: "draft",
    kind: "operation",
    date: "2024-05-01T09:29:23+11:00",
    publisher: "OxidizedHealth",
    description: "Deploy an operations code to the server.",
    code: "deploy",
    system: false,
    type: false,
    instance: true,
    resource: ["OperationDefinition"],
    parameter: [
      { name: "code", use: "in", min: 1, max: "1", type: "string" },
      {
        name: "environment",
        use: "in",
        min: 0,
        max: "*",
        documentation: "environment variables.",
        part: [
          { name: "name", use: "in", min: 1, max: "1", type: "string" },
          { name: "value", use: "in", min: 1, max: "1", type: "string" },
        ],
      },
      {
        name: "return",
        use: "out",
        min: 1,
        max: "1",
        documentation: "Result of the deployment.",
        type: "OperationOutcome",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace OxidizedHealthEncrypt {
  export type Input = { value: fhirTypes.string };
  export type Output = { encryptedValue: fhirTypes.string };
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "oxidized-health-encrypt",
    url: "https://oxidized-health.app/OperationDefinition/encrypt",
    version: "4.0.1",
    name: "Encrypt a string value",
    status: "draft",
    kind: "operation",
    date: "2023-11-16T09:29:23+11:00",
    publisher: "OxidizedHealth",
    description: "Encrypts a passed in string value",
    code: "encrypt",
    system: true,
    type: false,
    instance: false,
    parameter: [
      {
        name: "value",
        use: "in",
        min: 1,
        max: "1",
        documentation: "Value to encrypt",
        type: "string",
      },
      {
        name: "encryptedValue",
        use: "out",
        min: 1,
        max: "1",
        documentation: "Value that has been encrypted",
        type: "string",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace OxidizedHealthEvaluatePolicy {
  export type Input = { user?: fhirTypes.Reference; request: fhirTypes.Bundle };
  export type Output = fhirTypes.OperationOutcome;
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "oxidized-health-evaluate-policy",
    url: "https://oxidized-health.app/OperationDefinition/evaluate-policy",
    version: "4.0.1",
    name: "EvaluateAccessPolicy",
    status: "draft",
    kind: "operation",
    date: "2023-11-16T09:29:23+11:00",
    publisher: "OxidizedHealth",
    description: "Evaluate an Access Policy.",
    code: "evaluate-policy",
    system: false,
    type: false,
    instance: true,
    resource: ["AccessPolicyV2"],
    parameter: [
      {
        name: "user",
        use: "in",
        min: 0,
        max: "1",
        documentation:
          "The user to evaluate the policy against. Defaults to logged in user if not present.",
        type: "Reference",
      },
      {
        name: "request",
        use: "in",
        min: 1,
        max: "1",
        documentation: "The requests to evaluate against the policy.",
        type: "Bundle",
      },
      {
        name: "return",
        use: "out",
        min: 1,
        max: "1",
        documentation: "The result of the policy evaluation.",
        type: "OperationOutcome",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace OxidizedHealthIdpRegistrationInfo {
  export type Input = Record<string, never>;
  export type Output = {
    information?: Array<{ name: fhirTypes.string; value: fhirTypes.string }>;
  };
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "oxidized-health-idp-registration-info",
    url: "https://oxidized-health.app/OperationDefinition/identity-provider-registration-info",
    version: "4.0.1",
    name: "IdentityProviderRegistrationInformation",
    status: "draft",
    kind: "operation",
    publisher: "OxidizedHealth",
    description: "Get the registration information for an identity provider.",
    code: "registration-info",
    system: false,
    type: false,
    instance: true,
    resource: ["IdentityProvider"],
    parameter: [
      {
        name: "information",
        use: "out",
        min: 0,
        max: "*",
        documentation: "IdentityProviders registration information.",
        part: [
          {
            name: "name",
            use: "out",
            min: 1,
            max: "1",
            documentation: "The name of the property.",
            type: "string",
          },
          {
            name: "value",
            use: "out",
            min: 1,
            max: "1",
            documentation: "the value of the property.",
            type: "string",
          },
        ],
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace OxidizedHealthInviteUser {
  export type Input = {
    email: fhirTypes.string;
    role: fhirTypes.code;
    accessPolicy?: fhirTypes.Reference;
  };
  export type Output = fhirTypes.OperationOutcome;
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "oxidized-health-invite-user",
    url: "https://oxidized-health.app/OperationDefinition/invite-user",
    version: "4.0.1",
    name: "Invite a user and send an email.",
    status: "draft",
    kind: "operation",
    date: "2023-11-16T09:29:23+11:00",
    publisher: "OxidizedHealth",
    description: "Invite a user to join tenant and send an email for invite.",
    code: "invite-user",
    system: false,
    type: true,
    instance: false,
    resource: ["Membership"],
    parameter: [
      {
        name: "email",
        use: "in",
        min: 1,
        max: "1",
        documentation: "Email for new user",
        type: "string",
      },
      {
        name: "role",
        use: "in",
        min: 1,
        max: "1",
        documentation: "Role for new user",
        type: "code",
      },
      {
        name: "accessPolicy",
        use: "in",
        min: 0,
        max: "1",
        documentation: "Access Policy for user.",
        type: "Reference",
      },
      {
        name: "return",
        use: "out",
        min: 1,
        max: "1",
        documentation: "The result of the operation.",
        type: "OperationOutcome",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace OxidizedHealthMessagePost {
  export type Input = { input: fhirTypes.Resource };
  export type Output = fhirTypes.OperationOutcome;
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "oxidized-health-message-post",
    url: "https://oxidized-health.app/OperationDefinition/message-post",
    version: "4.0.1",
    name: "Send a message to a given topic.",
    status: "draft",
    kind: "operation",
    date: "2024-05-01T09:29:23+11:00",
    publisher: "OxidizedHealth",
    description: "Posts a message to a given topic.",
    code: "message-post",
    system: false,
    type: false,
    instance: true,
    resource: ["MessageTopic"],
    parameter: [
      {
        name: "input",
        use: "in",
        min: 1,
        max: "1",
        documentation: "Value to encrypt",
        type: "Resource",
      },
      {
        name: "return",
        use: "out",
        min: 1,
        max: "1",
        documentation: "Result of the message post operation.",
        type: "OperationOutcome",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace OxidizedHealthPasswordReset {
  export type Input = {
    email: {
      subject: fhirTypes.string;
      body: fhirTypes.string;
      acceptText: fhirTypes.string;
    };
  };
  export type Output = fhirTypes.OperationOutcome;
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "oxidized-health-password-reset",
    url: "https://oxidized-health.app/OperationDefinition/password-reset",
    version: "4.0.1",
    name: "Password Reset",
    status: "draft",
    kind: "operation",
    date: "2025-01-08T09:29:23+11:00",
    publisher: "OxidizedHealth",
    description: "Password Reset for a user.",
    code: "password-reset",
    system: false,
    type: false,
    instance: true,
    resource: ["Membership"],
    parameter: [
      {
        name: "email",
        use: "in",
        min: 1,
        max: "1",
        documentation: "email content",
        part: [
          {
            name: "subject",
            use: "in",
            min: 1,
            max: "1",
            documentation: "Subject for the email.",
            type: "string",
          },
          {
            name: "body",
            use: "in",
            min: 1,
            max: "1",
            documentation: "Body for the email.",
            type: "string",
          },
          {
            name: "acceptText",
            use: "in",
            min: 1,
            max: "1",
            documentation: "Button text for the password reset.",
            type: "string",
          },
        ],
      },
      {
        name: "return",
        use: "out",
        min: 1,
        max: "1",
        documentation: "The result of the operation.",
        type: "OperationOutcome",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace OxidizedHealthDeleteRefreshToken {
  export type Input = { id: fhirTypes.id };
  export type Output = fhirTypes.OperationOutcome;
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "oxidized-health-delete-refresh-token",
    url: "https://oxidized-health.app/OperationDefinition/delete-refresh-tokens",
    version: "4.0.1",
    name: "Delete refresh token",
    status: "draft",
    kind: "operation",
    date: "2023-11-16T09:29:23+11:00",
    publisher: "OxidizedHealth",
    description: "Delete refresh token from the user for the client.",
    code: "delete-refresh-token",
    system: true,
    type: false,
    instance: false,
    parameter: [
      {
        name: "id",
        use: "in",
        min: 1,
        max: "1",
        documentation: "Refresh token id to delete.",
        type: "id",
      },
      {
        name: "return",
        use: "out",
        min: 1,
        max: "1",
        documentation: "Result of the delete operation.",
        type: "OperationOutcome",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace OxidizedHealthListRefreshTokens {
  export type Input = Record<string, never>;
  export type Output = {
    "refresh-tokens"?: Array<{
      client_id: fhirTypes.id;
      id: fhirTypes.id;
      created_at: fhirTypes.dateTime;
    }>;
  };
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "oxidized-health-list-refresh-tokens",
    url: "https://oxidized-health.app/OperationDefinition/list-refresh-tokens",
    version: "4.0.1",
    name: "Show Refresh Tokens",
    status: "draft",
    kind: "operation",
    date: "2023-11-16T09:29:23+11:00",
    publisher: "OxidizedHealth",
    description: "Show list of users refresh tokens.",
    code: "refresh-tokens",
    system: true,
    type: false,
    instance: false,
    parameter: [
      {
        name: "refresh-tokens",
        use: "out",
        min: 0,
        max: "*",
        documentation: "The result of the operation.",
        part: [
          {
            name: "client_id",
            use: "out",
            min: 1,
            max: "1",
            documentation: "Client for refresh token.",
            type: "id",
          },
          {
            name: "id",
            use: "out",
            min: 1,
            max: "1",
            documentation: "Refresh token id to delete.",
            type: "id",
          },
          {
            name: "created_at",
            use: "out",
            min: 1,
            max: "1",
            documentation: "When the refresh token was created.",
            type: "dateTime",
          },
        ],
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace OxidizedHealthDeleteScope {
  export type Input = { client_id: fhirTypes.id };
  export type Output = fhirTypes.OperationOutcome;
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "oxidized-health-delete-scope",
    url: "https://oxidized-health.app/OperationDefinition/delete-scopes",
    version: "4.0.1",
    name: "Delete Scope",
    status: "draft",
    kind: "operation",
    date: "2023-11-16T09:29:23+11:00",
    publisher: "OxidizedHealth",
    description: "Delete scope from user accepted scopes for the client.",
    code: "delete-scope",
    system: true,
    type: false,
    instance: false,
    parameter: [
      {
        name: "client_id",
        use: "in",
        min: 1,
        max: "1",
        documentation: "Client for which scopes are being shown.",
        type: "id",
      },
      {
        name: "return",
        use: "out",
        min: 1,
        max: "1",
        documentation: "Result of the delete operation.",
        type: "OperationOutcome",
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace OxidizedHealthListScopes {
  export type Input = Record<string, never>;
  export type Output = {
    scopes?: Array<{
      client_id: fhirTypes.id;
      scopes: fhirTypes.string;
      created_at: fhirTypes.dateTime;
    }>;
  };
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "oxidized-health-list-scopes",
    url: "https://oxidized-health.app/OperationDefinition/list-scopes",
    version: "4.0.1",
    name: "Show Scopes",
    status: "draft",
    kind: "operation",
    date: "2023-11-16T09:29:23+11:00",
    publisher: "OxidizedHealth",
    description: "Show list of user accepted scopes for apps.",
    code: "scopes",
    system: true,
    type: false,
    instance: false,
    parameter: [
      {
        name: "scopes",
        use: "out",
        min: 0,
        max: "*",
        documentation: "The result of the operation.",
        part: [
          {
            name: "client_id",
            use: "out",
            min: 1,
            max: "1",
            documentation: "Client for which scopes are being shown.",
            type: "id",
          },
          {
            name: "scopes",
            use: "out",
            min: 1,
            max: "1",
            documentation: "Scopes user accepted.",
            type: "string",
          },
          {
            name: "created_at",
            use: "out",
            min: 1,
            max: "1",
            documentation: "When the scopes were accepted.",
            type: "dateTime",
          },
        ],
      },
    ],
  } as fhirTypes.OperationDefinition);
}
export namespace OxidizedHealthUsageStatistics {
  export type Input = Record<string, never>;
  export type Output = {
    statistics?: Array<{
      name: fhirTypes.string;
      version: fhirTypes.string;
      limit: fhirTypes.integer;
      usage: fhirTypes.integer;
      description: fhirTypes.string;
    }>;
  };
  export type IOp = IOperation<Input, Output>;
  export const Op: IOp = new Operation<Input, Output>({
    resourceType: "OperationDefinition",
    id: "oxidized-health-usage-statistics",
    url: "https://oxidized-health.app/OperationDefinition/usage-statistics",
    version: "4.0.1",
    name: "OxidizedHealth Usage Statistics",
    status: "draft",
    kind: "operation",
    date: "2024-05-16T09:29:23+11:00",
    publisher: "OxidizedHealth",
    description: "Get current tenants limitations and usage.",
    code: "usage-statistics",
    system: true,
    type: false,
    instance: false,
    parameter: [
      {
        name: "statistics",
        use: "out",
        min: 0,
        max: "*",
        documentation: "The result of the operation.",
        part: [
          {
            name: "name",
            use: "out",
            min: 1,
            max: "1",
            documentation: "The name of the limit.",
            type: "string",
          },
          {
            name: "version",
            use: "out",
            min: 1,
            max: "1",
            documentation: "The fhir version for the limit.",
            type: "string",
          },
          {
            name: "limit",
            use: "out",
            min: 1,
            max: "1",
            documentation: "Value of the limit.",
            type: "integer",
          },
          {
            name: "usage",
            use: "out",
            min: 1,
            max: "1",
            documentation: "The current usage.",
            type: "integer",
          },
          {
            name: "description",
            use: "out",
            min: 1,
            max: "1",
            documentation: "The description of the limit.",
            type: "string",
          },
        ],
      },
    ],
  } as fhirTypes.OperationDefinition);
}
