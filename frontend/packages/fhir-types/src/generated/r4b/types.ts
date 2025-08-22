export type base64Binary = string & { _base64Binary: "fhir_base64Binary"; } ;
// @ts-ignore
export type boolean = boolean;
export type canonical = string & { _canonical: "fhir_canonical"; }  & uri;
export type code = string & { _code: "fhir_code"; }  & string;
export type date = string & { _date: "fhir_date"; } ;
export type dateTime = string & { _dateTime: "fhir_dateTime"; } ;
export type decimal = number & { _decimal: "fhir_decimal"; } ;
export type id = string & { _id: "fhir_id"; }  & string;
export type instant = string & { _instant: "fhir_instant"; } ;
export type integer = number & { _integer: "fhir_integer"; } ;
export type markdown = string & { _markdown: "fhir_markdown"; }  & string;
export type oid = string & { _oid: "fhir_oid"; }  & uri;
export type positiveInt = string & { _positiveInt: "fhir_positiveInt"; }  & integer;
// @ts-ignore
export type string = string;
export type time = string & { _time: "fhir_time"; } ;
export type unsignedInt = string & { _unsignedInt: "fhir_unsignedInt"; }  & integer;
export type uri = string & { _uri: "fhir_uri"; } ;
export type url = string & { _url: "fhir_url"; }  & uri;
export type uuid = string & { _uuid: "fhir_uuid"; }  & uri;
export type xhtml = string & { _xhtml: "fhir_xhtml"; } ;

export interface Element {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
}

export interface BackboneElement {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
}

export interface Address {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * home | work | temp | old | billing - purpose of this address
   */
  use?: code;
  /** 
   * home | work | temp | old | billing - purpose of this address
   */
  _use?: Element
  /** 
   * postal | physical | both
   */
  type?: code;
  /** 
   * postal | physical | both
   */
  _type?: Element
  /** 
   * Text representation of the address
   */
  text?: string;
  /** 
   * Text representation of the address
   */
  _text?: Element
  /** 
   * Street name, number, direction & P.O. Box etc.
   */
  line?: Array<string>;
  /** 
   * Street name, number, direction & P.O. Box etc.
   */
  _line?: Array<Element>
  /** 
   * Name of city, town etc.
   */
  city?: string;
  /** 
   * Name of city, town etc.
   */
  _city?: Element
  /** 
   * District name (aka county)
   */
  district?: string;
  /** 
   * District name (aka county)
   */
  _district?: Element
  /** 
   * Sub-unit of country (abbreviations ok)
   */
  state?: string;
  /** 
   * Sub-unit of country (abbreviations ok)
   */
  _state?: Element
  /** 
   * Postal code for area
   */
  postalCode?: string;
  /** 
   * Postal code for area
   */
  _postalCode?: Element
  /** 
   * Country (e.g. can be ISO 3166 2 or 3 letter code)
   */
  country?: string;
  /** 
   * Country (e.g. can be ISO 3166 2 or 3 letter code)
   */
  _country?: Element
  /** 
   * Time period when address was/is in use
   */
  period?: Period;
}

export interface Age {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Numerical value (with implicit precision)
   */
  value?: decimal;
  /** 
   * Numerical value (with implicit precision)
   */
  _value?: Element
  /** 
   * < | <= | >= | > - how to understand the value
   */
  comparator?: code;
  /** 
   * < | <= | >= | > - how to understand the value
   */
  _comparator?: Element
  /** 
   * Unit representation
   */
  unit?: string;
  /** 
   * Unit representation
   */
  _unit?: Element
  /** 
   * System that defines coded unit form
   */
  system?: uri;
  /** 
   * System that defines coded unit form
   */
  _system?: Element
  /** 
   * Coded form of the unit
   */
  code?: code;
  /** 
   * Coded form of the unit
   */
  _code?: Element
}

export interface Annotation {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Individual responsible for the annotation
   */
  authorReference?: Reference;
  /** 
   * Individual responsible for the annotation
   */
  authorString?: string;
  /** 
   * Individual responsible for the annotation
   */
  _authorString?: Element
  /** 
   * When the annotation was made
   */
  time?: dateTime;
  /** 
   * When the annotation was made
   */
  _time?: Element
  /** 
   * The annotation  - text content (as markdown)
   */
  text: markdown;
  /** 
   * The annotation  - text content (as markdown)
   */
  _text?: Element
}

export interface Attachment {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Mime type of the content, with charset etc.
   */
  contentType?: code;
  /** 
   * Mime type of the content, with charset etc.
   */
  _contentType?: Element
  /** 
   * Human language of the content (BCP-47)
   */
  language?: code;
  /** 
   * Human language of the content (BCP-47)
   */
  _language?: Element
  /** 
   * Data inline, base64ed
   */
  data?: base64Binary;
  /** 
   * Data inline, base64ed
   */
  _data?: Element
  /** 
   * Uri where the data can be found
   */
  url?: url;
  /** 
   * Uri where the data can be found
   */
  _url?: Element
  /** 
   * Number of bytes of content (if url provided)
   */
  size?: unsignedInt;
  /** 
   * Number of bytes of content (if url provided)
   */
  _size?: Element
  /** 
   * Hash of the data (sha-1, base64ed)
   */
  hash?: base64Binary;
  /** 
   * Hash of the data (sha-1, base64ed)
   */
  _hash?: Element
  /** 
   * Label to display in place of the data
   */
  title?: string;
  /** 
   * Label to display in place of the data
   */
  _title?: Element
  /** 
   * Date attachment was first created
   */
  creation?: dateTime;
  /** 
   * Date attachment was first created
   */
  _creation?: Element
}

export interface CodeableConcept {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Code defined by a terminology system
   */
  coding?: Array<Coding>;
  /** 
   * Plain text representation of the concept
   */
  text?: string;
  /** 
   * Plain text representation of the concept
   */
  _text?: Element
}

export interface CodeableReference {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Reference to a concept (by class)
   */
  concept?: CodeableConcept;
  /** 
   * Reference to a resource (by instance)
   */
  reference?: Reference;
}

export interface Coding {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Identity of the terminology system
   */
  system?: uri;
  /** 
   * Identity of the terminology system
   */
  _system?: Element
  /** 
   * Version of the system - if relevant
   */
  version?: string;
  /** 
   * Version of the system - if relevant
   */
  _version?: Element
  /** 
   * Symbol in syntax defined by the system
   */
  code?: code;
  /** 
   * Symbol in syntax defined by the system
   */
  _code?: Element
  /** 
   * Representation defined by the system
   */
  display?: string;
  /** 
   * Representation defined by the system
   */
  _display?: Element
  /** 
   * If this coding was chosen directly by the user
   */
  userSelected?: boolean;
  /** 
   * If this coding was chosen directly by the user
   */
  _userSelected?: Element
}

export interface ContactDetail {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Name of an individual to contact
   */
  name?: string;
  /** 
   * Name of an individual to contact
   */
  _name?: Element
  /** 
   * Contact details for individual or organization
   */
  telecom?: Array<ContactPoint>;
}

export interface ContactPoint {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * phone | fax | email | pager | url | sms | other
   */
  system?: code;
  /** 
   * phone | fax | email | pager | url | sms | other
   */
  _system?: Element
  /** 
   * The actual contact point details
   */
  value?: string;
  /** 
   * The actual contact point details
   */
  _value?: Element
  /** 
   * home | work | temp | old | mobile - purpose of this contact point
   */
  use?: code;
  /** 
   * home | work | temp | old | mobile - purpose of this contact point
   */
  _use?: Element
  /** 
   * Specify preferred order of use (1 = highest)
   */
  rank?: positiveInt;
  /** 
   * Specify preferred order of use (1 = highest)
   */
  _rank?: Element
  /** 
   * Time period when the contact point was/is in use
   */
  period?: Period;
}

export interface Contributor {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * author | editor | reviewer | endorser
   */
  type: code;
  /** 
   * author | editor | reviewer | endorser
   */
  _type?: Element
  /** 
   * Who contributed the content
   */
  name: string;
  /** 
   * Who contributed the content
   */
  _name?: Element
  /** 
   * Contact details of the contributor
   */
  contact?: Array<ContactDetail>;
}

export interface Count {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Numerical value (with implicit precision)
   */
  value?: decimal;
  /** 
   * Numerical value (with implicit precision)
   */
  _value?: Element
  /** 
   * < | <= | >= | > - how to understand the value
   */
  comparator?: code;
  /** 
   * < | <= | >= | > - how to understand the value
   */
  _comparator?: Element
  /** 
   * Unit representation
   */
  unit?: string;
  /** 
   * Unit representation
   */
  _unit?: Element
  /** 
   * System that defines coded unit form
   */
  system?: uri;
  /** 
   * System that defines coded unit form
   */
  _system?: Element
  /** 
   * Coded form of the unit
   */
  code?: code;
  /** 
   * Coded form of the unit
   */
  _code?: Element
}

export interface DataRequirementCodeFilter {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * A code-valued attribute to filter on
   */
  path?: string;
  /** 
   * A code-valued attribute to filter on
   */
  _path?: Element
  /** 
   * A coded (token) parameter to search on
   */
  searchParam?: string;
  /** 
   * A coded (token) parameter to search on
   */
  _searchParam?: Element
  /** 
   * Valueset for the filter
   */
  valueSet?: canonical;
  /** 
   * Valueset for the filter
   */
  _valueSet?: Element
  /** 
   * What code is expected
   */
  code?: Array<Coding>;
}
export interface DataRequirementDateFilter {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * A date-valued attribute to filter on
   */
  path?: string;
  /** 
   * A date-valued attribute to filter on
   */
  _path?: Element
  /** 
   * A date valued parameter to search on
   */
  searchParam?: string;
  /** 
   * A date valued parameter to search on
   */
  _searchParam?: Element
  /** 
   * The value of the filter, as a Period, DateTime, or Duration value
   */
  valueDateTime?: dateTime;
  /** 
   * The value of the filter, as a Period, DateTime, or Duration value
   */
  _valueDateTime?: Element
  /** 
   * The value of the filter, as a Period, DateTime, or Duration value
   */
  valuePeriod?: Period;
  /** 
   * The value of the filter, as a Period, DateTime, or Duration value
   */
  valueDuration?: Duration;
}
export interface DataRequirementSort {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * The name of the attribute to perform the sort
   */
  path: string;
  /** 
   * The name of the attribute to perform the sort
   */
  _path?: Element
  /** 
   * ascending | descending
   */
  direction: code;
  /** 
   * ascending | descending
   */
  _direction?: Element
}
export interface DataRequirement {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * The type of the required data
   */
  type: code;
  /** 
   * The type of the required data
   */
  _type?: Element
  /** 
   * The profile of the required data
   */
  profile?: Array<canonical>;
  /** 
   * The profile of the required data
   */
  _profile?: Array<Element>
  /** 
   * E.g. Patient, Practitioner, RelatedPerson, Organization, Location, Device
   */
  subjectCodeableConcept?: CodeableConcept;
  /** 
   * E.g. Patient, Practitioner, RelatedPerson, Organization, Location, Device
   */
  subjectReference?: Reference;
  /** 
   * Indicates specific structure elements that are referenced by the knowledge module
   */
  mustSupport?: Array<string>;
  /** 
   * Indicates specific structure elements that are referenced by the knowledge module
   */
  _mustSupport?: Array<Element>
  /** 
   * What codes are expected
   */
  codeFilter?: Array<DataRequirementCodeFilter>;
  /** 
   * What dates/date ranges are expected
   */
  dateFilter?: Array<DataRequirementDateFilter>;
  /** 
   * Number of results
   */
  limit?: positiveInt;
  /** 
   * Number of results
   */
  _limit?: Element
  /** 
   * Order of the results
   */
  sort?: Array<DataRequirementSort>;
}

export interface Distance {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Numerical value (with implicit precision)
   */
  value?: decimal;
  /** 
   * Numerical value (with implicit precision)
   */
  _value?: Element
  /** 
   * < | <= | >= | > - how to understand the value
   */
  comparator?: code;
  /** 
   * < | <= | >= | > - how to understand the value
   */
  _comparator?: Element
  /** 
   * Unit representation
   */
  unit?: string;
  /** 
   * Unit representation
   */
  _unit?: Element
  /** 
   * System that defines coded unit form
   */
  system?: uri;
  /** 
   * System that defines coded unit form
   */
  _system?: Element
  /** 
   * Coded form of the unit
   */
  code?: code;
  /** 
   * Coded form of the unit
   */
  _code?: Element
}

export interface DosageDoseAndRate {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * The kind of dose or rate specified
   */
  type?: CodeableConcept;
  /** 
   * Amount of medication per dose
   */
  doseRange?: Range;
  /** 
   * Amount of medication per dose
   */
  doseQuantity?: Quantity;
  /** 
   * Amount of medication per unit of time
   */
  rateRatio?: Ratio;
  /** 
   * Amount of medication per unit of time
   */
  rateRange?: Range;
  /** 
   * Amount of medication per unit of time
   */
  rateQuantity?: Quantity;
}
export interface Dosage {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The order of the dosage instructions
   */
  sequence?: integer;
  /** 
   * The order of the dosage instructions
   */
  _sequence?: Element
  /** 
   * Free text dosage instructions e.g. SIG
   */
  text?: string;
  /** 
   * Free text dosage instructions e.g. SIG
   */
  _text?: Element
  /** 
   * Supplemental instruction or warnings to the patient - e.g. "with meals", "may cause drowsiness"
   */
  additionalInstruction?: Array<CodeableConcept>;
  /** 
   * Patient or consumer oriented instructions
   */
  patientInstruction?: string;
  /** 
   * Patient or consumer oriented instructions
   */
  _patientInstruction?: Element
  /** 
   * When medication should be administered
   */
  timing?: Timing;
  /** 
   * Take "as needed" (for x)
   */
  asNeededBoolean?: boolean;
  /** 
   * Take "as needed" (for x)
   */
  _asNeededBoolean?: Element
  /** 
   * Take "as needed" (for x)
   */
  asNeededCodeableConcept?: CodeableConcept;
  /** 
   * Body site to administer to
   */
  site?: CodeableConcept;
  /** 
   * How drug should enter body
   */
  route?: CodeableConcept;
  /** 
   * Technique for administering medication
   */
  method?: CodeableConcept;
  /** 
   * Amount of medication administered
   */
  doseAndRate?: Array<DosageDoseAndRate>;
  /** 
   * Upper limit on medication per unit of time
   */
  maxDosePerPeriod?: Ratio;
  /** 
   * Upper limit on medication per administration
   */
  maxDosePerAdministration?: Quantity;
  /** 
   * Upper limit on medication per lifetime of the patient
   */
  maxDosePerLifetime?: Quantity;
}

export interface Duration {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Numerical value (with implicit precision)
   */
  value?: decimal;
  /** 
   * Numerical value (with implicit precision)
   */
  _value?: Element
  /** 
   * < | <= | >= | > - how to understand the value
   */
  comparator?: code;
  /** 
   * < | <= | >= | > - how to understand the value
   */
  _comparator?: Element
  /** 
   * Unit representation
   */
  unit?: string;
  /** 
   * Unit representation
   */
  _unit?: Element
  /** 
   * System that defines coded unit form
   */
  system?: uri;
  /** 
   * System that defines coded unit form
   */
  _system?: Element
  /** 
   * Coded form of the unit
   */
  code?: code;
  /** 
   * Coded form of the unit
   */
  _code?: Element
}

export interface ElementDefinitionSlicingDiscriminator {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * value | exists | pattern | type | profile
   */
  type: code;
  /** 
   * value | exists | pattern | type | profile
   */
  _type?: Element
  /** 
   * Path to element value
   */
  path: string;
  /** 
   * Path to element value
   */
  _path?: Element
}
export interface ElementDefinitionSlicing {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Element values that are used to distinguish the slices
   */
  discriminator?: Array<ElementDefinitionSlicingDiscriminator>;
  /** 
   * Text description of how slicing works (or not)
   */
  description?: string;
  /** 
   * Text description of how slicing works (or not)
   */
  _description?: Element
  /** 
   * If elements must be in same order as slices
   */
  ordered?: boolean;
  /** 
   * If elements must be in same order as slices
   */
  _ordered?: Element
  /** 
   * closed | open | openAtEnd
   */
  rules: code;
  /** 
   * closed | open | openAtEnd
   */
  _rules?: Element
}
export interface ElementDefinitionBase {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Path that identifies the base element
   */
  path: string;
  /** 
   * Path that identifies the base element
   */
  _path?: Element
  /** 
   * Min cardinality of the base element
   */
  min: unsignedInt;
  /** 
   * Min cardinality of the base element
   */
  _min?: Element
  /** 
   * Max cardinality of the base element
   */
  max: string;
  /** 
   * Max cardinality of the base element
   */
  _max?: Element
}
export interface ElementDefinitionType {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Data type or Resource (reference to definition)
   */
  code: uri;
  /** 
   * Data type or Resource (reference to definition)
   */
  _code?: Element
  /** 
   * Profiles (StructureDefinition or IG) - one must apply
   */
  profile?: Array<canonical>;
  /** 
   * Profiles (StructureDefinition or IG) - one must apply
   */
  _profile?: Array<Element>
  /** 
   * Profile (StructureDefinition or IG) on the Reference/canonical target - one must apply
   */
  targetProfile?: Array<canonical>;
  /** 
   * Profile (StructureDefinition or IG) on the Reference/canonical target - one must apply
   */
  _targetProfile?: Array<Element>
  /** 
   * contained | referenced | bundled - how aggregated
   */
  aggregation?: Array<code>;
  /** 
   * contained | referenced | bundled - how aggregated
   */
  _aggregation?: Array<Element>
  /** 
   * either | independent | specific
   */
  versioning?: code;
  /** 
   * either | independent | specific
   */
  _versioning?: Element
}
export interface ElementDefinitionExample {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Describes the purpose of this example
   */
  label: string;
  /** 
   * Describes the purpose of this example
   */
  _label?: Element
  /** 
   * Value of Example (one of allowed types)
   */
  valueBase64Binary?: base64Binary;
  /** 
   * Value of Example (one of allowed types)
   */
  _valueBase64Binary?: Element
  /** 
   * Value of Example (one of allowed types)
   */
  valueBoolean?: boolean;
  /** 
   * Value of Example (one of allowed types)
   */
  _valueBoolean?: Element
  /** 
   * Value of Example (one of allowed types)
   */
  valueCanonical?: canonical;
  /** 
   * Value of Example (one of allowed types)
   */
  _valueCanonical?: Element
  /** 
   * Value of Example (one of allowed types)
   */
  valueCode?: code;
  /** 
   * Value of Example (one of allowed types)
   */
  _valueCode?: Element
  /** 
   * Value of Example (one of allowed types)
   */
  valueDate?: date;
  /** 
   * Value of Example (one of allowed types)
   */
  _valueDate?: Element
  /** 
   * Value of Example (one of allowed types)
   */
  valueDateTime?: dateTime;
  /** 
   * Value of Example (one of allowed types)
   */
  _valueDateTime?: Element
  /** 
   * Value of Example (one of allowed types)
   */
  valueDecimal?: decimal;
  /** 
   * Value of Example (one of allowed types)
   */
  _valueDecimal?: Element
  /** 
   * Value of Example (one of allowed types)
   */
  valueId?: id;
  /** 
   * Value of Example (one of allowed types)
   */
  _valueId?: Element
  /** 
   * Value of Example (one of allowed types)
   */
  valueInstant?: instant;
  /** 
   * Value of Example (one of allowed types)
   */
  _valueInstant?: Element
  /** 
   * Value of Example (one of allowed types)
   */
  valueInteger?: integer;
  /** 
   * Value of Example (one of allowed types)
   */
  _valueInteger?: Element
  /** 
   * Value of Example (one of allowed types)
   */
  valueMarkdown?: markdown;
  /** 
   * Value of Example (one of allowed types)
   */
  _valueMarkdown?: Element
  /** 
   * Value of Example (one of allowed types)
   */
  valueOid?: oid;
  /** 
   * Value of Example (one of allowed types)
   */
  _valueOid?: Element
  /** 
   * Value of Example (one of allowed types)
   */
  valuePositiveInt?: positiveInt;
  /** 
   * Value of Example (one of allowed types)
   */
  _valuePositiveInt?: Element
  /** 
   * Value of Example (one of allowed types)
   */
  valueString?: string;
  /** 
   * Value of Example (one of allowed types)
   */
  _valueString?: Element
  /** 
   * Value of Example (one of allowed types)
   */
  valueTime?: time;
  /** 
   * Value of Example (one of allowed types)
   */
  _valueTime?: Element
  /** 
   * Value of Example (one of allowed types)
   */
  valueUnsignedInt?: unsignedInt;
  /** 
   * Value of Example (one of allowed types)
   */
  _valueUnsignedInt?: Element
  /** 
   * Value of Example (one of allowed types)
   */
  valueUri?: uri;
  /** 
   * Value of Example (one of allowed types)
   */
  _valueUri?: Element
  /** 
   * Value of Example (one of allowed types)
   */
  valueUrl?: url;
  /** 
   * Value of Example (one of allowed types)
   */
  _valueUrl?: Element
  /** 
   * Value of Example (one of allowed types)
   */
  valueUuid?: uuid;
  /** 
   * Value of Example (one of allowed types)
   */
  _valueUuid?: Element
  /** 
   * Value of Example (one of allowed types)
   */
  valueAddress?: Address;
  /** 
   * Value of Example (one of allowed types)
   */
  valueAge?: Age;
  /** 
   * Value of Example (one of allowed types)
   */
  valueAnnotation?: Annotation;
  /** 
   * Value of Example (one of allowed types)
   */
  valueAttachment?: Attachment;
  /** 
   * Value of Example (one of allowed types)
   */
  valueCodeableConcept?: CodeableConcept;
  /** 
   * Value of Example (one of allowed types)
   */
  valueCodeableReference?: CodeableReference;
  /** 
   * Value of Example (one of allowed types)
   */
  valueCoding?: Coding;
  /** 
   * Value of Example (one of allowed types)
   */
  valueContactPoint?: ContactPoint;
  /** 
   * Value of Example (one of allowed types)
   */
  valueCount?: Count;
  /** 
   * Value of Example (one of allowed types)
   */
  valueDistance?: Distance;
  /** 
   * Value of Example (one of allowed types)
   */
  valueDuration?: Duration;
  /** 
   * Value of Example (one of allowed types)
   */
  valueHumanName?: HumanName;
  /** 
   * Value of Example (one of allowed types)
   */
  valueIdentifier?: Identifier;
  /** 
   * Value of Example (one of allowed types)
   */
  valueMoney?: Money;
  /** 
   * Value of Example (one of allowed types)
   */
  valuePeriod?: Period;
  /** 
   * Value of Example (one of allowed types)
   */
  valueQuantity?: Quantity;
  /** 
   * Value of Example (one of allowed types)
   */
  valueRange?: Range;
  /** 
   * Value of Example (one of allowed types)
   */
  valueRatio?: Ratio;
  /** 
   * Value of Example (one of allowed types)
   */
  valueRatioRange?: RatioRange;
  /** 
   * Value of Example (one of allowed types)
   */
  valueReference?: Reference;
  /** 
   * Value of Example (one of allowed types)
   */
  valueSampledData?: SampledData;
  /** 
   * Value of Example (one of allowed types)
   */
  valueSignature?: Signature;
  /** 
   * Value of Example (one of allowed types)
   */
  valueTiming?: Timing;
  /** 
   * Value of Example (one of allowed types)
   */
  valueContactDetail?: ContactDetail;
  /** 
   * Value of Example (one of allowed types)
   */
  valueContributor?: Contributor;
  /** 
   * Value of Example (one of allowed types)
   */
  valueDataRequirement?: DataRequirement;
  /** 
   * Value of Example (one of allowed types)
   */
  valueExpression?: Expression;
  /** 
   * Value of Example (one of allowed types)
   */
  valueParameterDefinition?: ParameterDefinition;
  /** 
   * Value of Example (one of allowed types)
   */
  valueRelatedArtifact?: RelatedArtifact;
  /** 
   * Value of Example (one of allowed types)
   */
  valueTriggerDefinition?: TriggerDefinition;
  /** 
   * Value of Example (one of allowed types)
   */
  valueUsageContext?: UsageContext;
  /** 
   * Value of Example (one of allowed types)
   */
  valueDosage?: Dosage;
}
export interface ElementDefinitionConstraint {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Target of 'condition' reference above
   */
  key: id;
  /** 
   * Target of 'condition' reference above
   */
  _key?: Element
  /** 
   * Why this constraint is necessary or appropriate
   */
  requirements?: string;
  /** 
   * Why this constraint is necessary or appropriate
   */
  _requirements?: Element
  /** 
   * error | warning
   */
  severity: code;
  /** 
   * error | warning
   */
  _severity?: Element
  /** 
   * Human description of constraint
   */
  human: string;
  /** 
   * Human description of constraint
   */
  _human?: Element
  /** 
   * FHIRPath expression of constraint
   */
  expression?: string;
  /** 
   * FHIRPath expression of constraint
   */
  _expression?: Element
  /** 
   * XPath expression of constraint
   */
  xpath?: string;
  /** 
   * XPath expression of constraint
   */
  _xpath?: Element
  /** 
   * Reference to original source of constraint
   */
  source?: canonical;
  /** 
   * Reference to original source of constraint
   */
  _source?: Element
}
export interface ElementDefinitionBinding {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * required | extensible | preferred | example
   */
  strength: code;
  /** 
   * required | extensible | preferred | example
   */
  _strength?: Element
  /** 
   * Human explanation of the value set
   */
  description?: string;
  /** 
   * Human explanation of the value set
   */
  _description?: Element
  /** 
   * Source of value set
   */
  valueSet?: canonical;
  /** 
   * Source of value set
   */
  _valueSet?: Element
}
export interface ElementDefinitionMapping {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Reference to mapping declaration
   */
  identity: id;
  /** 
   * Reference to mapping declaration
   */
  _identity?: Element
  /** 
   * Computable language of mapping
   */
  language?: code;
  /** 
   * Computable language of mapping
   */
  _language?: Element
  /** 
   * Details of the mapping
   */
  map: string;
  /** 
   * Details of the mapping
   */
  _map?: Element
  /** 
   * Comments about the mapping or its use
   */
  comment?: string;
  /** 
   * Comments about the mapping or its use
   */
  _comment?: Element
}
export interface ElementDefinition {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Path of the element in the hierarchy of elements
   */
  path: string;
  /** 
   * Path of the element in the hierarchy of elements
   */
  _path?: Element
  /** 
   * xmlAttr | xmlText | typeAttr | cdaText | xhtml
   */
  representation?: Array<code>;
  /** 
   * xmlAttr | xmlText | typeAttr | cdaText | xhtml
   */
  _representation?: Array<Element>
  /** 
   * Name for this particular element (in a set of slices)
   */
  sliceName?: string;
  /** 
   * Name for this particular element (in a set of slices)
   */
  _sliceName?: Element
  /** 
   * If this slice definition constrains an inherited slice definition (or not)
   */
  sliceIsConstraining?: boolean;
  /** 
   * If this slice definition constrains an inherited slice definition (or not)
   */
  _sliceIsConstraining?: Element
  /** 
   * Name for element to display with or prompt for element
   */
  label?: string;
  /** 
   * Name for element to display with or prompt for element
   */
  _label?: Element
  /** 
   * Corresponding codes in terminologies
   */
  code?: Array<Coding>;
  /** 
   * This element is sliced - slices follow
   */
  slicing?: ElementDefinitionSlicing;
  /** 
   * Concise definition for space-constrained presentation
   */
  short?: string;
  /** 
   * Concise definition for space-constrained presentation
   */
  _short?: Element
  /** 
   * Full formal definition as narrative text
   */
  definition?: markdown;
  /** 
   * Full formal definition as narrative text
   */
  _definition?: Element
  /** 
   * Comments about the use of this element
   */
  comment?: markdown;
  /** 
   * Comments about the use of this element
   */
  _comment?: Element
  /** 
   * Why this resource has been created
   */
  requirements?: markdown;
  /** 
   * Why this resource has been created
   */
  _requirements?: Element
  /** 
   * Other names
   */
  alias?: Array<string>;
  /** 
   * Other names
   */
  _alias?: Array<Element>
  /** 
   * Minimum Cardinality
   */
  min?: unsignedInt;
  /** 
   * Minimum Cardinality
   */
  _min?: Element
  /** 
   * Maximum Cardinality (a number or *)
   */
  max?: string;
  /** 
   * Maximum Cardinality (a number or *)
   */
  _max?: Element
  /** 
   * Base definition information for tools
   */
  base?: ElementDefinitionBase;
  /** 
   * Reference to definition of content for the element
   */
  contentReference?: uri;
  /** 
   * Reference to definition of content for the element
   */
  _contentReference?: Element
  /** 
   * Data type and Profile for this element
   */
  type?: Array<ElementDefinitionType>;
  /** 
   * Specified value if missing from instance
   */
  defaultValueBase64Binary?: base64Binary;
  /** 
   * Specified value if missing from instance
   */
  _defaultValueBase64Binary?: Element
  /** 
   * Specified value if missing from instance
   */
  defaultValueBoolean?: boolean;
  /** 
   * Specified value if missing from instance
   */
  _defaultValueBoolean?: Element
  /** 
   * Specified value if missing from instance
   */
  defaultValueCanonical?: canonical;
  /** 
   * Specified value if missing from instance
   */
  _defaultValueCanonical?: Element
  /** 
   * Specified value if missing from instance
   */
  defaultValueCode?: code;
  /** 
   * Specified value if missing from instance
   */
  _defaultValueCode?: Element
  /** 
   * Specified value if missing from instance
   */
  defaultValueDate?: date;
  /** 
   * Specified value if missing from instance
   */
  _defaultValueDate?: Element
  /** 
   * Specified value if missing from instance
   */
  defaultValueDateTime?: dateTime;
  /** 
   * Specified value if missing from instance
   */
  _defaultValueDateTime?: Element
  /** 
   * Specified value if missing from instance
   */
  defaultValueDecimal?: decimal;
  /** 
   * Specified value if missing from instance
   */
  _defaultValueDecimal?: Element
  /** 
   * Specified value if missing from instance
   */
  defaultValueId?: id;
  /** 
   * Specified value if missing from instance
   */
  _defaultValueId?: Element
  /** 
   * Specified value if missing from instance
   */
  defaultValueInstant?: instant;
  /** 
   * Specified value if missing from instance
   */
  _defaultValueInstant?: Element
  /** 
   * Specified value if missing from instance
   */
  defaultValueInteger?: integer;
  /** 
   * Specified value if missing from instance
   */
  _defaultValueInteger?: Element
  /** 
   * Specified value if missing from instance
   */
  defaultValueMarkdown?: markdown;
  /** 
   * Specified value if missing from instance
   */
  _defaultValueMarkdown?: Element
  /** 
   * Specified value if missing from instance
   */
  defaultValueOid?: oid;
  /** 
   * Specified value if missing from instance
   */
  _defaultValueOid?: Element
  /** 
   * Specified value if missing from instance
   */
  defaultValuePositiveInt?: positiveInt;
  /** 
   * Specified value if missing from instance
   */
  _defaultValuePositiveInt?: Element
  /** 
   * Specified value if missing from instance
   */
  defaultValueString?: string;
  /** 
   * Specified value if missing from instance
   */
  _defaultValueString?: Element
  /** 
   * Specified value if missing from instance
   */
  defaultValueTime?: time;
  /** 
   * Specified value if missing from instance
   */
  _defaultValueTime?: Element
  /** 
   * Specified value if missing from instance
   */
  defaultValueUnsignedInt?: unsignedInt;
  /** 
   * Specified value if missing from instance
   */
  _defaultValueUnsignedInt?: Element
  /** 
   * Specified value if missing from instance
   */
  defaultValueUri?: uri;
  /** 
   * Specified value if missing from instance
   */
  _defaultValueUri?: Element
  /** 
   * Specified value if missing from instance
   */
  defaultValueUrl?: url;
  /** 
   * Specified value if missing from instance
   */
  _defaultValueUrl?: Element
  /** 
   * Specified value if missing from instance
   */
  defaultValueUuid?: uuid;
  /** 
   * Specified value if missing from instance
   */
  _defaultValueUuid?: Element
  /** 
   * Specified value if missing from instance
   */
  defaultValueAddress?: Address;
  /** 
   * Specified value if missing from instance
   */
  defaultValueAge?: Age;
  /** 
   * Specified value if missing from instance
   */
  defaultValueAnnotation?: Annotation;
  /** 
   * Specified value if missing from instance
   */
  defaultValueAttachment?: Attachment;
  /** 
   * Specified value if missing from instance
   */
  defaultValueCodeableConcept?: CodeableConcept;
  /** 
   * Specified value if missing from instance
   */
  defaultValueCodeableReference?: CodeableReference;
  /** 
   * Specified value if missing from instance
   */
  defaultValueCoding?: Coding;
  /** 
   * Specified value if missing from instance
   */
  defaultValueContactPoint?: ContactPoint;
  /** 
   * Specified value if missing from instance
   */
  defaultValueCount?: Count;
  /** 
   * Specified value if missing from instance
   */
  defaultValueDistance?: Distance;
  /** 
   * Specified value if missing from instance
   */
  defaultValueDuration?: Duration;
  /** 
   * Specified value if missing from instance
   */
  defaultValueHumanName?: HumanName;
  /** 
   * Specified value if missing from instance
   */
  defaultValueIdentifier?: Identifier;
  /** 
   * Specified value if missing from instance
   */
  defaultValueMoney?: Money;
  /** 
   * Specified value if missing from instance
   */
  defaultValuePeriod?: Period;
  /** 
   * Specified value if missing from instance
   */
  defaultValueQuantity?: Quantity;
  /** 
   * Specified value if missing from instance
   */
  defaultValueRange?: Range;
  /** 
   * Specified value if missing from instance
   */
  defaultValueRatio?: Ratio;
  /** 
   * Specified value if missing from instance
   */
  defaultValueRatioRange?: RatioRange;
  /** 
   * Specified value if missing from instance
   */
  defaultValueReference?: Reference;
  /** 
   * Specified value if missing from instance
   */
  defaultValueSampledData?: SampledData;
  /** 
   * Specified value if missing from instance
   */
  defaultValueSignature?: Signature;
  /** 
   * Specified value if missing from instance
   */
  defaultValueTiming?: Timing;
  /** 
   * Specified value if missing from instance
   */
  defaultValueContactDetail?: ContactDetail;
  /** 
   * Specified value if missing from instance
   */
  defaultValueContributor?: Contributor;
  /** 
   * Specified value if missing from instance
   */
  defaultValueDataRequirement?: DataRequirement;
  /** 
   * Specified value if missing from instance
   */
  defaultValueExpression?: Expression;
  /** 
   * Specified value if missing from instance
   */
  defaultValueParameterDefinition?: ParameterDefinition;
  /** 
   * Specified value if missing from instance
   */
  defaultValueRelatedArtifact?: RelatedArtifact;
  /** 
   * Specified value if missing from instance
   */
  defaultValueTriggerDefinition?: TriggerDefinition;
  /** 
   * Specified value if missing from instance
   */
  defaultValueUsageContext?: UsageContext;
  /** 
   * Specified value if missing from instance
   */
  defaultValueDosage?: Dosage;
  /** 
   * Implicit meaning when this element is missing
   */
  meaningWhenMissing?: markdown;
  /** 
   * Implicit meaning when this element is missing
   */
  _meaningWhenMissing?: Element
  /** 
   * What the order of the elements means
   */
  orderMeaning?: string;
  /** 
   * What the order of the elements means
   */
  _orderMeaning?: Element
  /** 
   * Value must be exactly this
   */
  fixedBase64Binary?: base64Binary;
  /** 
   * Value must be exactly this
   */
  _fixedBase64Binary?: Element
  /** 
   * Value must be exactly this
   */
  fixedBoolean?: boolean;
  /** 
   * Value must be exactly this
   */
  _fixedBoolean?: Element
  /** 
   * Value must be exactly this
   */
  fixedCanonical?: canonical;
  /** 
   * Value must be exactly this
   */
  _fixedCanonical?: Element
  /** 
   * Value must be exactly this
   */
  fixedCode?: code;
  /** 
   * Value must be exactly this
   */
  _fixedCode?: Element
  /** 
   * Value must be exactly this
   */
  fixedDate?: date;
  /** 
   * Value must be exactly this
   */
  _fixedDate?: Element
  /** 
   * Value must be exactly this
   */
  fixedDateTime?: dateTime;
  /** 
   * Value must be exactly this
   */
  _fixedDateTime?: Element
  /** 
   * Value must be exactly this
   */
  fixedDecimal?: decimal;
  /** 
   * Value must be exactly this
   */
  _fixedDecimal?: Element
  /** 
   * Value must be exactly this
   */
  fixedId?: id;
  /** 
   * Value must be exactly this
   */
  _fixedId?: Element
  /** 
   * Value must be exactly this
   */
  fixedInstant?: instant;
  /** 
   * Value must be exactly this
   */
  _fixedInstant?: Element
  /** 
   * Value must be exactly this
   */
  fixedInteger?: integer;
  /** 
   * Value must be exactly this
   */
  _fixedInteger?: Element
  /** 
   * Value must be exactly this
   */
  fixedMarkdown?: markdown;
  /** 
   * Value must be exactly this
   */
  _fixedMarkdown?: Element
  /** 
   * Value must be exactly this
   */
  fixedOid?: oid;
  /** 
   * Value must be exactly this
   */
  _fixedOid?: Element
  /** 
   * Value must be exactly this
   */
  fixedPositiveInt?: positiveInt;
  /** 
   * Value must be exactly this
   */
  _fixedPositiveInt?: Element
  /** 
   * Value must be exactly this
   */
  fixedString?: string;
  /** 
   * Value must be exactly this
   */
  _fixedString?: Element
  /** 
   * Value must be exactly this
   */
  fixedTime?: time;
  /** 
   * Value must be exactly this
   */
  _fixedTime?: Element
  /** 
   * Value must be exactly this
   */
  fixedUnsignedInt?: unsignedInt;
  /** 
   * Value must be exactly this
   */
  _fixedUnsignedInt?: Element
  /** 
   * Value must be exactly this
   */
  fixedUri?: uri;
  /** 
   * Value must be exactly this
   */
  _fixedUri?: Element
  /** 
   * Value must be exactly this
   */
  fixedUrl?: url;
  /** 
   * Value must be exactly this
   */
  _fixedUrl?: Element
  /** 
   * Value must be exactly this
   */
  fixedUuid?: uuid;
  /** 
   * Value must be exactly this
   */
  _fixedUuid?: Element
  /** 
   * Value must be exactly this
   */
  fixedAddress?: Address;
  /** 
   * Value must be exactly this
   */
  fixedAge?: Age;
  /** 
   * Value must be exactly this
   */
  fixedAnnotation?: Annotation;
  /** 
   * Value must be exactly this
   */
  fixedAttachment?: Attachment;
  /** 
   * Value must be exactly this
   */
  fixedCodeableConcept?: CodeableConcept;
  /** 
   * Value must be exactly this
   */
  fixedCodeableReference?: CodeableReference;
  /** 
   * Value must be exactly this
   */
  fixedCoding?: Coding;
  /** 
   * Value must be exactly this
   */
  fixedContactPoint?: ContactPoint;
  /** 
   * Value must be exactly this
   */
  fixedCount?: Count;
  /** 
   * Value must be exactly this
   */
  fixedDistance?: Distance;
  /** 
   * Value must be exactly this
   */
  fixedDuration?: Duration;
  /** 
   * Value must be exactly this
   */
  fixedHumanName?: HumanName;
  /** 
   * Value must be exactly this
   */
  fixedIdentifier?: Identifier;
  /** 
   * Value must be exactly this
   */
  fixedMoney?: Money;
  /** 
   * Value must be exactly this
   */
  fixedPeriod?: Period;
  /** 
   * Value must be exactly this
   */
  fixedQuantity?: Quantity;
  /** 
   * Value must be exactly this
   */
  fixedRange?: Range;
  /** 
   * Value must be exactly this
   */
  fixedRatio?: Ratio;
  /** 
   * Value must be exactly this
   */
  fixedRatioRange?: RatioRange;
  /** 
   * Value must be exactly this
   */
  fixedReference?: Reference;
  /** 
   * Value must be exactly this
   */
  fixedSampledData?: SampledData;
  /** 
   * Value must be exactly this
   */
  fixedSignature?: Signature;
  /** 
   * Value must be exactly this
   */
  fixedTiming?: Timing;
  /** 
   * Value must be exactly this
   */
  fixedContactDetail?: ContactDetail;
  /** 
   * Value must be exactly this
   */
  fixedContributor?: Contributor;
  /** 
   * Value must be exactly this
   */
  fixedDataRequirement?: DataRequirement;
  /** 
   * Value must be exactly this
   */
  fixedExpression?: Expression;
  /** 
   * Value must be exactly this
   */
  fixedParameterDefinition?: ParameterDefinition;
  /** 
   * Value must be exactly this
   */
  fixedRelatedArtifact?: RelatedArtifact;
  /** 
   * Value must be exactly this
   */
  fixedTriggerDefinition?: TriggerDefinition;
  /** 
   * Value must be exactly this
   */
  fixedUsageContext?: UsageContext;
  /** 
   * Value must be exactly this
   */
  fixedDosage?: Dosage;
  /** 
   * Value must have at least these property values
   */
  patternBase64Binary?: base64Binary;
  /** 
   * Value must have at least these property values
   */
  _patternBase64Binary?: Element
  /** 
   * Value must have at least these property values
   */
  patternBoolean?: boolean;
  /** 
   * Value must have at least these property values
   */
  _patternBoolean?: Element
  /** 
   * Value must have at least these property values
   */
  patternCanonical?: canonical;
  /** 
   * Value must have at least these property values
   */
  _patternCanonical?: Element
  /** 
   * Value must have at least these property values
   */
  patternCode?: code;
  /** 
   * Value must have at least these property values
   */
  _patternCode?: Element
  /** 
   * Value must have at least these property values
   */
  patternDate?: date;
  /** 
   * Value must have at least these property values
   */
  _patternDate?: Element
  /** 
   * Value must have at least these property values
   */
  patternDateTime?: dateTime;
  /** 
   * Value must have at least these property values
   */
  _patternDateTime?: Element
  /** 
   * Value must have at least these property values
   */
  patternDecimal?: decimal;
  /** 
   * Value must have at least these property values
   */
  _patternDecimal?: Element
  /** 
   * Value must have at least these property values
   */
  patternId?: id;
  /** 
   * Value must have at least these property values
   */
  _patternId?: Element
  /** 
   * Value must have at least these property values
   */
  patternInstant?: instant;
  /** 
   * Value must have at least these property values
   */
  _patternInstant?: Element
  /** 
   * Value must have at least these property values
   */
  patternInteger?: integer;
  /** 
   * Value must have at least these property values
   */
  _patternInteger?: Element
  /** 
   * Value must have at least these property values
   */
  patternMarkdown?: markdown;
  /** 
   * Value must have at least these property values
   */
  _patternMarkdown?: Element
  /** 
   * Value must have at least these property values
   */
  patternOid?: oid;
  /** 
   * Value must have at least these property values
   */
  _patternOid?: Element
  /** 
   * Value must have at least these property values
   */
  patternPositiveInt?: positiveInt;
  /** 
   * Value must have at least these property values
   */
  _patternPositiveInt?: Element
  /** 
   * Value must have at least these property values
   */
  patternString?: string;
  /** 
   * Value must have at least these property values
   */
  _patternString?: Element
  /** 
   * Value must have at least these property values
   */
  patternTime?: time;
  /** 
   * Value must have at least these property values
   */
  _patternTime?: Element
  /** 
   * Value must have at least these property values
   */
  patternUnsignedInt?: unsignedInt;
  /** 
   * Value must have at least these property values
   */
  _patternUnsignedInt?: Element
  /** 
   * Value must have at least these property values
   */
  patternUri?: uri;
  /** 
   * Value must have at least these property values
   */
  _patternUri?: Element
  /** 
   * Value must have at least these property values
   */
  patternUrl?: url;
  /** 
   * Value must have at least these property values
   */
  _patternUrl?: Element
  /** 
   * Value must have at least these property values
   */
  patternUuid?: uuid;
  /** 
   * Value must have at least these property values
   */
  _patternUuid?: Element
  /** 
   * Value must have at least these property values
   */
  patternAddress?: Address;
  /** 
   * Value must have at least these property values
   */
  patternAge?: Age;
  /** 
   * Value must have at least these property values
   */
  patternAnnotation?: Annotation;
  /** 
   * Value must have at least these property values
   */
  patternAttachment?: Attachment;
  /** 
   * Value must have at least these property values
   */
  patternCodeableConcept?: CodeableConcept;
  /** 
   * Value must have at least these property values
   */
  patternCodeableReference?: CodeableReference;
  /** 
   * Value must have at least these property values
   */
  patternCoding?: Coding;
  /** 
   * Value must have at least these property values
   */
  patternContactPoint?: ContactPoint;
  /** 
   * Value must have at least these property values
   */
  patternCount?: Count;
  /** 
   * Value must have at least these property values
   */
  patternDistance?: Distance;
  /** 
   * Value must have at least these property values
   */
  patternDuration?: Duration;
  /** 
   * Value must have at least these property values
   */
  patternHumanName?: HumanName;
  /** 
   * Value must have at least these property values
   */
  patternIdentifier?: Identifier;
  /** 
   * Value must have at least these property values
   */
  patternMoney?: Money;
  /** 
   * Value must have at least these property values
   */
  patternPeriod?: Period;
  /** 
   * Value must have at least these property values
   */
  patternQuantity?: Quantity;
  /** 
   * Value must have at least these property values
   */
  patternRange?: Range;
  /** 
   * Value must have at least these property values
   */
  patternRatio?: Ratio;
  /** 
   * Value must have at least these property values
   */
  patternRatioRange?: RatioRange;
  /** 
   * Value must have at least these property values
   */
  patternReference?: Reference;
  /** 
   * Value must have at least these property values
   */
  patternSampledData?: SampledData;
  /** 
   * Value must have at least these property values
   */
  patternSignature?: Signature;
  /** 
   * Value must have at least these property values
   */
  patternTiming?: Timing;
  /** 
   * Value must have at least these property values
   */
  patternContactDetail?: ContactDetail;
  /** 
   * Value must have at least these property values
   */
  patternContributor?: Contributor;
  /** 
   * Value must have at least these property values
   */
  patternDataRequirement?: DataRequirement;
  /** 
   * Value must have at least these property values
   */
  patternExpression?: Expression;
  /** 
   * Value must have at least these property values
   */
  patternParameterDefinition?: ParameterDefinition;
  /** 
   * Value must have at least these property values
   */
  patternRelatedArtifact?: RelatedArtifact;
  /** 
   * Value must have at least these property values
   */
  patternTriggerDefinition?: TriggerDefinition;
  /** 
   * Value must have at least these property values
   */
  patternUsageContext?: UsageContext;
  /** 
   * Value must have at least these property values
   */
  patternDosage?: Dosage;
  /** 
   * Example value (as defined for type)
   */
  example?: Array<ElementDefinitionExample>;
  /** 
   * Minimum Allowed Value (for some types)
   */
  minValueDate?: date;
  /** 
   * Minimum Allowed Value (for some types)
   */
  _minValueDate?: Element
  /** 
   * Minimum Allowed Value (for some types)
   */
  minValueDateTime?: dateTime;
  /** 
   * Minimum Allowed Value (for some types)
   */
  _minValueDateTime?: Element
  /** 
   * Minimum Allowed Value (for some types)
   */
  minValueInstant?: instant;
  /** 
   * Minimum Allowed Value (for some types)
   */
  _minValueInstant?: Element
  /** 
   * Minimum Allowed Value (for some types)
   */
  minValueTime?: time;
  /** 
   * Minimum Allowed Value (for some types)
   */
  _minValueTime?: Element
  /** 
   * Minimum Allowed Value (for some types)
   */
  minValueDecimal?: decimal;
  /** 
   * Minimum Allowed Value (for some types)
   */
  _minValueDecimal?: Element
  /** 
   * Minimum Allowed Value (for some types)
   */
  minValueInteger?: integer;
  /** 
   * Minimum Allowed Value (for some types)
   */
  _minValueInteger?: Element
  /** 
   * Minimum Allowed Value (for some types)
   */
  minValuePositiveInt?: positiveInt;
  /** 
   * Minimum Allowed Value (for some types)
   */
  _minValuePositiveInt?: Element
  /** 
   * Minimum Allowed Value (for some types)
   */
  minValueUnsignedInt?: unsignedInt;
  /** 
   * Minimum Allowed Value (for some types)
   */
  _minValueUnsignedInt?: Element
  /** 
   * Minimum Allowed Value (for some types)
   */
  minValueQuantity?: Quantity;
  /** 
   * Maximum Allowed Value (for some types)
   */
  maxValueDate?: date;
  /** 
   * Maximum Allowed Value (for some types)
   */
  _maxValueDate?: Element
  /** 
   * Maximum Allowed Value (for some types)
   */
  maxValueDateTime?: dateTime;
  /** 
   * Maximum Allowed Value (for some types)
   */
  _maxValueDateTime?: Element
  /** 
   * Maximum Allowed Value (for some types)
   */
  maxValueInstant?: instant;
  /** 
   * Maximum Allowed Value (for some types)
   */
  _maxValueInstant?: Element
  /** 
   * Maximum Allowed Value (for some types)
   */
  maxValueTime?: time;
  /** 
   * Maximum Allowed Value (for some types)
   */
  _maxValueTime?: Element
  /** 
   * Maximum Allowed Value (for some types)
   */
  maxValueDecimal?: decimal;
  /** 
   * Maximum Allowed Value (for some types)
   */
  _maxValueDecimal?: Element
  /** 
   * Maximum Allowed Value (for some types)
   */
  maxValueInteger?: integer;
  /** 
   * Maximum Allowed Value (for some types)
   */
  _maxValueInteger?: Element
  /** 
   * Maximum Allowed Value (for some types)
   */
  maxValuePositiveInt?: positiveInt;
  /** 
   * Maximum Allowed Value (for some types)
   */
  _maxValuePositiveInt?: Element
  /** 
   * Maximum Allowed Value (for some types)
   */
  maxValueUnsignedInt?: unsignedInt;
  /** 
   * Maximum Allowed Value (for some types)
   */
  _maxValueUnsignedInt?: Element
  /** 
   * Maximum Allowed Value (for some types)
   */
  maxValueQuantity?: Quantity;
  /** 
   * Max length for strings
   */
  maxLength?: integer;
  /** 
   * Max length for strings
   */
  _maxLength?: Element
  /** 
   * Reference to invariant about presence
   */
  condition?: Array<id>;
  /** 
   * Reference to invariant about presence
   */
  _condition?: Array<Element>
  /** 
   * Condition that must evaluate to true
   */
  constraint?: Array<ElementDefinitionConstraint>;
  /** 
   * If the element must be supported
   */
  mustSupport?: boolean;
  /** 
   * If the element must be supported
   */
  _mustSupport?: Element
  /** 
   * If this modifies the meaning of other elements
   */
  isModifier?: boolean;
  /** 
   * If this modifies the meaning of other elements
   */
  _isModifier?: Element
  /** 
   * Reason that this element is marked as a modifier
   */
  isModifierReason?: string;
  /** 
   * Reason that this element is marked as a modifier
   */
  _isModifierReason?: Element
  /** 
   * Include when _summary = true?
   */
  isSummary?: boolean;
  /** 
   * Include when _summary = true?
   */
  _isSummary?: Element
  /** 
   * ValueSet details if this is coded
   */
  binding?: ElementDefinitionBinding;
  /** 
   * Map element to another set of definitions
   */
  mapping?: Array<ElementDefinitionMapping>;
}

export interface Expression {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Natural language description of the condition
   */
  description?: string;
  /** 
   * Natural language description of the condition
   */
  _description?: Element
  /** 
   * Short name assigned to expression for reuse
   */
  name?: id;
  /** 
   * Short name assigned to expression for reuse
   */
  _name?: Element
  /** 
   * text/cql | text/fhirpath | application/x-fhir-query | text/cql-identifier | text/cql-expression | etc.
   */
  language: code;
  /** 
   * text/cql | text/fhirpath | application/x-fhir-query | text/cql-identifier | text/cql-expression | etc.
   */
  _language?: Element
  /** 
   * Expression in specified language
   */
  expression?: string;
  /** 
   * Expression in specified language
   */
  _expression?: Element
  /** 
   * Where the expression is found
   */
  reference?: uri;
  /** 
   * Where the expression is found
   */
  _reference?: Element
}

export interface Extension {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * identifies the meaning of the extension
   */
  url: id;
  /** 
   * Value of extension
   */
  valueBase64Binary?: base64Binary;
  /** 
   * Value of extension
   */
  _valueBase64Binary?: Element
  /** 
   * Value of extension
   */
  valueBoolean?: boolean;
  /** 
   * Value of extension
   */
  _valueBoolean?: Element
  /** 
   * Value of extension
   */
  valueCanonical?: canonical;
  /** 
   * Value of extension
   */
  _valueCanonical?: Element
  /** 
   * Value of extension
   */
  valueCode?: code;
  /** 
   * Value of extension
   */
  _valueCode?: Element
  /** 
   * Value of extension
   */
  valueDate?: date;
  /** 
   * Value of extension
   */
  _valueDate?: Element
  /** 
   * Value of extension
   */
  valueDateTime?: dateTime;
  /** 
   * Value of extension
   */
  _valueDateTime?: Element
  /** 
   * Value of extension
   */
  valueDecimal?: decimal;
  /** 
   * Value of extension
   */
  _valueDecimal?: Element
  /** 
   * Value of extension
   */
  valueId?: id;
  /** 
   * Value of extension
   */
  _valueId?: Element
  /** 
   * Value of extension
   */
  valueInstant?: instant;
  /** 
   * Value of extension
   */
  _valueInstant?: Element
  /** 
   * Value of extension
   */
  valueInteger?: integer;
  /** 
   * Value of extension
   */
  _valueInteger?: Element
  /** 
   * Value of extension
   */
  valueMarkdown?: markdown;
  /** 
   * Value of extension
   */
  _valueMarkdown?: Element
  /** 
   * Value of extension
   */
  valueOid?: oid;
  /** 
   * Value of extension
   */
  _valueOid?: Element
  /** 
   * Value of extension
   */
  valuePositiveInt?: positiveInt;
  /** 
   * Value of extension
   */
  _valuePositiveInt?: Element
  /** 
   * Value of extension
   */
  valueString?: string;
  /** 
   * Value of extension
   */
  _valueString?: Element
  /** 
   * Value of extension
   */
  valueTime?: time;
  /** 
   * Value of extension
   */
  _valueTime?: Element
  /** 
   * Value of extension
   */
  valueUnsignedInt?: unsignedInt;
  /** 
   * Value of extension
   */
  _valueUnsignedInt?: Element
  /** 
   * Value of extension
   */
  valueUri?: uri;
  /** 
   * Value of extension
   */
  _valueUri?: Element
  /** 
   * Value of extension
   */
  valueUrl?: url;
  /** 
   * Value of extension
   */
  _valueUrl?: Element
  /** 
   * Value of extension
   */
  valueUuid?: uuid;
  /** 
   * Value of extension
   */
  _valueUuid?: Element
  /** 
   * Value of extension
   */
  valueAddress?: Address;
  /** 
   * Value of extension
   */
  valueAge?: Age;
  /** 
   * Value of extension
   */
  valueAnnotation?: Annotation;
  /** 
   * Value of extension
   */
  valueAttachment?: Attachment;
  /** 
   * Value of extension
   */
  valueCodeableConcept?: CodeableConcept;
  /** 
   * Value of extension
   */
  valueCodeableReference?: CodeableReference;
  /** 
   * Value of extension
   */
  valueCoding?: Coding;
  /** 
   * Value of extension
   */
  valueContactPoint?: ContactPoint;
  /** 
   * Value of extension
   */
  valueCount?: Count;
  /** 
   * Value of extension
   */
  valueDistance?: Distance;
  /** 
   * Value of extension
   */
  valueDuration?: Duration;
  /** 
   * Value of extension
   */
  valueHumanName?: HumanName;
  /** 
   * Value of extension
   */
  valueIdentifier?: Identifier;
  /** 
   * Value of extension
   */
  valueMoney?: Money;
  /** 
   * Value of extension
   */
  valuePeriod?: Period;
  /** 
   * Value of extension
   */
  valueQuantity?: Quantity;
  /** 
   * Value of extension
   */
  valueRange?: Range;
  /** 
   * Value of extension
   */
  valueRatio?: Ratio;
  /** 
   * Value of extension
   */
  valueRatioRange?: RatioRange;
  /** 
   * Value of extension
   */
  valueReference?: Reference;
  /** 
   * Value of extension
   */
  valueSampledData?: SampledData;
  /** 
   * Value of extension
   */
  valueSignature?: Signature;
  /** 
   * Value of extension
   */
  valueTiming?: Timing;
  /** 
   * Value of extension
   */
  valueContactDetail?: ContactDetail;
  /** 
   * Value of extension
   */
  valueContributor?: Contributor;
  /** 
   * Value of extension
   */
  valueDataRequirement?: DataRequirement;
  /** 
   * Value of extension
   */
  valueExpression?: Expression;
  /** 
   * Value of extension
   */
  valueParameterDefinition?: ParameterDefinition;
  /** 
   * Value of extension
   */
  valueRelatedArtifact?: RelatedArtifact;
  /** 
   * Value of extension
   */
  valueTriggerDefinition?: TriggerDefinition;
  /** 
   * Value of extension
   */
  valueUsageContext?: UsageContext;
  /** 
   * Value of extension
   */
  valueDosage?: Dosage;
}

export interface HumanName {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * usual | official | temp | nickname | anonymous | old | maiden
   */
  use?: code;
  /** 
   * usual | official | temp | nickname | anonymous | old | maiden
   */
  _use?: Element
  /** 
   * Text representation of the full name
   */
  text?: string;
  /** 
   * Text representation of the full name
   */
  _text?: Element
  /** 
   * Family name (often called 'Surname')
   */
  family?: string;
  /** 
   * Family name (often called 'Surname')
   */
  _family?: Element
  /** 
   * Given names (not always 'first'). Includes middle names
   */
  given?: Array<string>;
  /** 
   * Given names (not always 'first'). Includes middle names
   */
  _given?: Array<Element>
  /** 
   * Parts that come before the name
   */
  prefix?: Array<string>;
  /** 
   * Parts that come before the name
   */
  _prefix?: Array<Element>
  /** 
   * Parts that come after the name
   */
  suffix?: Array<string>;
  /** 
   * Parts that come after the name
   */
  _suffix?: Array<Element>
  /** 
   * Time period when name was/is in use
   */
  period?: Period;
}

export interface Identifier {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * usual | official | temp | secondary | old (If known)
   */
  use?: code;
  /** 
   * usual | official | temp | secondary | old (If known)
   */
  _use?: Element
  /** 
   * Description of identifier
   */
  type?: CodeableConcept;
  /** 
   * The namespace for the identifier value
   */
  system?: uri;
  /** 
   * The namespace for the identifier value
   */
  _system?: Element
  /** 
   * The value that is unique
   */
  value?: string;
  /** 
   * The value that is unique
   */
  _value?: Element
  /** 
   * Time period when id is/was valid for use
   */
  period?: Period;
  /** 
   * Organization that issued id (may be just text)
   */
  assigner?: Reference;
}

export interface MarketingStatus {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The country in which the marketing authorisation has been granted shall be specified It should be specified using the ISO 3166 ‑ 1 alpha-2 code elements
   */
  country?: CodeableConcept;
  /** 
   * Where a Medicines Regulatory Agency has granted a marketing authorisation for which specific provisions within a jurisdiction apply, the jurisdiction can be specified using an appropriate controlled terminology The controlled term and the controlled term identifier shall be specified
   */
  jurisdiction?: CodeableConcept;
  /** 
   * This attribute provides information on the status of the marketing of the medicinal product See ISO/TS 20443 for more information and examples
   */
  status: CodeableConcept;
  /** 
   * The date when the Medicinal Product is placed on the market by the Marketing Authorisation Holder (or where applicable, the manufacturer/distributor) in a country and/or jurisdiction shall be provided A complete date consisting of day, month and year shall be specified using the ISO 8601 date format NOTE “Placed on the market” refers to the release of the Medicinal Product into the distribution chain
   */
  dateRange?: Period;
  /** 
   * The date when the Medicinal Product is placed on the market by the Marketing Authorisation Holder (or where applicable, the manufacturer/distributor) in a country and/or jurisdiction shall be provided A complete date consisting of day, month and year shall be specified using the ISO 8601 date format NOTE “Placed on the market” refers to the release of the Medicinal Product into the distribution chain
   */
  restoreDate?: dateTime;
  /** 
   * The date when the Medicinal Product is placed on the market by the Marketing Authorisation Holder (or where applicable, the manufacturer/distributor) in a country and/or jurisdiction shall be provided A complete date consisting of day, month and year shall be specified using the ISO 8601 date format NOTE “Placed on the market” refers to the release of the Medicinal Product into the distribution chain
   */
  _restoreDate?: Element
}

export interface Meta {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Version specific identifier
   */
  versionId?: id;
  /** 
   * Version specific identifier
   */
  _versionId?: Element
  /** 
   * When the resource version last changed
   */
  lastUpdated?: instant;
  /** 
   * When the resource version last changed
   */
  _lastUpdated?: Element
  /** 
   * Identifies where the resource comes from
   */
  source?: uri;
  /** 
   * Identifies where the resource comes from
   */
  _source?: Element
  /** 
   * Profiles this resource claims to conform to
   */
  profile?: Array<canonical>;
  /** 
   * Profiles this resource claims to conform to
   */
  _profile?: Array<Element>
  /** 
   * Security Labels applied to this resource
   */
  security?: Array<Coding>;
  /** 
   * Tags applied to this resource
   */
  tag?: Array<Coding>;
}

export interface Money {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Numerical value (with implicit precision)
   */
  value?: decimal;
  /** 
   * Numerical value (with implicit precision)
   */
  _value?: Element
  /** 
   * ISO 4217 Currency Code
   */
  currency?: code;
  /** 
   * ISO 4217 Currency Code
   */
  _currency?: Element
}

export interface Narrative {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * generated | extensions | additional | empty
   */
  status: code;
  /** 
   * generated | extensions | additional | empty
   */
  _status?: Element
  /** 
   * Limited xhtml content
   */
  div: xhtml;
  /** 
   * Limited xhtml content
   */
  _div?: Element
}

export interface ParameterDefinition {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Name used to access the parameter value
   */
  name?: code;
  /** 
   * Name used to access the parameter value
   */
  _name?: Element
  /** 
   * in | out
   */
  use: code;
  /** 
   * in | out
   */
  _use?: Element
  /** 
   * Minimum cardinality
   */
  min?: integer;
  /** 
   * Minimum cardinality
   */
  _min?: Element
  /** 
   * Maximum cardinality (a number of *)
   */
  max?: string;
  /** 
   * Maximum cardinality (a number of *)
   */
  _max?: Element
  /** 
   * A brief description of the parameter
   */
  documentation?: string;
  /** 
   * A brief description of the parameter
   */
  _documentation?: Element
  /** 
   * What type of value
   */
  type: code;
  /** 
   * What type of value
   */
  _type?: Element
  /** 
   * What profile the value is expected to be
   */
  profile?: canonical;
  /** 
   * What profile the value is expected to be
   */
  _profile?: Element
}

export interface Period {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Starting time with inclusive boundary
   */
  start?: dateTime;
  /** 
   * Starting time with inclusive boundary
   */
  _start?: Element
  /** 
   * End time with inclusive boundary, if not ongoing
   */
  end?: dateTime;
  /** 
   * End time with inclusive boundary, if not ongoing
   */
  _end?: Element
}

export interface Population {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The age of the specific population
   */
  ageRange?: Range;
  /** 
   * The age of the specific population
   */
  ageCodeableConcept?: CodeableConcept;
  /** 
   * The gender of the specific population
   */
  gender?: CodeableConcept;
  /** 
   * Race of the specific population
   */
  race?: CodeableConcept;
  /** 
   * The existing physiological conditions of the specific population to which this applies
   */
  physiologicalCondition?: CodeableConcept;
}

export interface ProdCharacteristic {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Where applicable, the height can be specified using a numerical value and its unit of measurement The unit of measurement shall be specified in accordance with ISO 11240 and the resulting terminology The symbol and the symbol identifier shall be used
   */
  height?: Quantity;
  /** 
   * Where applicable, the width can be specified using a numerical value and its unit of measurement The unit of measurement shall be specified in accordance with ISO 11240 and the resulting terminology The symbol and the symbol identifier shall be used
   */
  width?: Quantity;
  /** 
   * Where applicable, the depth can be specified using a numerical value and its unit of measurement The unit of measurement shall be specified in accordance with ISO 11240 and the resulting terminology The symbol and the symbol identifier shall be used
   */
  depth?: Quantity;
  /** 
   * Where applicable, the weight can be specified using a numerical value and its unit of measurement The unit of measurement shall be specified in accordance with ISO 11240 and the resulting terminology The symbol and the symbol identifier shall be used
   */
  weight?: Quantity;
  /** 
   * Where applicable, the nominal volume can be specified using a numerical value and its unit of measurement The unit of measurement shall be specified in accordance with ISO 11240 and the resulting terminology The symbol and the symbol identifier shall be used
   */
  nominalVolume?: Quantity;
  /** 
   * Where applicable, the external diameter can be specified using a numerical value and its unit of measurement The unit of measurement shall be specified in accordance with ISO 11240 and the resulting terminology The symbol and the symbol identifier shall be used
   */
  externalDiameter?: Quantity;
  /** 
   * Where applicable, the shape can be specified An appropriate controlled vocabulary shall be used The term and the term identifier shall be used
   */
  shape?: string;
  /** 
   * Where applicable, the shape can be specified An appropriate controlled vocabulary shall be used The term and the term identifier shall be used
   */
  _shape?: Element
  /** 
   * Where applicable, the color can be specified An appropriate controlled vocabulary shall be used The term and the term identifier shall be used
   */
  color?: Array<string>;
  /** 
   * Where applicable, the color can be specified An appropriate controlled vocabulary shall be used The term and the term identifier shall be used
   */
  _color?: Array<Element>
  /** 
   * Where applicable, the imprint can be specified as text
   */
  imprint?: Array<string>;
  /** 
   * Where applicable, the imprint can be specified as text
   */
  _imprint?: Array<Element>
  /** 
   * Where applicable, the image can be provided The format of the image attachment shall be specified by regional implementations
   */
  image?: Array<Attachment>;
  /** 
   * Where applicable, the scoring can be specified An appropriate controlled vocabulary shall be used The term and the term identifier shall be used
   */
  scoring?: CodeableConcept;
}

export interface ProductShelfLife {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Unique identifier for the packaged Medicinal Product
   */
  identifier?: Identifier;
  /** 
   * This describes the shelf life, taking into account various scenarios such as shelf life of the packaged Medicinal Product itself, shelf life after transformation where necessary and shelf life after the first opening of a bottle, etc. The shelf life type shall be specified using an appropriate controlled vocabulary The controlled term and the controlled term identifier shall be specified
   */
  type: CodeableConcept;
  /** 
   * The shelf life time period can be specified using a numerical value for the period of time and its unit of time measurement The unit of measurement shall be specified in accordance with ISO 11240 and the resulting terminology The symbol and the symbol identifier shall be used
   */
  period: Quantity;
  /** 
   * Special precautions for storage, if any, can be specified using an appropriate controlled vocabulary The controlled term and the controlled term identifier shall be specified
   */
  specialPrecautionsForStorage?: Array<CodeableConcept>;
}

export interface Quantity {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Numerical value (with implicit precision)
   */
  value?: decimal;
  /** 
   * Numerical value (with implicit precision)
   */
  _value?: Element
  /** 
   * < | <= | >= | > - how to understand the value
   */
  comparator?: code;
  /** 
   * < | <= | >= | > - how to understand the value
   */
  _comparator?: Element
  /** 
   * Unit representation
   */
  unit?: string;
  /** 
   * Unit representation
   */
  _unit?: Element
  /** 
   * System that defines coded unit form
   */
  system?: uri;
  /** 
   * System that defines coded unit form
   */
  _system?: Element
  /** 
   * Coded form of the unit
   */
  code?: code;
  /** 
   * Coded form of the unit
   */
  _code?: Element
}

export interface Range {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Low limit
   */
  low?: Quantity;
  /** 
   * High limit
   */
  high?: Quantity;
}

export interface Ratio {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Numerator value
   */
  numerator?: Quantity;
  /** 
   * Denominator value
   */
  denominator?: Quantity;
}

export interface RatioRange {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Low Numerator limit
   */
  lowNumerator?: Quantity;
  /** 
   * High Numerator limit
   */
  highNumerator?: Quantity;
  /** 
   * Denominator value
   */
  denominator?: Quantity;
}

export interface Reference {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Literal reference, Relative, internal or absolute URL
   */
  reference?: string;
  /** 
   * Literal reference, Relative, internal or absolute URL
   */
  _reference?: Element
  /** 
   * Type the reference refers to (e.g. "Patient")
   */
  type?: uri;
  /** 
   * Type the reference refers to (e.g. "Patient")
   */
  _type?: Element
  /** 
   * Logical reference, when literal reference is not known
   */
  identifier?: Identifier;
  /** 
   * Text alternative for the resource
   */
  display?: string;
  /** 
   * Text alternative for the resource
   */
  _display?: Element
}

export interface RelatedArtifact {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * documentation | justification | citation | predecessor | successor | derived-from | depends-on | composed-of
   */
  type: code;
  /** 
   * documentation | justification | citation | predecessor | successor | derived-from | depends-on | composed-of
   */
  _type?: Element
  /** 
   * Short label
   */
  label?: string;
  /** 
   * Short label
   */
  _label?: Element
  /** 
   * Brief description of the related artifact
   */
  display?: string;
  /** 
   * Brief description of the related artifact
   */
  _display?: Element
  /** 
   * Bibliographic citation for the artifact
   */
  citation?: markdown;
  /** 
   * Bibliographic citation for the artifact
   */
  _citation?: Element
  /** 
   * Where the artifact can be accessed
   */
  url?: url;
  /** 
   * Where the artifact can be accessed
   */
  _url?: Element
  /** 
   * What document is being referenced
   */
  document?: Attachment;
  /** 
   * What resource is being referenced
   */
  resource?: canonical;
  /** 
   * What resource is being referenced
   */
  _resource?: Element
}

export interface SampledData {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Zero value and units
   */
  origin: Quantity;
  /** 
   * Number of milliseconds between samples
   */
  period: decimal;
  /** 
   * Number of milliseconds between samples
   */
  _period?: Element
  /** 
   * Multiply data by this before adding to origin
   */
  factor?: decimal;
  /** 
   * Multiply data by this before adding to origin
   */
  _factor?: Element
  /** 
   * Lower limit of detection
   */
  lowerLimit?: decimal;
  /** 
   * Lower limit of detection
   */
  _lowerLimit?: Element
  /** 
   * Upper limit of detection
   */
  upperLimit?: decimal;
  /** 
   * Upper limit of detection
   */
  _upperLimit?: Element
  /** 
   * Number of sample points at each time point
   */
  dimensions: positiveInt;
  /** 
   * Number of sample points at each time point
   */
  _dimensions?: Element
  /** 
   * Decimal values with spaces, or "E" | "U" | "L"
   */
  data?: string;
  /** 
   * Decimal values with spaces, or "E" | "U" | "L"
   */
  _data?: Element
}

export interface Signature {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Indication of the reason the entity signed the object(s)
   */
  type: Array<Coding>;
  /** 
   * When the signature was created
   */
  when: instant;
  /** 
   * When the signature was created
   */
  _when?: Element
  /** 
   * Who signed
   */
  who: Reference;
  /** 
   * The party represented
   */
  onBehalfOf?: Reference;
  /** 
   * The technical format of the signed resources
   */
  targetFormat?: code;
  /** 
   * The technical format of the signed resources
   */
  _targetFormat?: Element
  /** 
   * The technical format of the signature
   */
  sigFormat?: code;
  /** 
   * The technical format of the signature
   */
  _sigFormat?: Element
  /** 
   * The actual signature content (XML DigSig. JWS, picture, etc.)
   */
  data?: base64Binary;
  /** 
   * The actual signature content (XML DigSig. JWS, picture, etc.)
   */
  _data?: Element
}

export interface TimingRepeat {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Length/Range of lengths, or (Start and/or end) limits
   */
  boundsDuration?: Duration;
  /** 
   * Length/Range of lengths, or (Start and/or end) limits
   */
  boundsRange?: Range;
  /** 
   * Length/Range of lengths, or (Start and/or end) limits
   */
  boundsPeriod?: Period;
  /** 
   * Number of times to repeat
   */
  count?: positiveInt;
  /** 
   * Number of times to repeat
   */
  _count?: Element
  /** 
   * Maximum number of times to repeat
   */
  countMax?: positiveInt;
  /** 
   * Maximum number of times to repeat
   */
  _countMax?: Element
  /** 
   * How long when it happens
   */
  duration?: decimal;
  /** 
   * How long when it happens
   */
  _duration?: Element
  /** 
   * How long when it happens (Max)
   */
  durationMax?: decimal;
  /** 
   * How long when it happens (Max)
   */
  _durationMax?: Element
  /** 
   * s | min | h | d | wk | mo | a - unit of time (UCUM)
   */
  durationUnit?: code;
  /** 
   * s | min | h | d | wk | mo | a - unit of time (UCUM)
   */
  _durationUnit?: Element
  /** 
   * Event occurs frequency times per period
   */
  frequency?: positiveInt;
  /** 
   * Event occurs frequency times per period
   */
  _frequency?: Element
  /** 
   * Event occurs up to frequencyMax times per period
   */
  frequencyMax?: positiveInt;
  /** 
   * Event occurs up to frequencyMax times per period
   */
  _frequencyMax?: Element
  /** 
   * Event occurs frequency times per period
   */
  period?: decimal;
  /** 
   * Event occurs frequency times per period
   */
  _period?: Element
  /** 
   * Upper limit of period (3-4 hours)
   */
  periodMax?: decimal;
  /** 
   * Upper limit of period (3-4 hours)
   */
  _periodMax?: Element
  /** 
   * s | min | h | d | wk | mo | a - unit of time (UCUM)
   */
  periodUnit?: code;
  /** 
   * s | min | h | d | wk | mo | a - unit of time (UCUM)
   */
  _periodUnit?: Element
  /** 
   * mon | tue | wed | thu | fri | sat | sun
   */
  dayOfWeek?: Array<code>;
  /** 
   * mon | tue | wed | thu | fri | sat | sun
   */
  _dayOfWeek?: Array<Element>
  /** 
   * Time of day for action
   */
  timeOfDay?: Array<time>;
  /** 
   * Time of day for action
   */
  _timeOfDay?: Array<Element>
  /** 
   * Code for time period of occurrence
   */
  when?: Array<code>;
  /** 
   * Code for time period of occurrence
   */
  _when?: Array<Element>
  /** 
   * Minutes from event (before or after)
   */
  offset?: unsignedInt;
  /** 
   * Minutes from event (before or after)
   */
  _offset?: Element
}
export interface Timing {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * When the event occurs
   */
  event?: Array<dateTime>;
  /** 
   * When the event occurs
   */
  _event?: Array<Element>
  /** 
   * When the event is to occur
   */
  repeat?: TimingRepeat;
  /** 
   * BID | TID | QID | AM | PM | QD | QOD | +
   */
  code?: CodeableConcept;
}

export interface TriggerDefinition {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * named-event | periodic | data-changed | data-added | data-modified | data-removed | data-accessed | data-access-ended
   */
  type: code;
  /** 
   * named-event | periodic | data-changed | data-added | data-modified | data-removed | data-accessed | data-access-ended
   */
  _type?: Element
  /** 
   * Name or URI that identifies the event
   */
  name?: string;
  /** 
   * Name or URI that identifies the event
   */
  _name?: Element
  /** 
   * Timing of the event
   */
  timingTiming?: Timing;
  /** 
   * Timing of the event
   */
  timingReference?: Reference;
  /** 
   * Timing of the event
   */
  timingDate?: date;
  /** 
   * Timing of the event
   */
  _timingDate?: Element
  /** 
   * Timing of the event
   */
  timingDateTime?: dateTime;
  /** 
   * Timing of the event
   */
  _timingDateTime?: Element
  /** 
   * Triggering data of the event (multiple = 'and')
   */
  data?: Array<DataRequirement>;
  /** 
   * Whether the event triggers (boolean expression)
   */
  condition?: Expression;
}

export interface UsageContext {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Type of context being specified
   */
  code: Coding;
  /** 
   * Value that defines the context
   */
  valueCodeableConcept?: CodeableConcept;
  /** 
   * Value that defines the context
   */
  valueQuantity?: Quantity;
  /** 
   * Value that defines the context
   */
  valueRange?: Range;
  /** 
   * Value that defines the context
   */
  valueReference?: Reference;
}

export interface AccountCoverage {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The party(s), such as insurances, that may contribute to the payment of this account
   */
  coverage: Reference;
  /** 
   * The priority of the coverage in the context of this account
   */
  priority?: positiveInt;
  /** 
   * The priority of the coverage in the context of this account
   */
  _priority?: Element
}
export interface AccountGuarantor {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Responsible entity
   */
  party: Reference;
  /** 
   * Credit or other hold applied
   */
  onHold?: boolean;
  /** 
   * Credit or other hold applied
   */
  _onHold?: Element
  /** 
   * Guarantee account during
   */
  period?: Period;
}
export interface Account {
resourceType: "Account"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Account number
   */
  identifier?: Array<Identifier>;
  /** 
   * active | inactive | entered-in-error | on-hold | unknown
   */
  status: code;
  /** 
   * active | inactive | entered-in-error | on-hold | unknown
   */
  _status?: Element
  /** 
   * E.g. patient, expense, depreciation
   */
  type?: CodeableConcept;
  /** 
   * Human-readable label
   */
  name?: string;
  /** 
   * Human-readable label
   */
  _name?: Element
  /** 
   * The entity that caused the expenses
   */
  subject?: Array<Reference>;
  /** 
   * Transaction window
   */
  servicePeriod?: Period;
  /** 
   * The party(s) that are responsible for covering the payment of this account, and what order should they be applied to the account
   */
  coverage?: Array<AccountCoverage>;
  /** 
   * Entity managing the Account
   */
  owner?: Reference;
  /** 
   * Explanation of purpose/use
   */
  description?: string;
  /** 
   * Explanation of purpose/use
   */
  _description?: Element
  /** 
   * The parties ultimately responsible for balancing the Account
   */
  guarantor?: Array<AccountGuarantor>;
  /** 
   * Reference to a parent Account
   */
  partOf?: Reference;
}

export interface ActivityDefinitionParticipant {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * patient | practitioner | related-person | device
   */
  type: code;
  /** 
   * patient | practitioner | related-person | device
   */
  _type?: Element
  /** 
   * E.g. Nurse, Surgeon, Parent, etc.
   */
  role?: CodeableConcept;
}
export interface ActivityDefinitionDynamicValue {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The path to the element to be set dynamically
   */
  path: string;
  /** 
   * The path to the element to be set dynamically
   */
  _path?: Element
  /** 
   * An expression that provides the dynamic value for the customization
   */
  expression: Expression;
}
export interface ActivityDefinition {
resourceType: "ActivityDefinition"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Canonical identifier for this activity definition, represented as a URI (globally unique)
   */
  url?: uri;
  /** 
   * Canonical identifier for this activity definition, represented as a URI (globally unique)
   */
  _url?: Element
  /** 
   * Additional identifier for the activity definition
   */
  identifier?: Array<Identifier>;
  /** 
   * Business version of the activity definition
   */
  version?: string;
  /** 
   * Business version of the activity definition
   */
  _version?: Element
  /** 
   * Name for this activity definition (computer friendly)
   */
  name?: string;
  /** 
   * Name for this activity definition (computer friendly)
   */
  _name?: Element
  /** 
   * Name for this activity definition (human friendly)
   */
  title?: string;
  /** 
   * Name for this activity definition (human friendly)
   */
  _title?: Element
  /** 
   * Subordinate title of the activity definition
   */
  subtitle?: string;
  /** 
   * Subordinate title of the activity definition
   */
  _subtitle?: Element
  /** 
   * draft | active | retired | unknown
   */
  status: code;
  /** 
   * draft | active | retired | unknown
   */
  _status?: Element
  /** 
   * For testing purposes, not real usage
   */
  experimental?: boolean;
  /** 
   * For testing purposes, not real usage
   */
  _experimental?: Element
  /** 
   * Type of individual the activity definition is intended for
   */
  subjectCodeableConcept?: CodeableConcept;
  /** 
   * Type of individual the activity definition is intended for
   */
  subjectReference?: Reference;
  /** 
   * Type of individual the activity definition is intended for
   */
  subjectCanonical?: canonical;
  /** 
   * Type of individual the activity definition is intended for
   */
  _subjectCanonical?: Element
  /** 
   * Date last changed
   */
  date?: dateTime;
  /** 
   * Date last changed
   */
  _date?: Element
  /** 
   * Name of the publisher (organization or individual)
   */
  publisher?: string;
  /** 
   * Name of the publisher (organization or individual)
   */
  _publisher?: Element
  /** 
   * Contact details for the publisher
   */
  contact?: Array<ContactDetail>;
  /** 
   * Natural language description of the activity definition
   */
  description?: markdown;
  /** 
   * Natural language description of the activity definition
   */
  _description?: Element
  /** 
   * The context that the content is intended to support
   */
  useContext?: Array<UsageContext>;
  /** 
   * Intended jurisdiction for activity definition (if applicable)
   */
  jurisdiction?: Array<CodeableConcept>;
  /** 
   * Why this activity definition is defined
   */
  purpose?: markdown;
  /** 
   * Why this activity definition is defined
   */
  _purpose?: Element
  /** 
   * Describes the clinical usage of the activity definition
   */
  usage?: string;
  /** 
   * Describes the clinical usage of the activity definition
   */
  _usage?: Element
  /** 
   * Use and/or publishing restrictions
   */
  copyright?: markdown;
  /** 
   * Use and/or publishing restrictions
   */
  _copyright?: Element
  /** 
   * When the activity definition was approved by publisher
   */
  approvalDate?: date;
  /** 
   * When the activity definition was approved by publisher
   */
  _approvalDate?: Element
  /** 
   * When the activity definition was last reviewed
   */
  lastReviewDate?: date;
  /** 
   * When the activity definition was last reviewed
   */
  _lastReviewDate?: Element
  /** 
   * When the activity definition is expected to be used
   */
  effectivePeriod?: Period;
  /** 
   * E.g. Education, Treatment, Assessment, etc.
   */
  topic?: Array<CodeableConcept>;
  /** 
   * Who authored the content
   */
  author?: Array<ContactDetail>;
  /** 
   * Who edited the content
   */
  editor?: Array<ContactDetail>;
  /** 
   * Who reviewed the content
   */
  reviewer?: Array<ContactDetail>;
  /** 
   * Who endorsed the content
   */
  endorser?: Array<ContactDetail>;
  /** 
   * Additional documentation, citations, etc.
   */
  relatedArtifact?: Array<RelatedArtifact>;
  /** 
   * Logic used by the activity definition
   */
  library?: Array<canonical>;
  /** 
   * Logic used by the activity definition
   */
  _library?: Array<Element>
  /** 
   * Kind of resource
   */
  kind?: code;
  /** 
   * Kind of resource
   */
  _kind?: Element
  /** 
   * What profile the resource needs to conform to
   */
  profile?: canonical;
  /** 
   * What profile the resource needs to conform to
   */
  _profile?: Element
  /** 
   * Detail type of activity
   */
  code?: CodeableConcept;
  /** 
   * proposal | plan | directive | order | original-order | reflex-order | filler-order | instance-order | option
   */
  intent?: code;
  /** 
   * proposal | plan | directive | order | original-order | reflex-order | filler-order | instance-order | option
   */
  _intent?: Element
  /** 
   * routine | urgent | asap | stat
   */
  priority?: code;
  /** 
   * routine | urgent | asap | stat
   */
  _priority?: Element
  /** 
   * True if the activity should not be performed
   */
  doNotPerform?: boolean;
  /** 
   * True if the activity should not be performed
   */
  _doNotPerform?: Element
  /** 
   * When activity is to occur
   */
  timingTiming?: Timing;
  /** 
   * When activity is to occur
   */
  timingDateTime?: dateTime;
  /** 
   * When activity is to occur
   */
  _timingDateTime?: Element
  /** 
   * When activity is to occur
   */
  timingAge?: Age;
  /** 
   * When activity is to occur
   */
  timingPeriod?: Period;
  /** 
   * When activity is to occur
   */
  timingRange?: Range;
  /** 
   * When activity is to occur
   */
  timingDuration?: Duration;
  /** 
   * Where it should happen
   */
  location?: Reference;
  /** 
   * Who should participate in the action
   */
  participant?: Array<ActivityDefinitionParticipant>;
  /** 
   * What's administered/supplied
   */
  productReference?: Reference;
  /** 
   * What's administered/supplied
   */
  productCodeableConcept?: CodeableConcept;
  /** 
   * How much is administered/consumed/supplied
   */
  quantity?: Quantity;
  /** 
   * Detailed dosage instructions
   */
  dosage?: Array<Dosage>;
  /** 
   * What part of body to perform on
   */
  bodySite?: Array<CodeableConcept>;
  /** 
   * What specimens are required to perform this action
   */
  specimenRequirement?: Array<Reference>;
  /** 
   * What observations are required to perform this action
   */
  observationRequirement?: Array<Reference>;
  /** 
   * What observations must be produced by this action
   */
  observationResultRequirement?: Array<Reference>;
  /** 
   * Transform to apply the template
   */
  transform?: canonical;
  /** 
   * Transform to apply the template
   */
  _transform?: Element
  /** 
   * Dynamic aspects of the definition
   */
  dynamicValue?: Array<ActivityDefinitionDynamicValue>;
}

export interface AdministrableProductDefinitionProperty {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * A code expressing the type of characteristic
   */
  type: CodeableConcept;
  /** 
   * A value for the characteristic
   */
  valueCodeableConcept?: CodeableConcept;
  /** 
   * A value for the characteristic
   */
  valueQuantity?: Quantity;
  /** 
   * A value for the characteristic
   */
  valueDate?: date;
  /** 
   * A value for the characteristic
   */
  _valueDate?: Element
  /** 
   * A value for the characteristic
   */
  valueBoolean?: boolean;
  /** 
   * A value for the characteristic
   */
  _valueBoolean?: Element
  /** 
   * A value for the characteristic
   */
  valueAttachment?: Attachment;
  /** 
   * The status of characteristic e.g. assigned or pending
   */
  status?: CodeableConcept;
}
export interface AdministrableProductDefinitionRouteOfAdministrationTargetSpeciesWithdrawalPeriod {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The type of tissue for which the withdrawal period applies, e.g. meat, milk
   */
  tissue: CodeableConcept;
  /** 
   * A value for the time
   */
  value: Quantity;
  /** 
   * Extra information about the withdrawal period
   */
  supportingInformation?: string;
  /** 
   * Extra information about the withdrawal period
   */
  _supportingInformation?: Element
}
export interface AdministrableProductDefinitionRouteOfAdministrationTargetSpecies {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Coded expression for the species
   */
  code: CodeableConcept;
  /** 
   * A species specific time during which consumption of animal product is not appropriate
   */
  withdrawalPeriod?: Array<AdministrableProductDefinitionRouteOfAdministrationTargetSpeciesWithdrawalPeriod>;
}
export interface AdministrableProductDefinitionRouteOfAdministration {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Coded expression for the route
   */
  code: CodeableConcept;
  /** 
   * The first dose (dose quantity) administered can be specified for the product
   */
  firstDose?: Quantity;
  /** 
   * The maximum single dose that can be administered
   */
  maxSingleDose?: Quantity;
  /** 
   * The maximum dose quantity to be administered in any one 24-h period
   */
  maxDosePerDay?: Quantity;
  /** 
   * The maximum dose per treatment period that can be administered
   */
  maxDosePerTreatmentPeriod?: Ratio;
  /** 
   * The maximum treatment period during which the product can be administered
   */
  maxTreatmentPeriod?: Duration;
  /** 
   * A species for which this route applies
   */
  targetSpecies?: Array<AdministrableProductDefinitionRouteOfAdministrationTargetSpecies>;
}
export interface AdministrableProductDefinition {
resourceType: "AdministrableProductDefinition"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * An identifier for the administrable product
   */
  identifier?: Array<Identifier>;
  /** 
   * draft | active | retired | unknown
   */
  status: code;
  /** 
   * draft | active | retired | unknown
   */
  _status?: Element
  /** 
   * References a product from which one or more of the constituent parts of that product can be prepared and used as described by this administrable product
   */
  formOf?: Array<Reference>;
  /** 
   * The dose form of the final product after necessary reconstitution or processing
   */
  administrableDoseForm?: CodeableConcept;
  /** 
   * The presentation type in which this item is given to a patient. e.g. for a spray - 'puff'
   */
  unitOfPresentation?: CodeableConcept;
  /** 
   * Indicates the specific manufactured items that are part of the 'formOf' product that are used in the preparation of this specific administrable form
   */
  producedFrom?: Array<Reference>;
  /** 
   * The ingredients of this administrable medicinal product. This is only needed if the ingredients are not specified either using ManufacturedItemDefiniton, or using by incoming references from the Ingredient resource
   */
  ingredient?: Array<CodeableConcept>;
  /** 
   * A device that is integral to the medicinal product, in effect being considered as an "ingredient" of the medicinal product
   */
  device?: Reference;
  /** 
   * Characteristics e.g. a product's onset of action
   */
  property?: Array<AdministrableProductDefinitionProperty>;
  /** 
   * The path by which the product is taken into or makes contact with the body
   */
  routeOfAdministration: Array<AdministrableProductDefinitionRouteOfAdministration>;
}

export interface AdverseEventSuspectEntityCausality {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Assessment of if the entity caused the event
   */
  assessment?: CodeableConcept;
  /** 
   * AdverseEvent.suspectEntity.causalityProductRelatedness
   */
  productRelatedness?: string;
  /** 
   * AdverseEvent.suspectEntity.causalityProductRelatedness
   */
  _productRelatedness?: Element
  /** 
   * AdverseEvent.suspectEntity.causalityAuthor
   */
  author?: Reference;
  /** 
   * ProbabilityScale | Bayesian | Checklist
   */
  method?: CodeableConcept;
}
export interface AdverseEventSuspectEntity {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Refers to the specific entity that caused the adverse event
   */
  instance: Reference;
  /** 
   * Information on the possible cause of the event
   */
  causality?: Array<AdverseEventSuspectEntityCausality>;
}
export interface AdverseEvent {
resourceType: "AdverseEvent"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business identifier for the event
   */
  identifier?: Identifier;
  /** 
   * actual | potential
   */
  actuality: code;
  /** 
   * actual | potential
   */
  _actuality?: Element
  /** 
   * product-problem | product-quality | product-use-error | wrong-dose | incorrect-prescribing-information | wrong-technique | wrong-route-of-administration | wrong-rate | wrong-duration | wrong-time | expired-drug | medical-device-use-error | problem-different-manufacturer | unsafe-physical-environment
   */
  category?: Array<CodeableConcept>;
  /** 
   * Type of the event itself in relation to the subject
   */
  event?: CodeableConcept;
  /** 
   * Subject impacted by event
   */
  subject: Reference;
  /** 
   * Encounter created as part of
   */
  encounter?: Reference;
  /** 
   * When the event occurred
   */
  date?: dateTime;
  /** 
   * When the event occurred
   */
  _date?: Element
  /** 
   * When the event was detected
   */
  detected?: dateTime;
  /** 
   * When the event was detected
   */
  _detected?: Element
  /** 
   * When the event was recorded
   */
  recordedDate?: dateTime;
  /** 
   * When the event was recorded
   */
  _recordedDate?: Element
  /** 
   * Effect on the subject due to this event
   */
  resultingCondition?: Array<Reference>;
  /** 
   * Location where adverse event occurred
   */
  location?: Reference;
  /** 
   * Seriousness of the event
   */
  seriousness?: CodeableConcept;
  /** 
   * mild | moderate | severe
   */
  severity?: CodeableConcept;
  /** 
   * resolved | recovering | ongoing | resolvedWithSequelae | fatal | unknown
   */
  outcome?: CodeableConcept;
  /** 
   * Who recorded the adverse event
   */
  recorder?: Reference;
  /** 
   * Who  was involved in the adverse event or the potential adverse event
   */
  contributor?: Array<Reference>;
  /** 
   * The suspected agent causing the adverse event
   */
  suspectEntity?: Array<AdverseEventSuspectEntity>;
  /** 
   * AdverseEvent.subjectMedicalHistory
   */
  subjectMedicalHistory?: Array<Reference>;
  /** 
   * AdverseEvent.referenceDocument
   */
  referenceDocument?: Array<Reference>;
  /** 
   * AdverseEvent.study
   */
  study?: Array<Reference>;
}

export interface AllergyIntoleranceReaction {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Specific substance or pharmaceutical product considered to be responsible for event
   */
  substance?: CodeableConcept;
  /** 
   * Clinical symptoms/signs associated with the Event
   */
  manifestation: Array<CodeableConcept>;
  /** 
   * Description of the event as a whole
   */
  description?: string;
  /** 
   * Description of the event as a whole
   */
  _description?: Element
  /** 
   * Date(/time) when manifestations showed
   */
  onset?: dateTime;
  /** 
   * Date(/time) when manifestations showed
   */
  _onset?: Element
  /** 
   * mild | moderate | severe (of event as a whole)
   */
  severity?: code;
  /** 
   * mild | moderate | severe (of event as a whole)
   */
  _severity?: Element
  /** 
   * How the subject was exposed to the substance
   */
  exposureRoute?: CodeableConcept;
  /** 
   * Text about event not captured in other fields
   */
  note?: Array<Annotation>;
}
export interface AllergyIntolerance {
resourceType: "AllergyIntolerance"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * External ids for this item
   */
  identifier?: Array<Identifier>;
  /** 
   * active | inactive | resolved
   */
  clinicalStatus?: CodeableConcept;
  /** 
   * unconfirmed | confirmed | refuted | entered-in-error
   */
  verificationStatus?: CodeableConcept;
  /** 
   * allergy | intolerance - Underlying mechanism (if known)
   */
  type?: code;
  /** 
   * allergy | intolerance - Underlying mechanism (if known)
   */
  _type?: Element
  /** 
   * food | medication | environment | biologic
   */
  category?: Array<code>;
  /** 
   * food | medication | environment | biologic
   */
  _category?: Array<Element>
  /** 
   * low | high | unable-to-assess
   */
  criticality?: code;
  /** 
   * low | high | unable-to-assess
   */
  _criticality?: Element
  /** 
   * Code that identifies the allergy or intolerance
   */
  code?: CodeableConcept;
  /** 
   * Who the sensitivity is for
   */
  patient: Reference;
  /** 
   * Encounter when the allergy or intolerance was asserted
   */
  encounter?: Reference;
  /** 
   * When allergy or intolerance was identified
   */
  onsetDateTime?: dateTime;
  /** 
   * When allergy or intolerance was identified
   */
  _onsetDateTime?: Element
  /** 
   * When allergy or intolerance was identified
   */
  onsetAge?: Age;
  /** 
   * When allergy or intolerance was identified
   */
  onsetPeriod?: Period;
  /** 
   * When allergy or intolerance was identified
   */
  onsetRange?: Range;
  /** 
   * When allergy or intolerance was identified
   */
  onsetString?: string;
  /** 
   * When allergy or intolerance was identified
   */
  _onsetString?: Element
  /** 
   * Date first version of the resource instance was recorded
   */
  recordedDate?: dateTime;
  /** 
   * Date first version of the resource instance was recorded
   */
  _recordedDate?: Element
  /** 
   * Who recorded the sensitivity
   */
  recorder?: Reference;
  /** 
   * Source of the information about the allergy
   */
  asserter?: Reference;
  /** 
   * Date(/time) of last known occurrence of a reaction
   */
  lastOccurrence?: dateTime;
  /** 
   * Date(/time) of last known occurrence of a reaction
   */
  _lastOccurrence?: Element
  /** 
   * Additional text not captured in other fields
   */
  note?: Array<Annotation>;
  /** 
   * Adverse Reaction Events linked to exposure to substance
   */
  reaction?: Array<AllergyIntoleranceReaction>;
}

export interface AppointmentParticipant {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Role of participant in the appointment
   */
  type?: Array<CodeableConcept>;
  /** 
   * Person, Location/HealthcareService or Device
   */
  actor?: Reference;
  /** 
   * required | optional | information-only
   */
  required?: code;
  /** 
   * required | optional | information-only
   */
  _required?: Element
  /** 
   * accepted | declined | tentative | needs-action
   */
  status: code;
  /** 
   * accepted | declined | tentative | needs-action
   */
  _status?: Element
  /** 
   * Participation period of the actor
   */
  period?: Period;
}
export interface Appointment {
resourceType: "Appointment"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * External Ids for this item
   */
  identifier?: Array<Identifier>;
  /** 
   * proposed | pending | booked | arrived | fulfilled | cancelled | noshow | entered-in-error | checked-in | waitlist
   */
  status: code;
  /** 
   * proposed | pending | booked | arrived | fulfilled | cancelled | noshow | entered-in-error | checked-in | waitlist
   */
  _status?: Element
  /** 
   * The coded reason for the appointment being cancelled
   */
  cancelationReason?: CodeableConcept;
  /** 
   * A broad categorization of the service that is to be performed during this appointment
   */
  serviceCategory?: Array<CodeableConcept>;
  /** 
   * The specific service that is to be performed during this appointment
   */
  serviceType?: Array<CodeableConcept>;
  /** 
   * The specialty of a practitioner that would be required to perform the service requested in this appointment
   */
  specialty?: Array<CodeableConcept>;
  /** 
   * The style of appointment or patient that has been booked in the slot (not service type)
   */
  appointmentType?: CodeableConcept;
  /** 
   * Coded reason this appointment is scheduled
   */
  reasonCode?: Array<CodeableConcept>;
  /** 
   * Reason the appointment is to take place (resource)
   */
  reasonReference?: Array<Reference>;
  /** 
   * Used to make informed decisions if needing to re-prioritize
   */
  priority?: unsignedInt;
  /** 
   * Used to make informed decisions if needing to re-prioritize
   */
  _priority?: Element
  /** 
   * Shown on a subject line in a meeting request, or appointment list
   */
  description?: string;
  /** 
   * Shown on a subject line in a meeting request, or appointment list
   */
  _description?: Element
  /** 
   * Additional information to support the appointment
   */
  supportingInformation?: Array<Reference>;
  /** 
   * When appointment is to take place
   */
  start?: instant;
  /** 
   * When appointment is to take place
   */
  _start?: Element
  /** 
   * When appointment is to conclude
   */
  end?: instant;
  /** 
   * When appointment is to conclude
   */
  _end?: Element
  /** 
   * Can be less than start/end (e.g. estimate)
   */
  minutesDuration?: positiveInt;
  /** 
   * Can be less than start/end (e.g. estimate)
   */
  _minutesDuration?: Element
  /** 
   * The slots that this appointment is filling
   */
  slot?: Array<Reference>;
  /** 
   * The date that this appointment was initially created
   */
  created?: dateTime;
  /** 
   * The date that this appointment was initially created
   */
  _created?: Element
  /** 
   * Additional comments
   */
  comment?: string;
  /** 
   * Additional comments
   */
  _comment?: Element
  /** 
   * Detailed information and instructions for the patient
   */
  patientInstruction?: string;
  /** 
   * Detailed information and instructions for the patient
   */
  _patientInstruction?: Element
  /** 
   * The service request this appointment is allocated to assess
   */
  basedOn?: Array<Reference>;
  /** 
   * Participants involved in appointment
   */
  participant: Array<AppointmentParticipant>;
  /** 
   * Potential date/time interval(s) requested to allocate the appointment within
   */
  requestedPeriod?: Array<Period>;
}

export interface AppointmentResponse {
resourceType: "AppointmentResponse"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * External Ids for this item
   */
  identifier?: Array<Identifier>;
  /** 
   * Appointment this response relates to
   */
  appointment: Reference;
  /** 
   * Time from appointment, or requested new start time
   */
  start?: instant;
  /** 
   * Time from appointment, or requested new start time
   */
  _start?: Element
  /** 
   * Time from appointment, or requested new end time
   */
  end?: instant;
  /** 
   * Time from appointment, or requested new end time
   */
  _end?: Element
  /** 
   * Role of participant in the appointment
   */
  participantType?: Array<CodeableConcept>;
  /** 
   * Person, Location, HealthcareService, or Device
   */
  actor?: Reference;
  /** 
   * accepted | declined | tentative | needs-action
   */
  participantStatus: code;
  /** 
   * accepted | declined | tentative | needs-action
   */
  _participantStatus?: Element
  /** 
   * Additional comments
   */
  comment?: string;
  /** 
   * Additional comments
   */
  _comment?: Element
}

export interface AuditEventAgentNetwork {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Identifier for the network access point of the user device
   */
  address?: string;
  /** 
   * Identifier for the network access point of the user device
   */
  _address?: Element
  /** 
   * The type of network access point
   */
  type?: code;
  /** 
   * The type of network access point
   */
  _type?: Element
}
export interface AuditEventAgent {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * How agent participated
   */
  type?: CodeableConcept;
  /** 
   * Agent role in the event
   */
  role?: Array<CodeableConcept>;
  /** 
   * Identifier of who
   */
  who?: Reference;
  /** 
   * Alternative User identity
   */
  altId?: string;
  /** 
   * Alternative User identity
   */
  _altId?: Element
  /** 
   * Human friendly name for the agent
   */
  name?: string;
  /** 
   * Human friendly name for the agent
   */
  _name?: Element
  /** 
   * Whether user is initiator
   */
  requestor: boolean;
  /** 
   * Whether user is initiator
   */
  _requestor?: Element
  /** 
   * Where
   */
  location?: Reference;
  /** 
   * Policy that authorized event
   */
  policy?: Array<uri>;
  /** 
   * Policy that authorized event
   */
  _policy?: Array<Element>
  /** 
   * Type of media
   */
  media?: Coding;
  /** 
   * Logical network location for application activity
   */
  network?: AuditEventAgentNetwork;
  /** 
   * Reason given for this user
   */
  purposeOfUse?: Array<CodeableConcept>;
}
export interface AuditEventSource {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Logical source location within the enterprise
   */
  site?: string;
  /** 
   * Logical source location within the enterprise
   */
  _site?: Element
  /** 
   * The identity of source detecting the event
   */
  observer: Reference;
  /** 
   * The type of source where event originated
   */
  type?: Array<Coding>;
}
export interface AuditEventEntityDetail {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Name of the property
   */
  type: string;
  /** 
   * Name of the property
   */
  _type?: Element
  /** 
   * Property value
   */
  valueString?: string;
  /** 
   * Property value
   */
  _valueString?: Element
  /** 
   * Property value
   */
  valueBase64Binary?: base64Binary;
  /** 
   * Property value
   */
  _valueBase64Binary?: Element
}
export interface AuditEventEntity {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Specific instance of resource
   */
  what?: Reference;
  /** 
   * Type of entity involved
   */
  type?: Coding;
  /** 
   * What role the entity played
   */
  role?: Coding;
  /** 
   * Life-cycle stage for the entity
   */
  lifecycle?: Coding;
  /** 
   * Security labels on the entity
   */
  securityLabel?: Array<Coding>;
  /** 
   * Descriptor for entity
   */
  name?: string;
  /** 
   * Descriptor for entity
   */
  _name?: Element
  /** 
   * Descriptive text
   */
  description?: string;
  /** 
   * Descriptive text
   */
  _description?: Element
  /** 
   * Query parameters
   */
  query?: base64Binary;
  /** 
   * Query parameters
   */
  _query?: Element
  /** 
   * Additional Information about the entity
   */
  detail?: Array<AuditEventEntityDetail>;
}
export interface AuditEvent {
resourceType: "AuditEvent"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Type/identifier of event
   */
  type: Coding;
  /** 
   * More specific type/id for the event
   */
  subtype?: Array<Coding>;
  /** 
   * Type of action performed during the event
   */
  action?: code;
  /** 
   * Type of action performed during the event
   */
  _action?: Element
  /** 
   * When the activity occurred
   */
  period?: Period;
  /** 
   * Time when the event was recorded
   */
  recorded: instant;
  /** 
   * Time when the event was recorded
   */
  _recorded?: Element
  /** 
   * Whether the event succeeded or failed
   */
  outcome?: code;
  /** 
   * Whether the event succeeded or failed
   */
  _outcome?: Element
  /** 
   * Description of the event outcome
   */
  outcomeDesc?: string;
  /** 
   * Description of the event outcome
   */
  _outcomeDesc?: Element
  /** 
   * The purposeOfUse of the event
   */
  purposeOfEvent?: Array<CodeableConcept>;
  /** 
   * Actor involved in the event
   */
  agent: Array<AuditEventAgent>;
  /** 
   * Audit Event Reporter
   */
  source: AuditEventSource;
  /** 
   * Data or objects used
   */
  entity?: Array<AuditEventEntity>;
}

export interface Basic {
resourceType: "Basic"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business identifier
   */
  identifier?: Array<Identifier>;
  /** 
   * Kind of Resource
   */
  code: CodeableConcept;
  /** 
   * Identifies the focus of this resource
   */
  subject?: Reference;
  /** 
   * When created
   */
  created?: date;
  /** 
   * When created
   */
  _created?: Element
  /** 
   * Who created
   */
  author?: Reference;
}

export interface Binary {
resourceType: "Binary"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * MimeType of the binary content
   */
  contentType: code;
  /** 
   * MimeType of the binary content
   */
  _contentType?: Element
  /** 
   * Identifies another resource to use as proxy when enforcing access control
   */
  securityContext?: Reference;
  /** 
   * The actual content
   */
  data?: base64Binary;
  /** 
   * The actual content
   */
  _data?: Element
}

export interface BiologicallyDerivedProductCollection {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Individual performing collection
   */
  collector?: Reference;
  /** 
   * Who is product from
   */
  source?: Reference;
  /** 
   * Time of product collection
   */
  collectedDateTime?: dateTime;
  /** 
   * Time of product collection
   */
  _collectedDateTime?: Element
  /** 
   * Time of product collection
   */
  collectedPeriod?: Period;
}
export interface BiologicallyDerivedProductProcessing {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Description of of processing
   */
  description?: string;
  /** 
   * Description of of processing
   */
  _description?: Element
  /** 
   * Procesing code
   */
  procedure?: CodeableConcept;
  /** 
   * Substance added during processing
   */
  additive?: Reference;
  /** 
   * Time of processing
   */
  timeDateTime?: dateTime;
  /** 
   * Time of processing
   */
  _timeDateTime?: Element
  /** 
   * Time of processing
   */
  timePeriod?: Period;
}
export interface BiologicallyDerivedProductManipulation {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Description of manipulation
   */
  description?: string;
  /** 
   * Description of manipulation
   */
  _description?: Element
  /** 
   * Time of manipulation
   */
  timeDateTime?: dateTime;
  /** 
   * Time of manipulation
   */
  _timeDateTime?: Element
  /** 
   * Time of manipulation
   */
  timePeriod?: Period;
}
export interface BiologicallyDerivedProductStorage {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Description of storage
   */
  description?: string;
  /** 
   * Description of storage
   */
  _description?: Element
  /** 
   * Storage temperature
   */
  temperature?: decimal;
  /** 
   * Storage temperature
   */
  _temperature?: Element
  /** 
   * farenheit | celsius | kelvin
   */
  scale?: code;
  /** 
   * farenheit | celsius | kelvin
   */
  _scale?: Element
  /** 
   * Storage timeperiod
   */
  duration?: Period;
}
export interface BiologicallyDerivedProduct {
resourceType: "BiologicallyDerivedProduct"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * External ids for this item
   */
  identifier?: Array<Identifier>;
  /** 
   * organ | tissue | fluid | cells | biologicalAgent
   */
  productCategory?: code;
  /** 
   * organ | tissue | fluid | cells | biologicalAgent
   */
  _productCategory?: Element
  /** 
   * What this biologically derived product is
   */
  productCode?: CodeableConcept;
  /** 
   * available | unavailable
   */
  status?: code;
  /** 
   * available | unavailable
   */
  _status?: Element
  /** 
   * Procedure request
   */
  request?: Array<Reference>;
  /** 
   * The amount of this biologically derived product
   */
  quantity?: integer;
  /** 
   * The amount of this biologically derived product
   */
  _quantity?: Element
  /** 
   * BiologicallyDerivedProduct parent
   */
  parent?: Array<Reference>;
  /** 
   * How this product was collected
   */
  collection?: BiologicallyDerivedProductCollection;
  /** 
   * Any processing of the product during collection
   */
  processing?: Array<BiologicallyDerivedProductProcessing>;
  /** 
   * Any manipulation of product post-collection
   */
  manipulation?: BiologicallyDerivedProductManipulation;
  /** 
   * Product storage
   */
  storage?: Array<BiologicallyDerivedProductStorage>;
}

export interface BodyStructure {
resourceType: "BodyStructure"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Bodystructure identifier
   */
  identifier?: Array<Identifier>;
  /** 
   * Whether this record is in active use
   */
  active?: boolean;
  /** 
   * Whether this record is in active use
   */
  _active?: Element
  /** 
   * Kind of Structure
   */
  morphology?: CodeableConcept;
  /** 
   * Body site
   */
  location?: CodeableConcept;
  /** 
   * Body site modifier
   */
  locationQualifier?: Array<CodeableConcept>;
  /** 
   * Text description
   */
  description?: string;
  /** 
   * Text description
   */
  _description?: Element
  /** 
   * Attached images
   */
  image?: Array<Attachment>;
  /** 
   * Who this is about
   */
  patient: Reference;
}

export interface BundleLink {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * See http://www.iana.org/assignments/link-relations/link-relations.xhtml#link-relations-1
   */
  relation: string;
  /** 
   * See http://www.iana.org/assignments/link-relations/link-relations.xhtml#link-relations-1
   */
  _relation?: Element
  /** 
   * Reference details for the link
   */
  url: uri;
  /** 
   * Reference details for the link
   */
  _url?: Element
}
export interface BundleEntrySearch {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * match | include | outcome - why this is in the result set
   */
  mode?: code;
  /** 
   * match | include | outcome - why this is in the result set
   */
  _mode?: Element
  /** 
   * Search ranking (between 0 and 1)
   */
  score?: decimal;
  /** 
   * Search ranking (between 0 and 1)
   */
  _score?: Element
}
export interface BundleEntryRequest {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * GET | HEAD | POST | PUT | DELETE | PATCH
   */
  method: code;
  /** 
   * GET | HEAD | POST | PUT | DELETE | PATCH
   */
  _method?: Element
  /** 
   * URL for HTTP equivalent of this entry
   */
  url: uri;
  /** 
   * URL for HTTP equivalent of this entry
   */
  _url?: Element
  /** 
   * For managing cache currency
   */
  ifNoneMatch?: string;
  /** 
   * For managing cache currency
   */
  _ifNoneMatch?: Element
  /** 
   * For managing cache currency
   */
  ifModifiedSince?: instant;
  /** 
   * For managing cache currency
   */
  _ifModifiedSince?: Element
  /** 
   * For managing update contention
   */
  ifMatch?: string;
  /** 
   * For managing update contention
   */
  _ifMatch?: Element
  /** 
   * For conditional creates
   */
  ifNoneExist?: string;
  /** 
   * For conditional creates
   */
  _ifNoneExist?: Element
}
export interface BundleEntryResponse {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Status response code (text optional)
   */
  status: string;
  /** 
   * Status response code (text optional)
   */
  _status?: Element
  /** 
   * The location (if the operation returns a location)
   */
  location?: uri;
  /** 
   * The location (if the operation returns a location)
   */
  _location?: Element
  /** 
   * The Etag for the resource (if relevant)
   */
  etag?: string;
  /** 
   * The Etag for the resource (if relevant)
   */
  _etag?: Element
  /** 
   * Server's date time modified
   */
  lastModified?: instant;
  /** 
   * Server's date time modified
   */
  _lastModified?: Element
  /** 
   * OperationOutcome with hints and warnings (for batch/transaction)
   */
  outcome?: Resource;
}
export interface BundleEntry {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Links related to this entry
   */
  link?: Array<BundleLink>;
  /** 
   * URI for resource (Absolute URL server address or URI for UUID/OID)
   */
  fullUrl?: uri;
  /** 
   * URI for resource (Absolute URL server address or URI for UUID/OID)
   */
  _fullUrl?: Element
  /** 
   * A resource in the bundle
   */
  resource?: Resource;
  /** 
   * Search related information
   */
  search?: BundleEntrySearch;
  /** 
   * Additional execution information (transaction/batch/history)
   */
  request?: BundleEntryRequest;
  /** 
   * Results of execution (transaction/batch/history)
   */
  response?: BundleEntryResponse;
}
export interface Bundle {
resourceType: "Bundle"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Persistent identifier for the bundle
   */
  identifier?: Identifier;
  /** 
   * document | message | transaction | transaction-response | batch | batch-response | history | searchset | collection
   */
  type: code;
  /** 
   * document | message | transaction | transaction-response | batch | batch-response | history | searchset | collection
   */
  _type?: Element
  /** 
   * When the bundle was assembled
   */
  timestamp?: instant;
  /** 
   * When the bundle was assembled
   */
  _timestamp?: Element
  /** 
   * If search, the total number of matches
   */
  total?: unsignedInt;
  /** 
   * If search, the total number of matches
   */
  _total?: Element
  /** 
   * Links related to this Bundle
   */
  link?: Array<BundleLink>;
  /** 
   * Entry in the bundle - will have a resource or information
   */
  entry?: Array<BundleEntry>;
  /** 
   * Digital Signature
   */
  signature?: Signature;
}

export interface CapabilityStatementSoftware {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * A name the software is known by
   */
  name: string;
  /** 
   * A name the software is known by
   */
  _name?: Element
  /** 
   * Version covered by this statement
   */
  version?: string;
  /** 
   * Version covered by this statement
   */
  _version?: Element
  /** 
   * Date this version was released
   */
  releaseDate?: dateTime;
  /** 
   * Date this version was released
   */
  _releaseDate?: Element
}
export interface CapabilityStatementImplementation {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Describes this specific instance
   */
  description: string;
  /** 
   * Describes this specific instance
   */
  _description?: Element
  /** 
   * Base URL for the installation
   */
  url?: url;
  /** 
   * Base URL for the installation
   */
  _url?: Element
  /** 
   * Organization that manages the data
   */
  custodian?: Reference;
}
export interface CapabilityStatementRestSecurity {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Adds CORS Headers (http://enable-cors.org/)
   */
  cors?: boolean;
  /** 
   * Adds CORS Headers (http://enable-cors.org/)
   */
  _cors?: Element
  /** 
   * OAuth | SMART-on-FHIR | NTLM | Basic | Kerberos | Certificates
   */
  service?: Array<CodeableConcept>;
  /** 
   * General description of how security works
   */
  description?: markdown;
  /** 
   * General description of how security works
   */
  _description?: Element
}
export interface CapabilityStatementRestResourceInteraction {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * read | vread | update | patch | delete | history-instance | history-type | create | search-type
   */
  code: code;
  /** 
   * read | vread | update | patch | delete | history-instance | history-type | create | search-type
   */
  _code?: Element
  /** 
   * Anything special about operation behavior
   */
  documentation?: markdown;
  /** 
   * Anything special about operation behavior
   */
  _documentation?: Element
}
export interface CapabilityStatementRestResourceSearchParam {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Name of search parameter
   */
  name: string;
  /** 
   * Name of search parameter
   */
  _name?: Element
  /** 
   * Source of definition for parameter
   */
  definition?: canonical;
  /** 
   * Source of definition for parameter
   */
  _definition?: Element
  /** 
   * number | date | string | token | reference | composite | quantity | uri | special
   */
  type: code;
  /** 
   * number | date | string | token | reference | composite | quantity | uri | special
   */
  _type?: Element
  /** 
   * Server-specific usage
   */
  documentation?: markdown;
  /** 
   * Server-specific usage
   */
  _documentation?: Element
}
export interface CapabilityStatementRestResourceOperation {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Name by which the operation/query is invoked
   */
  name: string;
  /** 
   * Name by which the operation/query is invoked
   */
  _name?: Element
  /** 
   * The defined operation/query
   */
  definition: canonical;
  /** 
   * The defined operation/query
   */
  _definition?: Element
  /** 
   * Specific details about operation behavior
   */
  documentation?: markdown;
  /** 
   * Specific details about operation behavior
   */
  _documentation?: Element
}
export interface CapabilityStatementRestResource {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * A resource type that is supported
   */
  type: code;
  /** 
   * A resource type that is supported
   */
  _type?: Element
  /** 
   * Base System profile for all uses of resource
   */
  profile?: canonical;
  /** 
   * Base System profile for all uses of resource
   */
  _profile?: Element
  /** 
   * Profiles for use cases supported
   */
  supportedProfile?: Array<canonical>;
  /** 
   * Profiles for use cases supported
   */
  _supportedProfile?: Array<Element>
  /** 
   * Additional information about the use of the resource type
   */
  documentation?: markdown;
  /** 
   * Additional information about the use of the resource type
   */
  _documentation?: Element
  /** 
   * What operations are supported?
   */
  interaction?: Array<CapabilityStatementRestResourceInteraction>;
  /** 
   * no-version | versioned | versioned-update
   */
  versioning?: code;
  /** 
   * no-version | versioned | versioned-update
   */
  _versioning?: Element
  /** 
   * Whether vRead can return past versions
   */
  readHistory?: boolean;
  /** 
   * Whether vRead can return past versions
   */
  _readHistory?: Element
  /** 
   * If update can commit to a new identity
   */
  updateCreate?: boolean;
  /** 
   * If update can commit to a new identity
   */
  _updateCreate?: Element
  /** 
   * If allows/uses conditional create
   */
  conditionalCreate?: boolean;
  /** 
   * If allows/uses conditional create
   */
  _conditionalCreate?: Element
  /** 
   * not-supported | modified-since | not-match | full-support
   */
  conditionalRead?: code;
  /** 
   * not-supported | modified-since | not-match | full-support
   */
  _conditionalRead?: Element
  /** 
   * If allows/uses conditional update
   */
  conditionalUpdate?: boolean;
  /** 
   * If allows/uses conditional update
   */
  _conditionalUpdate?: Element
  /** 
   * not-supported | single | multiple - how conditional delete is supported
   */
  conditionalDelete?: code;
  /** 
   * not-supported | single | multiple - how conditional delete is supported
   */
  _conditionalDelete?: Element
  /** 
   * literal | logical | resolves | enforced | local
   */
  referencePolicy?: Array<code>;
  /** 
   * literal | logical | resolves | enforced | local
   */
  _referencePolicy?: Array<Element>
  /** 
   * _include values supported by the server
   */
  searchInclude?: Array<string>;
  /** 
   * _include values supported by the server
   */
  _searchInclude?: Array<Element>
  /** 
   * _revinclude values supported by the server
   */
  searchRevInclude?: Array<string>;
  /** 
   * _revinclude values supported by the server
   */
  _searchRevInclude?: Array<Element>
  /** 
   * Search parameters supported by implementation
   */
  searchParam?: Array<CapabilityStatementRestResourceSearchParam>;
  /** 
   * Definition of a resource operation
   */
  operation?: Array<CapabilityStatementRestResourceOperation>;
}
export interface CapabilityStatementRestInteraction {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * transaction | batch | search-system | history-system
   */
  code: code;
  /** 
   * transaction | batch | search-system | history-system
   */
  _code?: Element
  /** 
   * Anything special about operation behavior
   */
  documentation?: markdown;
  /** 
   * Anything special about operation behavior
   */
  _documentation?: Element
}
export interface CapabilityStatementRest {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * client | server
   */
  mode: code;
  /** 
   * client | server
   */
  _mode?: Element
  /** 
   * General description of implementation
   */
  documentation?: markdown;
  /** 
   * General description of implementation
   */
  _documentation?: Element
  /** 
   * Information about security of implementation
   */
  security?: CapabilityStatementRestSecurity;
  /** 
   * Resource served on the REST interface
   */
  resource?: Array<CapabilityStatementRestResource>;
  /** 
   * What operations are supported?
   */
  interaction?: Array<CapabilityStatementRestInteraction>;
  /** 
   * Search parameters for searching all resources
   */
  searchParam?: Array<CapabilityStatementRestResourceSearchParam>;
  /** 
   * Definition of a system level operation
   */
  operation?: Array<CapabilityStatementRestResourceOperation>;
  /** 
   * Compartments served/used by system
   */
  compartment?: Array<canonical>;
  /** 
   * Compartments served/used by system
   */
  _compartment?: Array<Element>
}
export interface CapabilityStatementMessagingEndpoint {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * http | ftp | mllp +
   */
  protocol: Coding;
  /** 
   * Network address or identifier of the end-point
   */
  address: url;
  /** 
   * Network address or identifier of the end-point
   */
  _address?: Element
}
export interface CapabilityStatementMessagingSupportedMessage {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * sender | receiver
   */
  mode: code;
  /** 
   * sender | receiver
   */
  _mode?: Element
  /** 
   * Message supported by this system
   */
  definition: canonical;
  /** 
   * Message supported by this system
   */
  _definition?: Element
}
export interface CapabilityStatementMessaging {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Where messages should be sent
   */
  endpoint?: Array<CapabilityStatementMessagingEndpoint>;
  /** 
   * Reliable Message Cache Length (min)
   */
  reliableCache?: unsignedInt;
  /** 
   * Reliable Message Cache Length (min)
   */
  _reliableCache?: Element
  /** 
   * Messaging interface behavior details
   */
  documentation?: markdown;
  /** 
   * Messaging interface behavior details
   */
  _documentation?: Element
  /** 
   * Messages supported by this system
   */
  supportedMessage?: Array<CapabilityStatementMessagingSupportedMessage>;
}
export interface CapabilityStatementDocument {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * producer | consumer
   */
  mode: code;
  /** 
   * producer | consumer
   */
  _mode?: Element
  /** 
   * Description of document support
   */
  documentation?: markdown;
  /** 
   * Description of document support
   */
  _documentation?: Element
  /** 
   * Constraint on the resources used in the document
   */
  profile: canonical;
  /** 
   * Constraint on the resources used in the document
   */
  _profile?: Element
}
export interface CapabilityStatement {
resourceType: "CapabilityStatement"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Canonical identifier for this capability statement, represented as a URI (globally unique)
   */
  url?: uri;
  /** 
   * Canonical identifier for this capability statement, represented as a URI (globally unique)
   */
  _url?: Element
  /** 
   * Business version of the capability statement
   */
  version?: string;
  /** 
   * Business version of the capability statement
   */
  _version?: Element
  /** 
   * Name for this capability statement (computer friendly)
   */
  name?: string;
  /** 
   * Name for this capability statement (computer friendly)
   */
  _name?: Element
  /** 
   * Name for this capability statement (human friendly)
   */
  title?: string;
  /** 
   * Name for this capability statement (human friendly)
   */
  _title?: Element
  /** 
   * draft | active | retired | unknown
   */
  status: code;
  /** 
   * draft | active | retired | unknown
   */
  _status?: Element
  /** 
   * For testing purposes, not real usage
   */
  experimental?: boolean;
  /** 
   * For testing purposes, not real usage
   */
  _experimental?: Element
  /** 
   * Date last changed
   */
  date: dateTime;
  /** 
   * Date last changed
   */
  _date?: Element
  /** 
   * Name of the publisher (organization or individual)
   */
  publisher?: string;
  /** 
   * Name of the publisher (organization or individual)
   */
  _publisher?: Element
  /** 
   * Contact details for the publisher
   */
  contact?: Array<ContactDetail>;
  /** 
   * Natural language description of the capability statement
   */
  description?: markdown;
  /** 
   * Natural language description of the capability statement
   */
  _description?: Element
  /** 
   * The context that the content is intended to support
   */
  useContext?: Array<UsageContext>;
  /** 
   * Intended jurisdiction for capability statement (if applicable)
   */
  jurisdiction?: Array<CodeableConcept>;
  /** 
   * Why this capability statement is defined
   */
  purpose?: markdown;
  /** 
   * Why this capability statement is defined
   */
  _purpose?: Element
  /** 
   * Use and/or publishing restrictions
   */
  copyright?: markdown;
  /** 
   * Use and/or publishing restrictions
   */
  _copyright?: Element
  /** 
   * instance | capability | requirements
   */
  kind: code;
  /** 
   * instance | capability | requirements
   */
  _kind?: Element
  /** 
   * Canonical URL of another capability statement this implements
   */
  instantiates?: Array<canonical>;
  /** 
   * Canonical URL of another capability statement this implements
   */
  _instantiates?: Array<Element>
  /** 
   * Canonical URL of another capability statement this adds to
   */
  imports?: Array<canonical>;
  /** 
   * Canonical URL of another capability statement this adds to
   */
  _imports?: Array<Element>
  /** 
   * Software that is covered by this capability statement
   */
  software?: CapabilityStatementSoftware;
  /** 
   * If this describes a specific instance
   */
  implementation?: CapabilityStatementImplementation;
  /** 
   * FHIR Version the system supports
   */
  fhirVersion: code;
  /** 
   * FHIR Version the system supports
   */
  _fhirVersion?: Element
  /** 
   * formats supported (xml | json | ttl | mime type)
   */
  format: Array<code>;
  /** 
   * formats supported (xml | json | ttl | mime type)
   */
  _format?: Array<Element>
  /** 
   * Patch formats supported
   */
  patchFormat?: Array<code>;
  /** 
   * Patch formats supported
   */
  _patchFormat?: Array<Element>
  /** 
   * Implementation guides supported
   */
  implementationGuide?: Array<canonical>;
  /** 
   * Implementation guides supported
   */
  _implementationGuide?: Array<Element>
  /** 
   * If the endpoint is a RESTful one
   */
  rest?: Array<CapabilityStatementRest>;
  /** 
   * If messaging is supported
   */
  messaging?: Array<CapabilityStatementMessaging>;
  /** 
   * Document definition
   */
  document?: Array<CapabilityStatementDocument>;
}

export interface CarePlanActivityDetail {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Appointment | CommunicationRequest | DeviceRequest | MedicationRequest | NutritionOrder | Task | ServiceRequest | VisionPrescription
   */
  kind?: code;
  /** 
   * Appointment | CommunicationRequest | DeviceRequest | MedicationRequest | NutritionOrder | Task | ServiceRequest | VisionPrescription
   */
  _kind?: Element
  /** 
   * Instantiates FHIR protocol or definition
   */
  instantiatesCanonical?: Array<canonical>;
  /** 
   * Instantiates FHIR protocol or definition
   */
  _instantiatesCanonical?: Array<Element>
  /** 
   * Instantiates external protocol or definition
   */
  instantiatesUri?: Array<uri>;
  /** 
   * Instantiates external protocol or definition
   */
  _instantiatesUri?: Array<Element>
  /** 
   * Detail type of activity
   */
  code?: CodeableConcept;
  /** 
   * Why activity should be done or why activity was prohibited
   */
  reasonCode?: Array<CodeableConcept>;
  /** 
   * Why activity is needed
   */
  reasonReference?: Array<Reference>;
  /** 
   * Goals this activity relates to
   */
  goal?: Array<Reference>;
  /** 
   * not-started | scheduled | in-progress | on-hold | completed | cancelled | stopped | unknown | entered-in-error
   */
  status: code;
  /** 
   * not-started | scheduled | in-progress | on-hold | completed | cancelled | stopped | unknown | entered-in-error
   */
  _status?: Element
  /** 
   * Reason for current status
   */
  statusReason?: CodeableConcept;
  /** 
   * If true, activity is prohibiting action
   */
  doNotPerform?: boolean;
  /** 
   * If true, activity is prohibiting action
   */
  _doNotPerform?: Element
  /** 
   * When activity is to occur
   */
  scheduledTiming?: Timing;
  /** 
   * When activity is to occur
   */
  scheduledPeriod?: Period;
  /** 
   * When activity is to occur
   */
  scheduledString?: string;
  /** 
   * When activity is to occur
   */
  _scheduledString?: Element
  /** 
   * Where it should happen
   */
  location?: Reference;
  /** 
   * Who will be responsible?
   */
  performer?: Array<Reference>;
  /** 
   * What is to be administered/supplied
   */
  productCodeableConcept?: CodeableConcept;
  /** 
   * What is to be administered/supplied
   */
  productReference?: Reference;
  /** 
   * How to consume/day?
   */
  dailyAmount?: Quantity;
  /** 
   * How much to administer/supply/consume
   */
  quantity?: Quantity;
  /** 
   * Extra info describing activity to perform
   */
  description?: string;
  /** 
   * Extra info describing activity to perform
   */
  _description?: Element
}
export interface CarePlanActivity {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Results of the activity
   */
  outcomeCodeableConcept?: Array<CodeableConcept>;
  /** 
   * Appointment, Encounter, Procedure, etc.
   */
  outcomeReference?: Array<Reference>;
  /** 
   * Comments about the activity status/progress
   */
  progress?: Array<Annotation>;
  /** 
   * Activity details defined in specific resource
   */
  reference?: Reference;
  /** 
   * In-line definition of activity
   */
  detail?: CarePlanActivityDetail;
}
export interface CarePlan {
resourceType: "CarePlan"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * External Ids for this plan
   */
  identifier?: Array<Identifier>;
  /** 
   * Instantiates FHIR protocol or definition
   */
  instantiatesCanonical?: Array<canonical>;
  /** 
   * Instantiates FHIR protocol or definition
   */
  _instantiatesCanonical?: Array<Element>
  /** 
   * Instantiates external protocol or definition
   */
  instantiatesUri?: Array<uri>;
  /** 
   * Instantiates external protocol or definition
   */
  _instantiatesUri?: Array<Element>
  /** 
   * Fulfills CarePlan
   */
  basedOn?: Array<Reference>;
  /** 
   * CarePlan replaced by this CarePlan
   */
  replaces?: Array<Reference>;
  /** 
   * Part of referenced CarePlan
   */
  partOf?: Array<Reference>;
  /** 
   * draft | active | on-hold | revoked | completed | entered-in-error | unknown
   */
  status: code;
  /** 
   * draft | active | on-hold | revoked | completed | entered-in-error | unknown
   */
  _status?: Element
  /** 
   * proposal | plan | order | option
   */
  intent: code;
  /** 
   * proposal | plan | order | option
   */
  _intent?: Element
  /** 
   * Type of plan
   */
  category?: Array<CodeableConcept>;
  /** 
   * Human-friendly name for the care plan
   */
  title?: string;
  /** 
   * Human-friendly name for the care plan
   */
  _title?: Element
  /** 
   * Summary of nature of plan
   */
  description?: string;
  /** 
   * Summary of nature of plan
   */
  _description?: Element
  /** 
   * Who the care plan is for
   */
  subject: Reference;
  /** 
   * Encounter created as part of
   */
  encounter?: Reference;
  /** 
   * Time period plan covers
   */
  period?: Period;
  /** 
   * Date record was first recorded
   */
  created?: dateTime;
  /** 
   * Date record was first recorded
   */
  _created?: Element
  /** 
   * Who is the designated responsible party
   */
  author?: Reference;
  /** 
   * Who provided the content of the care plan
   */
  contributor?: Array<Reference>;
  /** 
   * Who's involved in plan?
   */
  careTeam?: Array<Reference>;
  /** 
   * Health issues this plan addresses
   */
  addresses?: Array<Reference>;
  /** 
   * Information considered as part of plan
   */
  supportingInfo?: Array<Reference>;
  /** 
   * Desired outcome of plan
   */
  goal?: Array<Reference>;
  /** 
   * Action to occur as part of plan
   */
  activity?: Array<CarePlanActivity>;
  /** 
   * Comments about the plan
   */
  note?: Array<Annotation>;
}

export interface CareTeamParticipant {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Type of involvement
   */
  role?: Array<CodeableConcept>;
  /** 
   * Who is involved
   */
  member?: Reference;
  /** 
   * Organization of the practitioner
   */
  onBehalfOf?: Reference;
  /** 
   * Time period of participant
   */
  period?: Period;
}
export interface CareTeam {
resourceType: "CareTeam"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * External Ids for this team
   */
  identifier?: Array<Identifier>;
  /** 
   * proposed | active | suspended | inactive | entered-in-error
   */
  status?: code;
  /** 
   * proposed | active | suspended | inactive | entered-in-error
   */
  _status?: Element
  /** 
   * Type of team
   */
  category?: Array<CodeableConcept>;
  /** 
   * Name of the team, such as crisis assessment team
   */
  name?: string;
  /** 
   * Name of the team, such as crisis assessment team
   */
  _name?: Element
  /** 
   * Who care team is for
   */
  subject?: Reference;
  /** 
   * Encounter created as part of
   */
  encounter?: Reference;
  /** 
   * Time period team covers
   */
  period?: Period;
  /** 
   * Members of the team
   */
  participant?: Array<CareTeamParticipant>;
  /** 
   * Why the care team exists
   */
  reasonCode?: Array<CodeableConcept>;
  /** 
   * Why the care team exists
   */
  reasonReference?: Array<Reference>;
  /** 
   * Organization responsible for the care team
   */
  managingOrganization?: Array<Reference>;
  /** 
   * A contact detail for the care team (that applies to all members)
   */
  telecom?: Array<ContactPoint>;
  /** 
   * Comments made about the CareTeam
   */
  note?: Array<Annotation>;
}

export interface CatalogEntryRelatedEntry {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * triggers | is-replaced-by
   */
  relationtype: code;
  /** 
   * triggers | is-replaced-by
   */
  _relationtype?: Element
  /** 
   * The reference to the related item
   */
  item: Reference;
}
export interface CatalogEntry {
resourceType: "CatalogEntry"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Unique identifier of the catalog item
   */
  identifier?: Array<Identifier>;
  /** 
   * The type of item - medication, device, service, protocol or other
   */
  type?: CodeableConcept;
  /** 
   * Whether the entry represents an orderable item
   */
  orderable: boolean;
  /** 
   * Whether the entry represents an orderable item
   */
  _orderable?: Element
  /** 
   * The item that is being defined
   */
  referencedItem: Reference;
  /** 
   * Any additional identifier(s) for the catalog item, in the same granularity or concept
   */
  additionalIdentifier?: Array<Identifier>;
  /** 
   * Classification (category or class) of the item entry
   */
  classification?: Array<CodeableConcept>;
  /** 
   * draft | active | retired | unknown
   */
  status?: code;
  /** 
   * draft | active | retired | unknown
   */
  _status?: Element
  /** 
   * The time period in which this catalog entry is expected to be active
   */
  validityPeriod?: Period;
  /** 
   * The date until which this catalog entry is expected to be active
   */
  validTo?: dateTime;
  /** 
   * The date until which this catalog entry is expected to be active
   */
  _validTo?: Element
  /** 
   * When was this catalog last updated
   */
  lastUpdated?: dateTime;
  /** 
   * When was this catalog last updated
   */
  _lastUpdated?: Element
  /** 
   * Additional characteristics of the catalog entry
   */
  additionalCharacteristic?: Array<CodeableConcept>;
  /** 
   * Additional classification of the catalog entry
   */
  additionalClassification?: Array<CodeableConcept>;
  /** 
   * An item that this catalog entry is related to
   */
  relatedEntry?: Array<CatalogEntryRelatedEntry>;
}

export interface ChargeItemPerformer {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * What type of performance was done
   */
  function?: CodeableConcept;
  /** 
   * Individual who was performing
   */
  actor: Reference;
}
export interface ChargeItem {
resourceType: "ChargeItem"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business Identifier for item
   */
  identifier?: Array<Identifier>;
  /** 
   * Defining information about the code of this charge item
   */
  definitionUri?: Array<uri>;
  /** 
   * Defining information about the code of this charge item
   */
  _definitionUri?: Array<Element>
  /** 
   * Resource defining the code of this ChargeItem
   */
  definitionCanonical?: Array<canonical>;
  /** 
   * Resource defining the code of this ChargeItem
   */
  _definitionCanonical?: Array<Element>
  /** 
   * planned | billable | not-billable | aborted | billed | entered-in-error | unknown
   */
  status: code;
  /** 
   * planned | billable | not-billable | aborted | billed | entered-in-error | unknown
   */
  _status?: Element
  /** 
   * Part of referenced ChargeItem
   */
  partOf?: Array<Reference>;
  /** 
   * A code that identifies the charge, like a billing code
   */
  code: CodeableConcept;
  /** 
   * Individual service was done for/to
   */
  subject: Reference;
  /** 
   * Encounter / Episode associated with event
   */
  context?: Reference;
  /** 
   * When the charged service was applied
   */
  occurrenceDateTime?: dateTime;
  /** 
   * When the charged service was applied
   */
  _occurrenceDateTime?: Element
  /** 
   * When the charged service was applied
   */
  occurrencePeriod?: Period;
  /** 
   * When the charged service was applied
   */
  occurrenceTiming?: Timing;
  /** 
   * Who performed charged service
   */
  performer?: Array<ChargeItemPerformer>;
  /** 
   * Organization providing the charged service
   */
  performingOrganization?: Reference;
  /** 
   * Organization requesting the charged service
   */
  requestingOrganization?: Reference;
  /** 
   * Organization that has ownership of the (potential, future) revenue
   */
  costCenter?: Reference;
  /** 
   * Quantity of which the charge item has been serviced
   */
  quantity?: Quantity;
  /** 
   * Anatomical location, if relevant
   */
  bodysite?: Array<CodeableConcept>;
  /** 
   * Factor overriding the associated rules
   */
  factorOverride?: decimal;
  /** 
   * Factor overriding the associated rules
   */
  _factorOverride?: Element
  /** 
   * Price overriding the associated rules
   */
  priceOverride?: Money;
  /** 
   * Reason for overriding the list price/factor
   */
  overrideReason?: string;
  /** 
   * Reason for overriding the list price/factor
   */
  _overrideReason?: Element
  /** 
   * Individual who was entering
   */
  enterer?: Reference;
  /** 
   * Date the charge item was entered
   */
  enteredDate?: dateTime;
  /** 
   * Date the charge item was entered
   */
  _enteredDate?: Element
  /** 
   * Why was the charged  service rendered?
   */
  reason?: Array<CodeableConcept>;
  /** 
   * Which rendered service is being charged?
   */
  service?: Array<Reference>;
  /** 
   * Product charged
   */
  productReference?: Reference;
  /** 
   * Product charged
   */
  productCodeableConcept?: CodeableConcept;
  /** 
   * Account to place this charge
   */
  account?: Array<Reference>;
  /** 
   * Comments made about the ChargeItem
   */
  note?: Array<Annotation>;
  /** 
   * Further information supporting this charge
   */
  supportingInformation?: Array<Reference>;
}

export interface ChargeItemDefinitionApplicability {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Natural language description of the condition
   */
  description?: string;
  /** 
   * Natural language description of the condition
   */
  _description?: Element
  /** 
   * Language of the expression
   */
  language?: string;
  /** 
   * Language of the expression
   */
  _language?: Element
  /** 
   * Boolean-valued expression
   */
  expression?: string;
  /** 
   * Boolean-valued expression
   */
  _expression?: Element
}
export interface ChargeItemDefinitionPropertyGroupPriceComponent {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * base | surcharge | deduction | discount | tax | informational
   */
  type: code;
  /** 
   * base | surcharge | deduction | discount | tax | informational
   */
  _type?: Element
  /** 
   * Code identifying the specific component
   */
  code?: CodeableConcept;
  /** 
   * Factor used for calculating this component
   */
  factor?: decimal;
  /** 
   * Factor used for calculating this component
   */
  _factor?: Element
  /** 
   * Monetary amount associated with this component
   */
  amount?: Money;
}
export interface ChargeItemDefinitionPropertyGroup {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Conditions under which the priceComponent is applicable
   */
  applicability?: Array<ChargeItemDefinitionApplicability>;
  /** 
   * Components of total line item price
   */
  priceComponent?: Array<ChargeItemDefinitionPropertyGroupPriceComponent>;
}
export interface ChargeItemDefinition {
resourceType: "ChargeItemDefinition"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Canonical identifier for this charge item definition, represented as a URI (globally unique)
   */
  url: uri;
  /** 
   * Canonical identifier for this charge item definition, represented as a URI (globally unique)
   */
  _url?: Element
  /** 
   * Additional identifier for the charge item definition
   */
  identifier?: Array<Identifier>;
  /** 
   * Business version of the charge item definition
   */
  version?: string;
  /** 
   * Business version of the charge item definition
   */
  _version?: Element
  /** 
   * Name for this charge item definition (human friendly)
   */
  title?: string;
  /** 
   * Name for this charge item definition (human friendly)
   */
  _title?: Element
  /** 
   * Underlying externally-defined charge item definition
   */
  derivedFromUri?: Array<uri>;
  /** 
   * Underlying externally-defined charge item definition
   */
  _derivedFromUri?: Array<Element>
  /** 
   * A larger definition of which this particular definition is a component or step
   */
  partOf?: Array<canonical>;
  /** 
   * A larger definition of which this particular definition is a component or step
   */
  _partOf?: Array<Element>
  /** 
   * Completed or terminated request(s) whose function is taken by this new request
   */
  replaces?: Array<canonical>;
  /** 
   * Completed or terminated request(s) whose function is taken by this new request
   */
  _replaces?: Array<Element>
  /** 
   * draft | active | retired | unknown
   */
  status: code;
  /** 
   * draft | active | retired | unknown
   */
  _status?: Element
  /** 
   * For testing purposes, not real usage
   */
  experimental?: boolean;
  /** 
   * For testing purposes, not real usage
   */
  _experimental?: Element
  /** 
   * Date last changed
   */
  date?: dateTime;
  /** 
   * Date last changed
   */
  _date?: Element
  /** 
   * Name of the publisher (organization or individual)
   */
  publisher?: string;
  /** 
   * Name of the publisher (organization or individual)
   */
  _publisher?: Element
  /** 
   * Contact details for the publisher
   */
  contact?: Array<ContactDetail>;
  /** 
   * Natural language description of the charge item definition
   */
  description?: markdown;
  /** 
   * Natural language description of the charge item definition
   */
  _description?: Element
  /** 
   * The context that the content is intended to support
   */
  useContext?: Array<UsageContext>;
  /** 
   * Intended jurisdiction for charge item definition (if applicable)
   */
  jurisdiction?: Array<CodeableConcept>;
  /** 
   * Use and/or publishing restrictions
   */
  copyright?: markdown;
  /** 
   * Use and/or publishing restrictions
   */
  _copyright?: Element
  /** 
   * When the charge item definition was approved by publisher
   */
  approvalDate?: date;
  /** 
   * When the charge item definition was approved by publisher
   */
  _approvalDate?: Element
  /** 
   * When the charge item definition was last reviewed
   */
  lastReviewDate?: date;
  /** 
   * When the charge item definition was last reviewed
   */
  _lastReviewDate?: Element
  /** 
   * When the charge item definition is expected to be used
   */
  effectivePeriod?: Period;
  /** 
   * Billing codes or product types this definition applies to
   */
  code?: CodeableConcept;
  /** 
   * Instances this definition applies to
   */
  instance?: Array<Reference>;
  /** 
   * Whether or not the billing code is applicable
   */
  applicability?: Array<ChargeItemDefinitionApplicability>;
  /** 
   * Group of properties which are applicable under the same conditions
   */
  propertyGroup?: Array<ChargeItemDefinitionPropertyGroup>;
}

export interface CitationSummary {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Format for display of the citation
   */
  style?: CodeableConcept;
  /** 
   * The human-readable display of the citation
   */
  text: markdown;
  /** 
   * The human-readable display of the citation
   */
  _text?: Element
}
export interface CitationClassification {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The kind of classifier (e.g. publication type, keyword)
   */
  type?: CodeableConcept;
  /** 
   * The specific classification value
   */
  classifier?: Array<CodeableConcept>;
}
export interface CitationStatusDate {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Classification of the status
   */
  activity: CodeableConcept;
  /** 
   * Either occurred or expected
   */
  actual?: boolean;
  /** 
   * Either occurred or expected
   */
  _actual?: Element
  /** 
   * When the status started and/or ended
   */
  period: Period;
}
export interface CitationRelatesTo {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * How the Citation resource relates to the target artifact
   */
  relationshipType: CodeableConcept;
  /** 
   * The clasification of the related artifact
   */
  targetClassifier?: Array<CodeableConcept>;
  /** 
   * The article or artifact that the Citation Resource is related to
   */
  targetUri?: uri;
  /** 
   * The article or artifact that the Citation Resource is related to
   */
  _targetUri?: Element
  /** 
   * The article or artifact that the Citation Resource is related to
   */
  targetIdentifier?: Identifier;
  /** 
   * The article or artifact that the Citation Resource is related to
   */
  targetReference?: Reference;
  /** 
   * The article or artifact that the Citation Resource is related to
   */
  targetAttachment?: Attachment;
}
export interface CitationCitedArtifactVersion {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The version number or other version identifier
   */
  value: string;
  /** 
   * The version number or other version identifier
   */
  _value?: Element
  /** 
   * Citation for the main version of the cited artifact
   */
  baseCitation?: Reference;
}
export interface CitationCitedArtifactStatusDate {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Classification of the status
   */
  activity: CodeableConcept;
  /** 
   * Either occurred or expected
   */
  actual?: boolean;
  /** 
   * Either occurred or expected
   */
  _actual?: Element
  /** 
   * When the status started and/or ended
   */
  period: Period;
}
export interface CitationCitedArtifactTitle {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The kind of title
   */
  type?: Array<CodeableConcept>;
  /** 
   * Used to express the specific language
   */
  language?: CodeableConcept;
  /** 
   * The title of the article or artifact
   */
  text: markdown;
  /** 
   * The title of the article or artifact
   */
  _text?: Element
}
export interface CitationCitedArtifactAbstract {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The kind of abstract
   */
  type?: CodeableConcept;
  /** 
   * Used to express the specific language
   */
  language?: CodeableConcept;
  /** 
   * Abstract content
   */
  text: markdown;
  /** 
   * Abstract content
   */
  _text?: Element
  /** 
   * Copyright notice for the abstract
   */
  copyright?: markdown;
  /** 
   * Copyright notice for the abstract
   */
  _copyright?: Element
}
export interface CitationCitedArtifactPart {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The kind of component
   */
  type?: CodeableConcept;
  /** 
   * The specification of the component
   */
  value?: string;
  /** 
   * The specification of the component
   */
  _value?: Element
  /** 
   * The citation for the full article or artifact
   */
  baseCitation?: Reference;
}
export interface CitationCitedArtifactRelatesTo {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * How the cited artifact relates to the target artifact
   */
  relationshipType: CodeableConcept;
  /** 
   * The clasification of the related artifact
   */
  targetClassifier?: Array<CodeableConcept>;
  /** 
   * The article or artifact that the cited artifact is related to
   */
  targetUri?: uri;
  /** 
   * The article or artifact that the cited artifact is related to
   */
  _targetUri?: Element
  /** 
   * The article or artifact that the cited artifact is related to
   */
  targetIdentifier?: Identifier;
  /** 
   * The article or artifact that the cited artifact is related to
   */
  targetReference?: Reference;
  /** 
   * The article or artifact that the cited artifact is related to
   */
  targetAttachment?: Attachment;
}
export interface CitationCitedArtifactPublicationFormPublishedIn {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Kind of container (e.g. Periodical, database, or book)
   */
  type?: CodeableConcept;
  /** 
   * Journal identifiers include ISSN, ISO Abbreviation and NLMuniqueID; Book identifiers include ISBN
   */
  identifier?: Array<Identifier>;
  /** 
   * Name of the database or title of the book or journal
   */
  title?: string;
  /** 
   * Name of the database or title of the book or journal
   */
  _title?: Element
  /** 
   * Name of the publisher
   */
  publisher?: Reference;
  /** 
   * Geographic location of the publisher
   */
  publisherLocation?: string;
  /** 
   * Geographic location of the publisher
   */
  _publisherLocation?: Element
}
export interface CitationCitedArtifactPublicationFormPeriodicReleaseDateOfPublication {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Date on which the issue of the journal was published
   */
  date?: date;
  /** 
   * Date on which the issue of the journal was published
   */
  _date?: Element
  /** 
   * Year on which the issue of the journal was published
   */
  year?: string;
  /** 
   * Year on which the issue of the journal was published
   */
  _year?: Element
  /** 
   * Month on which the issue of the journal was published
   */
  month?: string;
  /** 
   * Month on which the issue of the journal was published
   */
  _month?: Element
  /** 
   * Day on which the issue of the journal was published
   */
  day?: string;
  /** 
   * Day on which the issue of the journal was published
   */
  _day?: Element
  /** 
   * Season on which the issue of the journal was published
   */
  season?: string;
  /** 
   * Season on which the issue of the journal was published
   */
  _season?: Element
  /** 
   * Text representation of the date of which the issue of the journal was published
   */
  text?: string;
  /** 
   * Text representation of the date of which the issue of the journal was published
   */
  _text?: Element
}
export interface CitationCitedArtifactPublicationFormPeriodicRelease {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Internet or Print
   */
  citedMedium?: CodeableConcept;
  /** 
   * Volume number of journal in which the article is published
   */
  volume?: string;
  /** 
   * Volume number of journal in which the article is published
   */
  _volume?: Element
  /** 
   * Issue, part or supplement of journal in which the article is published
   */
  issue?: string;
  /** 
   * Issue, part or supplement of journal in which the article is published
   */
  _issue?: Element
  /** 
   * Defining the date on which the issue of the journal was published
   */
  dateOfPublication?: CitationCitedArtifactPublicationFormPeriodicReleaseDateOfPublication;
}
export interface CitationCitedArtifactPublicationForm {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The collection the cited article or artifact is published in
   */
  publishedIn?: CitationCitedArtifactPublicationFormPublishedIn;
  /** 
   * The specific issue in which the cited article resides
   */
  periodicRelease?: CitationCitedArtifactPublicationFormPeriodicRelease;
  /** 
   * The date the article was added to the database, or the date the article was released
   */
  articleDate?: dateTime;
  /** 
   * The date the article was added to the database, or the date the article was released
   */
  _articleDate?: Element
  /** 
   * The date the article was last revised or updated in the database
   */
  lastRevisionDate?: dateTime;
  /** 
   * The date the article was last revised or updated in the database
   */
  _lastRevisionDate?: Element
  /** 
   * Language in which this form of the article is published
   */
  language?: Array<CodeableConcept>;
  /** 
   * Entry number or identifier for inclusion in a database
   */
  accessionNumber?: string;
  /** 
   * Entry number or identifier for inclusion in a database
   */
  _accessionNumber?: Element
  /** 
   * Used for full display of pagination
   */
  pageString?: string;
  /** 
   * Used for full display of pagination
   */
  _pageString?: Element
  /** 
   * Used for isolated representation of first page
   */
  firstPage?: string;
  /** 
   * Used for isolated representation of first page
   */
  _firstPage?: Element
  /** 
   * Used for isolated representation of last page
   */
  lastPage?: string;
  /** 
   * Used for isolated representation of last page
   */
  _lastPage?: Element
  /** 
   * Number of pages or screens
   */
  pageCount?: string;
  /** 
   * Number of pages or screens
   */
  _pageCount?: Element
  /** 
   * Copyright notice for the full article or artifact
   */
  copyright?: markdown;
  /** 
   * Copyright notice for the full article or artifact
   */
  _copyright?: Element
}
export interface CitationCitedArtifactWebLocation {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Code the reason for different URLs, e.g. abstract and full-text
   */
  type?: CodeableConcept;
  /** 
   * The specific URL
   */
  url?: uri;
  /** 
   * The specific URL
   */
  _url?: Element
}
export interface CitationCitedArtifactClassificationWhoClassified {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Person who created the classification
   */
  person?: Reference;
  /** 
   * Organization who created the classification
   */
  organization?: Reference;
  /** 
   * The publisher of the classification, not the publisher of the article or artifact being cited
   */
  publisher?: Reference;
  /** 
   * Rights management statement for the classification
   */
  classifierCopyright?: string;
  /** 
   * Rights management statement for the classification
   */
  _classifierCopyright?: Element
  /** 
   * Acceptable to re-use the classification
   */
  freeToShare?: boolean;
  /** 
   * Acceptable to re-use the classification
   */
  _freeToShare?: Element
}
export interface CitationCitedArtifactClassification {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The kind of classifier (e.g. publication type, keyword)
   */
  type?: CodeableConcept;
  /** 
   * The specific classification value
   */
  classifier?: Array<CodeableConcept>;
  /** 
   * Provenance and copyright of classification
   */
  whoClassified?: CitationCitedArtifactClassificationWhoClassified;
}
export interface CitationCitedArtifactContributorshipEntryAffiliationInfo {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Display for the organization
   */
  affiliation?: string;
  /** 
   * Display for the organization
   */
  _affiliation?: Element
  /** 
   * Role within the organization, such as professional title
   */
  role?: string;
  /** 
   * Role within the organization, such as professional title
   */
  _role?: Element
  /** 
   * Identifier for the organization
   */
  identifier?: Array<Identifier>;
}
export interface CitationCitedArtifactContributorshipEntryContributionInstance {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The specific contribution
   */
  type: CodeableConcept;
  /** 
   * The time that the contribution was made
   */
  time?: dateTime;
  /** 
   * The time that the contribution was made
   */
  _time?: Element
}
export interface CitationCitedArtifactContributorshipEntry {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * A name associated with the person
   */
  name?: HumanName;
  /** 
   * Initials for forename
   */
  initials?: string;
  /** 
   * Initials for forename
   */
  _initials?: Element
  /** 
   * Used for collective or corporate name as an author
   */
  collectiveName?: string;
  /** 
   * Used for collective or corporate name as an author
   */
  _collectiveName?: Element
  /** 
   * Author identifier, eg ORCID
   */
  identifier?: Array<Identifier>;
  /** 
   * Organizational affiliation
   */
  affiliationInfo?: Array<CitationCitedArtifactContributorshipEntryAffiliationInfo>;
  /** 
   * Physical mailing address
   */
  address?: Array<Address>;
  /** 
   * Email or telephone contact methods for the author or contributor
   */
  telecom?: Array<ContactPoint>;
  /** 
   * The specific contribution
   */
  contributionType?: Array<CodeableConcept>;
  /** 
   * The role of the contributor (e.g. author, editor, reviewer)
   */
  role?: CodeableConcept;
  /** 
   * Contributions with accounting for time or number
   */
  contributionInstance?: Array<CitationCitedArtifactContributorshipEntryContributionInstance>;
  /** 
   * Indication of which contributor is the corresponding contributor for the role
   */
  correspondingContact?: boolean;
  /** 
   * Indication of which contributor is the corresponding contributor for the role
   */
  _correspondingContact?: Element
  /** 
   * Used to code order of authors
   */
  listOrder?: positiveInt;
  /** 
   * Used to code order of authors
   */
  _listOrder?: Element
}
export interface CitationCitedArtifactContributorshipSummary {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Either authorList or contributorshipStatement
   */
  type?: CodeableConcept;
  /** 
   * The format for the display string
   */
  style?: CodeableConcept;
  /** 
   * Used to code the producer or rule for creating the display string
   */
  source?: CodeableConcept;
  /** 
   * The display string for the author list, contributor list, or contributorship statement
   */
  value: markdown;
  /** 
   * The display string for the author list, contributor list, or contributorship statement
   */
  _value?: Element
}
export interface CitationCitedArtifactContributorship {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Indicates if the list includes all authors and/or contributors
   */
  complete?: boolean;
  /** 
   * Indicates if the list includes all authors and/or contributors
   */
  _complete?: Element
  /** 
   * An individual entity named in the list
   */
  entry?: Array<CitationCitedArtifactContributorshipEntry>;
  /** 
   * Used to record a display of the author/contributor list without separate coding for each list member
   */
  summary?: Array<CitationCitedArtifactContributorshipSummary>;
}
export interface CitationCitedArtifact {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * May include DOI, PMID, PMCID, etc.
   */
  identifier?: Array<Identifier>;
  /** 
   * May include trial registry identifiers
   */
  relatedIdentifier?: Array<Identifier>;
  /** 
   * When the cited artifact was accessed
   */
  dateAccessed?: dateTime;
  /** 
   * When the cited artifact was accessed
   */
  _dateAccessed?: Element
  /** 
   * The defined version of the cited artifact
   */
  version?: CitationCitedArtifactVersion;
  /** 
   * The status of the cited artifact
   */
  currentState?: Array<CodeableConcept>;
  /** 
   * An effective date or period for a status of the cited artifact
   */
  statusDate?: Array<CitationCitedArtifactStatusDate>;
  /** 
   * The title details of the article or artifact
   */
  title?: Array<CitationCitedArtifactTitle>;
  /** 
   * Summary of the article or artifact
   */
  abstract?: Array<CitationCitedArtifactAbstract>;
  /** 
   * The component of the article or artifact
   */
  part?: CitationCitedArtifactPart;
  /** 
   * The artifact related to the cited artifact
   */
  relatesTo?: Array<CitationCitedArtifactRelatesTo>;
  /** 
   * If multiple, used to represent alternative forms of the article that are not separate citations
   */
  publicationForm?: Array<CitationCitedArtifactPublicationForm>;
  /** 
   * Used for any URL for the article or artifact cited
   */
  webLocation?: Array<CitationCitedArtifactWebLocation>;
  /** 
   * The assignment to an organizing scheme
   */
  classification?: Array<CitationCitedArtifactClassification>;
  /** 
   * Attribution of authors and other contributors
   */
  contributorship?: CitationCitedArtifactContributorship;
  /** 
   * Any additional information or content for the article or artifact
   */
  note?: Array<Annotation>;
}
export interface Citation {
resourceType: "Citation"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Canonical identifier for this citation, represented as a globally unique URI
   */
  url?: uri;
  /** 
   * Canonical identifier for this citation, represented as a globally unique URI
   */
  _url?: Element
  /** 
   * Identifier for the Citation resource itself
   */
  identifier?: Array<Identifier>;
  /** 
   * Business version of the citation
   */
  version?: string;
  /** 
   * Business version of the citation
   */
  _version?: Element
  /** 
   * Name for this citation (computer friendly)
   */
  name?: string;
  /** 
   * Name for this citation (computer friendly)
   */
  _name?: Element
  /** 
   * Name for this citation (human friendly)
   */
  title?: string;
  /** 
   * Name for this citation (human friendly)
   */
  _title?: Element
  /** 
   * draft | active | retired | unknown
   */
  status: code;
  /** 
   * draft | active | retired | unknown
   */
  _status?: Element
  /** 
   * For testing purposes, not real usage
   */
  experimental?: boolean;
  /** 
   * For testing purposes, not real usage
   */
  _experimental?: Element
  /** 
   * Date last changed
   */
  date?: dateTime;
  /** 
   * Date last changed
   */
  _date?: Element
  /** 
   * The publisher of the Citation, not the publisher of the article or artifact being cited
   */
  publisher?: string;
  /** 
   * The publisher of the Citation, not the publisher of the article or artifact being cited
   */
  _publisher?: Element
  /** 
   * Contact details for the publisher of the Citation Resource
   */
  contact?: Array<ContactDetail>;
  /** 
   * Natural language description of the citation
   */
  description?: markdown;
  /** 
   * Natural language description of the citation
   */
  _description?: Element
  /** 
   * The context that the Citation Resource content is intended to support
   */
  useContext?: Array<UsageContext>;
  /** 
   * Intended jurisdiction for citation (if applicable)
   */
  jurisdiction?: Array<CodeableConcept>;
  /** 
   * Why this citation is defined
   */
  purpose?: markdown;
  /** 
   * Why this citation is defined
   */
  _purpose?: Element
  /** 
   * Use and/or publishing restrictions for the Citation, not for the cited artifact
   */
  copyright?: markdown;
  /** 
   * Use and/or publishing restrictions for the Citation, not for the cited artifact
   */
  _copyright?: Element
  /** 
   * When the citation was approved by publisher
   */
  approvalDate?: date;
  /** 
   * When the citation was approved by publisher
   */
  _approvalDate?: Element
  /** 
   * When the citation was last reviewed
   */
  lastReviewDate?: date;
  /** 
   * When the citation was last reviewed
   */
  _lastReviewDate?: Element
  /** 
   * When the citation is expected to be used
   */
  effectivePeriod?: Period;
  /** 
   * Who authored the Citation
   */
  author?: Array<ContactDetail>;
  /** 
   * Who edited the Citation
   */
  editor?: Array<ContactDetail>;
  /** 
   * Who reviewed the Citation
   */
  reviewer?: Array<ContactDetail>;
  /** 
   * Who endorsed the Citation
   */
  endorser?: Array<ContactDetail>;
  /** 
   * A human-readable display of the citation
   */
  summary?: Array<CitationSummary>;
  /** 
   * The assignment to an organizing scheme
   */
  classification?: Array<CitationClassification>;
  /** 
   * Used for general notes and annotations not coded elsewhere
   */
  note?: Array<Annotation>;
  /** 
   * The status of the citation
   */
  currentState?: Array<CodeableConcept>;
  /** 
   * An effective date or period for a status of the citation
   */
  statusDate?: Array<CitationStatusDate>;
  /** 
   * Artifact related to the Citation Resource
   */
  relatesTo?: Array<CitationRelatesTo>;
  /** 
   * The article or artifact being described
   */
  citedArtifact?: CitationCitedArtifact;
}

export interface ClaimRelated {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Reference to the related claim
   */
  claim?: Reference;
  /** 
   * How the reference claim is related
   */
  relationship?: CodeableConcept;
  /** 
   * File or case reference
   */
  reference?: Identifier;
}
export interface ClaimPayee {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Category of recipient
   */
  type: CodeableConcept;
  /** 
   * Recipient reference
   */
  party?: Reference;
}
export interface ClaimCareTeam {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Order of care team
   */
  sequence: positiveInt;
  /** 
   * Order of care team
   */
  _sequence?: Element
  /** 
   * Practitioner or organization
   */
  provider: Reference;
  /** 
   * Indicator of the lead practitioner
   */
  responsible?: boolean;
  /** 
   * Indicator of the lead practitioner
   */
  _responsible?: Element
  /** 
   * Function within the team
   */
  role?: CodeableConcept;
  /** 
   * Practitioner credential or specialization
   */
  qualification?: CodeableConcept;
}
export interface ClaimSupportingInfo {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Information instance identifier
   */
  sequence: positiveInt;
  /** 
   * Information instance identifier
   */
  _sequence?: Element
  /** 
   * Classification of the supplied information
   */
  category: CodeableConcept;
  /** 
   * Type of information
   */
  code?: CodeableConcept;
  /** 
   * When it occurred
   */
  timingDate?: date;
  /** 
   * When it occurred
   */
  _timingDate?: Element
  /** 
   * When it occurred
   */
  timingPeriod?: Period;
  /** 
   * Data to be provided
   */
  valueBoolean?: boolean;
  /** 
   * Data to be provided
   */
  _valueBoolean?: Element
  /** 
   * Data to be provided
   */
  valueString?: string;
  /** 
   * Data to be provided
   */
  _valueString?: Element
  /** 
   * Data to be provided
   */
  valueQuantity?: Quantity;
  /** 
   * Data to be provided
   */
  valueAttachment?: Attachment;
  /** 
   * Data to be provided
   */
  valueReference?: Reference;
  /** 
   * Explanation for the information
   */
  reason?: CodeableConcept;
}
export interface ClaimDiagnosis {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Diagnosis instance identifier
   */
  sequence: positiveInt;
  /** 
   * Diagnosis instance identifier
   */
  _sequence?: Element
  /** 
   * Nature of illness or problem
   */
  diagnosisCodeableConcept?: CodeableConcept;
  /** 
   * Nature of illness or problem
   */
  diagnosisReference?: Reference;
  /** 
   * Timing or nature of the diagnosis
   */
  type?: Array<CodeableConcept>;
  /** 
   * Present on admission
   */
  onAdmission?: CodeableConcept;
  /** 
   * Package billing code
   */
  packageCode?: CodeableConcept;
}
export interface ClaimProcedure {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Procedure instance identifier
   */
  sequence: positiveInt;
  /** 
   * Procedure instance identifier
   */
  _sequence?: Element
  /** 
   * Category of Procedure
   */
  type?: Array<CodeableConcept>;
  /** 
   * When the procedure was performed
   */
  date?: dateTime;
  /** 
   * When the procedure was performed
   */
  _date?: Element
  /** 
   * Specific clinical procedure
   */
  procedureCodeableConcept?: CodeableConcept;
  /** 
   * Specific clinical procedure
   */
  procedureReference?: Reference;
  /** 
   * Unique device identifier
   */
  udi?: Array<Reference>;
}
export interface ClaimInsurance {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Insurance instance identifier
   */
  sequence: positiveInt;
  /** 
   * Insurance instance identifier
   */
  _sequence?: Element
  /** 
   * Coverage to be used for adjudication
   */
  focal: boolean;
  /** 
   * Coverage to be used for adjudication
   */
  _focal?: Element
  /** 
   * Pre-assigned Claim number
   */
  identifier?: Identifier;
  /** 
   * Insurance information
   */
  coverage: Reference;
  /** 
   * Additional provider contract number
   */
  businessArrangement?: string;
  /** 
   * Additional provider contract number
   */
  _businessArrangement?: Element
  /** 
   * Prior authorization reference number
   */
  preAuthRef?: Array<string>;
  /** 
   * Prior authorization reference number
   */
  _preAuthRef?: Array<Element>
  /** 
   * Adjudication results
   */
  claimResponse?: Reference;
}
export interface ClaimAccident {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * When the incident occurred
   */
  date: date;
  /** 
   * When the incident occurred
   */
  _date?: Element
  /** 
   * The nature of the accident
   */
  type?: CodeableConcept;
  /** 
   * Where the event occurred
   */
  locationAddress?: Address;
  /** 
   * Where the event occurred
   */
  locationReference?: Reference;
}
export interface ClaimItemDetailSubDetail {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Item instance identifier
   */
  sequence: positiveInt;
  /** 
   * Item instance identifier
   */
  _sequence?: Element
  /** 
   * Revenue or cost center code
   */
  revenue?: CodeableConcept;
  /** 
   * Benefit classification
   */
  category?: CodeableConcept;
  /** 
   * Billing, service, product, or drug code
   */
  productOrService: CodeableConcept;
  /** 
   * Service/Product billing modifiers
   */
  modifier?: Array<CodeableConcept>;
  /** 
   * Program the product or service is provided under
   */
  programCode?: Array<CodeableConcept>;
  /** 
   * Count of products or services
   */
  quantity?: Quantity;
  /** 
   * Fee, charge or cost per item
   */
  unitPrice?: Money;
  /** 
   * Price scaling factor
   */
  factor?: decimal;
  /** 
   * Price scaling factor
   */
  _factor?: Element
  /** 
   * Total item cost
   */
  net?: Money;
  /** 
   * Unique device identifier
   */
  udi?: Array<Reference>;
}
export interface ClaimItemDetail {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Item instance identifier
   */
  sequence: positiveInt;
  /** 
   * Item instance identifier
   */
  _sequence?: Element
  /** 
   * Revenue or cost center code
   */
  revenue?: CodeableConcept;
  /** 
   * Benefit classification
   */
  category?: CodeableConcept;
  /** 
   * Billing, service, product, or drug code
   */
  productOrService: CodeableConcept;
  /** 
   * Service/Product billing modifiers
   */
  modifier?: Array<CodeableConcept>;
  /** 
   * Program the product or service is provided under
   */
  programCode?: Array<CodeableConcept>;
  /** 
   * Count of products or services
   */
  quantity?: Quantity;
  /** 
   * Fee, charge or cost per item
   */
  unitPrice?: Money;
  /** 
   * Price scaling factor
   */
  factor?: decimal;
  /** 
   * Price scaling factor
   */
  _factor?: Element
  /** 
   * Total item cost
   */
  net?: Money;
  /** 
   * Unique device identifier
   */
  udi?: Array<Reference>;
  /** 
   * Product or service provided
   */
  subDetail?: Array<ClaimItemDetailSubDetail>;
}
export interface ClaimItem {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Item instance identifier
   */
  sequence: positiveInt;
  /** 
   * Item instance identifier
   */
  _sequence?: Element
  /** 
   * Applicable careTeam members
   */
  careTeamSequence?: Array<positiveInt>;
  /** 
   * Applicable careTeam members
   */
  _careTeamSequence?: Array<Element>
  /** 
   * Applicable diagnoses
   */
  diagnosisSequence?: Array<positiveInt>;
  /** 
   * Applicable diagnoses
   */
  _diagnosisSequence?: Array<Element>
  /** 
   * Applicable procedures
   */
  procedureSequence?: Array<positiveInt>;
  /** 
   * Applicable procedures
   */
  _procedureSequence?: Array<Element>
  /** 
   * Applicable exception and supporting information
   */
  informationSequence?: Array<positiveInt>;
  /** 
   * Applicable exception and supporting information
   */
  _informationSequence?: Array<Element>
  /** 
   * Revenue or cost center code
   */
  revenue?: CodeableConcept;
  /** 
   * Benefit classification
   */
  category?: CodeableConcept;
  /** 
   * Billing, service, product, or drug code
   */
  productOrService: CodeableConcept;
  /** 
   * Product or service billing modifiers
   */
  modifier?: Array<CodeableConcept>;
  /** 
   * Program the product or service is provided under
   */
  programCode?: Array<CodeableConcept>;
  /** 
   * Date or dates of service or product delivery
   */
  servicedDate?: date;
  /** 
   * Date or dates of service or product delivery
   */
  _servicedDate?: Element
  /** 
   * Date or dates of service or product delivery
   */
  servicedPeriod?: Period;
  /** 
   * Place of service or where product was supplied
   */
  locationCodeableConcept?: CodeableConcept;
  /** 
   * Place of service or where product was supplied
   */
  locationAddress?: Address;
  /** 
   * Place of service or where product was supplied
   */
  locationReference?: Reference;
  /** 
   * Count of products or services
   */
  quantity?: Quantity;
  /** 
   * Fee, charge or cost per item
   */
  unitPrice?: Money;
  /** 
   * Price scaling factor
   */
  factor?: decimal;
  /** 
   * Price scaling factor
   */
  _factor?: Element
  /** 
   * Total item cost
   */
  net?: Money;
  /** 
   * Unique device identifier
   */
  udi?: Array<Reference>;
  /** 
   * Anatomical location
   */
  bodySite?: CodeableConcept;
  /** 
   * Anatomical sub-location
   */
  subSite?: Array<CodeableConcept>;
  /** 
   * Encounters related to this billed item
   */
  encounter?: Array<Reference>;
  /** 
   * Product or service provided
   */
  detail?: Array<ClaimItemDetail>;
}
export interface Claim {
resourceType: "Claim"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business Identifier for claim
   */
  identifier?: Array<Identifier>;
  /** 
   * active | cancelled | draft | entered-in-error
   */
  status: code;
  /** 
   * active | cancelled | draft | entered-in-error
   */
  _status?: Element
  /** 
   * Category or discipline
   */
  type: CodeableConcept;
  /** 
   * More granular claim type
   */
  subType?: CodeableConcept;
  /** 
   * claim | preauthorization | predetermination
   */
  use: code;
  /** 
   * claim | preauthorization | predetermination
   */
  _use?: Element
  /** 
   * The recipient of the products and services
   */
  patient: Reference;
  /** 
   * Relevant time frame for the claim
   */
  billablePeriod?: Period;
  /** 
   * Resource creation date
   */
  created: dateTime;
  /** 
   * Resource creation date
   */
  _created?: Element
  /** 
   * Author of the claim
   */
  enterer?: Reference;
  /** 
   * Target
   */
  insurer?: Reference;
  /** 
   * Party responsible for the claim
   */
  provider: Reference;
  /** 
   * Desired processing ugency
   */
  priority: CodeableConcept;
  /** 
   * For whom to reserve funds
   */
  fundsReserve?: CodeableConcept;
  /** 
   * Prior or corollary claims
   */
  related?: Array<ClaimRelated>;
  /** 
   * Prescription authorizing services and products
   */
  prescription?: Reference;
  /** 
   * Original prescription if superseded by fulfiller
   */
  originalPrescription?: Reference;
  /** 
   * Recipient of benefits payable
   */
  payee?: ClaimPayee;
  /** 
   * Treatment referral
   */
  referral?: Reference;
  /** 
   * Servicing facility
   */
  facility?: Reference;
  /** 
   * Members of the care team
   */
  careTeam?: Array<ClaimCareTeam>;
  /** 
   * Supporting information
   */
  supportingInfo?: Array<ClaimSupportingInfo>;
  /** 
   * Pertinent diagnosis information
   */
  diagnosis?: Array<ClaimDiagnosis>;
  /** 
   * Clinical procedures performed
   */
  procedure?: Array<ClaimProcedure>;
  /** 
   * Patient insurance information
   */
  insurance: Array<ClaimInsurance>;
  /** 
   * Details of the event
   */
  accident?: ClaimAccident;
  /** 
   * Product or service provided
   */
  item?: Array<ClaimItem>;
  /** 
   * Total claim cost
   */
  total?: Money;
}

export interface ClaimResponseItemAdjudication {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Type of adjudication information
   */
  category: CodeableConcept;
  /** 
   * Explanation of adjudication outcome
   */
  reason?: CodeableConcept;
  /** 
   * Monetary amount
   */
  amount?: Money;
  /** 
   * Non-monetary value
   */
  value?: decimal;
  /** 
   * Non-monetary value
   */
  _value?: Element
}
export interface ClaimResponseItemDetailSubDetail {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Claim sub-detail instance identifier
   */
  subDetailSequence: positiveInt;
  /** 
   * Claim sub-detail instance identifier
   */
  _subDetailSequence?: Element
  /** 
   * Applicable note numbers
   */
  noteNumber?: Array<positiveInt>;
  /** 
   * Applicable note numbers
   */
  _noteNumber?: Array<Element>
  /** 
   * Subdetail level adjudication details
   */
  adjudication?: Array<ClaimResponseItemAdjudication>;
}
export interface ClaimResponseItemDetail {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Claim detail instance identifier
   */
  detailSequence: positiveInt;
  /** 
   * Claim detail instance identifier
   */
  _detailSequence?: Element
  /** 
   * Applicable note numbers
   */
  noteNumber?: Array<positiveInt>;
  /** 
   * Applicable note numbers
   */
  _noteNumber?: Array<Element>
  /** 
   * Detail level adjudication details
   */
  adjudication: Array<ClaimResponseItemAdjudication>;
  /** 
   * Adjudication for claim sub-details
   */
  subDetail?: Array<ClaimResponseItemDetailSubDetail>;
}
export interface ClaimResponseItem {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Claim item instance identifier
   */
  itemSequence: positiveInt;
  /** 
   * Claim item instance identifier
   */
  _itemSequence?: Element
  /** 
   * Applicable note numbers
   */
  noteNumber?: Array<positiveInt>;
  /** 
   * Applicable note numbers
   */
  _noteNumber?: Array<Element>
  /** 
   * Adjudication details
   */
  adjudication: Array<ClaimResponseItemAdjudication>;
  /** 
   * Adjudication for claim details
   */
  detail?: Array<ClaimResponseItemDetail>;
}
export interface ClaimResponseAddItemDetailSubDetail {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Billing, service, product, or drug code
   */
  productOrService: CodeableConcept;
  /** 
   * Service/Product billing modifiers
   */
  modifier?: Array<CodeableConcept>;
  /** 
   * Count of products or services
   */
  quantity?: Quantity;
  /** 
   * Fee, charge or cost per item
   */
  unitPrice?: Money;
  /** 
   * Price scaling factor
   */
  factor?: decimal;
  /** 
   * Price scaling factor
   */
  _factor?: Element
  /** 
   * Total item cost
   */
  net?: Money;
  /** 
   * Applicable note numbers
   */
  noteNumber?: Array<positiveInt>;
  /** 
   * Applicable note numbers
   */
  _noteNumber?: Array<Element>
  /** 
   * Added items detail adjudication
   */
  adjudication: Array<ClaimResponseItemAdjudication>;
}
export interface ClaimResponseAddItemDetail {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Billing, service, product, or drug code
   */
  productOrService: CodeableConcept;
  /** 
   * Service/Product billing modifiers
   */
  modifier?: Array<CodeableConcept>;
  /** 
   * Count of products or services
   */
  quantity?: Quantity;
  /** 
   * Fee, charge or cost per item
   */
  unitPrice?: Money;
  /** 
   * Price scaling factor
   */
  factor?: decimal;
  /** 
   * Price scaling factor
   */
  _factor?: Element
  /** 
   * Total item cost
   */
  net?: Money;
  /** 
   * Applicable note numbers
   */
  noteNumber?: Array<positiveInt>;
  /** 
   * Applicable note numbers
   */
  _noteNumber?: Array<Element>
  /** 
   * Added items detail adjudication
   */
  adjudication: Array<ClaimResponseItemAdjudication>;
  /** 
   * Insurer added line items
   */
  subDetail?: Array<ClaimResponseAddItemDetailSubDetail>;
}
export interface ClaimResponseAddItem {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Item sequence number
   */
  itemSequence?: Array<positiveInt>;
  /** 
   * Item sequence number
   */
  _itemSequence?: Array<Element>
  /** 
   * Detail sequence number
   */
  detailSequence?: Array<positiveInt>;
  /** 
   * Detail sequence number
   */
  _detailSequence?: Array<Element>
  /** 
   * Subdetail sequence number
   */
  subdetailSequence?: Array<positiveInt>;
  /** 
   * Subdetail sequence number
   */
  _subdetailSequence?: Array<Element>
  /** 
   * Authorized providers
   */
  provider?: Array<Reference>;
  /** 
   * Billing, service, product, or drug code
   */
  productOrService: CodeableConcept;
  /** 
   * Service/Product billing modifiers
   */
  modifier?: Array<CodeableConcept>;
  /** 
   * Program the product or service is provided under
   */
  programCode?: Array<CodeableConcept>;
  /** 
   * Date or dates of service or product delivery
   */
  servicedDate?: date;
  /** 
   * Date or dates of service or product delivery
   */
  _servicedDate?: Element
  /** 
   * Date or dates of service or product delivery
   */
  servicedPeriod?: Period;
  /** 
   * Place of service or where product was supplied
   */
  locationCodeableConcept?: CodeableConcept;
  /** 
   * Place of service or where product was supplied
   */
  locationAddress?: Address;
  /** 
   * Place of service or where product was supplied
   */
  locationReference?: Reference;
  /** 
   * Count of products or services
   */
  quantity?: Quantity;
  /** 
   * Fee, charge or cost per item
   */
  unitPrice?: Money;
  /** 
   * Price scaling factor
   */
  factor?: decimal;
  /** 
   * Price scaling factor
   */
  _factor?: Element
  /** 
   * Total item cost
   */
  net?: Money;
  /** 
   * Anatomical location
   */
  bodySite?: CodeableConcept;
  /** 
   * Anatomical sub-location
   */
  subSite?: Array<CodeableConcept>;
  /** 
   * Applicable note numbers
   */
  noteNumber?: Array<positiveInt>;
  /** 
   * Applicable note numbers
   */
  _noteNumber?: Array<Element>
  /** 
   * Added items adjudication
   */
  adjudication: Array<ClaimResponseItemAdjudication>;
  /** 
   * Insurer added line details
   */
  detail?: Array<ClaimResponseAddItemDetail>;
}
export interface ClaimResponseTotal {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Type of adjudication information
   */
  category: CodeableConcept;
  /** 
   * Financial total for the category
   */
  amount: Money;
}
export interface ClaimResponsePayment {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Partial or complete payment
   */
  type: CodeableConcept;
  /** 
   * Payment adjustment for non-claim issues
   */
  adjustment?: Money;
  /** 
   * Explanation for the adjustment
   */
  adjustmentReason?: CodeableConcept;
  /** 
   * Expected date of payment
   */
  date?: date;
  /** 
   * Expected date of payment
   */
  _date?: Element
  /** 
   * Payable amount after adjustment
   */
  amount: Money;
  /** 
   * Business identifier for the payment
   */
  identifier?: Identifier;
}
export interface ClaimResponseProcessNote {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Note instance identifier
   */
  number?: positiveInt;
  /** 
   * Note instance identifier
   */
  _number?: Element
  /** 
   * display | print | printoper
   */
  type?: code;
  /** 
   * display | print | printoper
   */
  _type?: Element
  /** 
   * Note explanatory text
   */
  text: string;
  /** 
   * Note explanatory text
   */
  _text?: Element
  /** 
   * Language of the text
   */
  language?: CodeableConcept;
}
export interface ClaimResponseInsurance {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Insurance instance identifier
   */
  sequence: positiveInt;
  /** 
   * Insurance instance identifier
   */
  _sequence?: Element
  /** 
   * Coverage to be used for adjudication
   */
  focal: boolean;
  /** 
   * Coverage to be used for adjudication
   */
  _focal?: Element
  /** 
   * Insurance information
   */
  coverage: Reference;
  /** 
   * Additional provider contract number
   */
  businessArrangement?: string;
  /** 
   * Additional provider contract number
   */
  _businessArrangement?: Element
  /** 
   * Adjudication results
   */
  claimResponse?: Reference;
}
export interface ClaimResponseError {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Item sequence number
   */
  itemSequence?: positiveInt;
  /** 
   * Item sequence number
   */
  _itemSequence?: Element
  /** 
   * Detail sequence number
   */
  detailSequence?: positiveInt;
  /** 
   * Detail sequence number
   */
  _detailSequence?: Element
  /** 
   * Subdetail sequence number
   */
  subDetailSequence?: positiveInt;
  /** 
   * Subdetail sequence number
   */
  _subDetailSequence?: Element
  /** 
   * Error code detailing processing issues
   */
  code: CodeableConcept;
}
export interface ClaimResponse {
resourceType: "ClaimResponse"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business Identifier for a claim response
   */
  identifier?: Array<Identifier>;
  /** 
   * active | cancelled | draft | entered-in-error
   */
  status: code;
  /** 
   * active | cancelled | draft | entered-in-error
   */
  _status?: Element
  /** 
   * More granular claim type
   */
  type: CodeableConcept;
  /** 
   * More granular claim type
   */
  subType?: CodeableConcept;
  /** 
   * claim | preauthorization | predetermination
   */
  use: code;
  /** 
   * claim | preauthorization | predetermination
   */
  _use?: Element
  /** 
   * The recipient of the products and services
   */
  patient: Reference;
  /** 
   * Response creation date
   */
  created: dateTime;
  /** 
   * Response creation date
   */
  _created?: Element
  /** 
   * Party responsible for reimbursement
   */
  insurer: Reference;
  /** 
   * Party responsible for the claim
   */
  requestor?: Reference;
  /** 
   * Id of resource triggering adjudication
   */
  request?: Reference;
  /** 
   * queued | complete | error | partial
   */
  outcome: code;
  /** 
   * queued | complete | error | partial
   */
  _outcome?: Element
  /** 
   * Disposition Message
   */
  disposition?: string;
  /** 
   * Disposition Message
   */
  _disposition?: Element
  /** 
   * Preauthorization reference
   */
  preAuthRef?: string;
  /** 
   * Preauthorization reference
   */
  _preAuthRef?: Element
  /** 
   * Preauthorization reference effective period
   */
  preAuthPeriod?: Period;
  /** 
   * Party to be paid any benefits payable
   */
  payeeType?: CodeableConcept;
  /** 
   * Adjudication for claim line items
   */
  item?: Array<ClaimResponseItem>;
  /** 
   * Insurer added line items
   */
  addItem?: Array<ClaimResponseAddItem>;
  /** 
   * Header-level adjudication
   */
  adjudication?: Array<ClaimResponseItemAdjudication>;
  /** 
   * Adjudication totals
   */
  total?: Array<ClaimResponseTotal>;
  /** 
   * Payment Details
   */
  payment?: ClaimResponsePayment;
  /** 
   * Funds reserved status
   */
  fundsReserve?: CodeableConcept;
  /** 
   * Printed form identifier
   */
  formCode?: CodeableConcept;
  /** 
   * Printed reference or actual form
   */
  form?: Attachment;
  /** 
   * Note concerning adjudication
   */
  processNote?: Array<ClaimResponseProcessNote>;
  /** 
   * Request for additional information
   */
  communicationRequest?: Array<Reference>;
  /** 
   * Patient insurance information
   */
  insurance?: Array<ClaimResponseInsurance>;
  /** 
   * Processing errors
   */
  error?: Array<ClaimResponseError>;
}

export interface ClinicalImpressionInvestigation {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * A name/code for the set
   */
  code: CodeableConcept;
  /** 
   * Record of a specific investigation
   */
  item?: Array<Reference>;
}
export interface ClinicalImpressionFinding {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * What was found
   */
  itemCodeableConcept?: CodeableConcept;
  /** 
   * What was found
   */
  itemReference?: Reference;
  /** 
   * Which investigations support finding
   */
  basis?: string;
  /** 
   * Which investigations support finding
   */
  _basis?: Element
}
export interface ClinicalImpression {
resourceType: "ClinicalImpression"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business identifier
   */
  identifier?: Array<Identifier>;
  /** 
   * in-progress | completed | entered-in-error
   */
  status: code;
  /** 
   * in-progress | completed | entered-in-error
   */
  _status?: Element
  /** 
   * Reason for current status
   */
  statusReason?: CodeableConcept;
  /** 
   * Kind of assessment performed
   */
  code?: CodeableConcept;
  /** 
   * Why/how the assessment was performed
   */
  description?: string;
  /** 
   * Why/how the assessment was performed
   */
  _description?: Element
  /** 
   * Patient or group assessed
   */
  subject: Reference;
  /** 
   * Encounter created as part of
   */
  encounter?: Reference;
  /** 
   * Time of assessment
   */
  effectiveDateTime?: dateTime;
  /** 
   * Time of assessment
   */
  _effectiveDateTime?: Element
  /** 
   * Time of assessment
   */
  effectivePeriod?: Period;
  /** 
   * When the assessment was documented
   */
  date?: dateTime;
  /** 
   * When the assessment was documented
   */
  _date?: Element
  /** 
   * The clinician performing the assessment
   */
  assessor?: Reference;
  /** 
   * Reference to last assessment
   */
  previous?: Reference;
  /** 
   * Relevant impressions of patient state
   */
  problem?: Array<Reference>;
  /** 
   * One or more sets of investigations (signs, symptoms, etc.)
   */
  investigation?: Array<ClinicalImpressionInvestigation>;
  /** 
   * Clinical Protocol followed
   */
  protocol?: Array<uri>;
  /** 
   * Clinical Protocol followed
   */
  _protocol?: Array<Element>
  /** 
   * Summary of the assessment
   */
  summary?: string;
  /** 
   * Summary of the assessment
   */
  _summary?: Element
  /** 
   * Possible or likely findings and diagnoses
   */
  finding?: Array<ClinicalImpressionFinding>;
  /** 
   * Estimate of likely outcome
   */
  prognosisCodeableConcept?: Array<CodeableConcept>;
  /** 
   * RiskAssessment expressing likely outcome
   */
  prognosisReference?: Array<Reference>;
  /** 
   * Information supporting the clinical impression
   */
  supportingInfo?: Array<Reference>;
  /** 
   * Comments made about the ClinicalImpression
   */
  note?: Array<Annotation>;
}

export interface ClinicalUseDefinitionContraindicationOtherTherapy {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The type of relationship between the product indication/contraindication and another therapy
   */
  relationshipType: CodeableConcept;
  /** 
   * Reference to a specific medication as part of an indication or contraindication
   */
  therapy: CodeableReference;
}
export interface ClinicalUseDefinitionContraindication {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The situation that is being documented as contraindicating against this item
   */
  diseaseSymptomProcedure?: CodeableReference;
  /** 
   * The status of the disease or symptom for the contraindication
   */
  diseaseStatus?: CodeableReference;
  /** 
   * A comorbidity (concurrent condition) or coinfection
   */
  comorbidity?: Array<CodeableReference>;
  /** 
   * The indication which this is a contraidication for
   */
  indication?: Array<Reference>;
  /** 
   * Information about use of the product in relation to other therapies described as part of the contraindication
   */
  otherTherapy?: Array<ClinicalUseDefinitionContraindicationOtherTherapy>;
}
export interface ClinicalUseDefinitionIndication {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The situation that is being documented as an indicaton for this item
   */
  diseaseSymptomProcedure?: CodeableReference;
  /** 
   * The status of the disease or symptom for the indication
   */
  diseaseStatus?: CodeableReference;
  /** 
   * A comorbidity or coinfection as part of the indication
   */
  comorbidity?: Array<CodeableReference>;
  /** 
   * The intended effect, aim or strategy to be achieved
   */
  intendedEffect?: CodeableReference;
  /** 
   * Timing or duration information
   */
  durationRange?: Range;
  /** 
   * Timing or duration information
   */
  durationString?: string;
  /** 
   * Timing or duration information
   */
  _durationString?: Element
  /** 
   * An unwanted side effect or negative outcome of the subject of this resource when being used for this indication
   */
  undesirableEffect?: Array<Reference>;
  /** 
   * The use of the medicinal product in relation to other therapies described as part of the indication
   */
  otherTherapy?: Array<ClinicalUseDefinitionContraindicationOtherTherapy>;
}
export interface ClinicalUseDefinitionInteractionInteractant {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The specific medication, food or laboratory test that interacts
   */
  itemReference?: Reference;
  /** 
   * The specific medication, food or laboratory test that interacts
   */
  itemCodeableConcept?: CodeableConcept;
}
export interface ClinicalUseDefinitionInteraction {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The specific medication, food, substance or laboratory test that interacts
   */
  interactant?: Array<ClinicalUseDefinitionInteractionInteractant>;
  /** 
   * The type of the interaction e.g. drug-drug interaction, drug-lab test interaction
   */
  type?: CodeableConcept;
  /** 
   * The effect of the interaction, for example "reduced gastric absorption of primary medication"
   */
  effect?: CodeableReference;
  /** 
   * The incidence of the interaction, e.g. theoretical, observed
   */
  incidence?: CodeableConcept;
  /** 
   * Actions for managing the interaction
   */
  management?: Array<CodeableConcept>;
}
export interface ClinicalUseDefinitionUndesirableEffect {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The situation in which the undesirable effect may manifest
   */
  symptomConditionEffect?: CodeableReference;
  /** 
   * High level classification of the effect
   */
  classification?: CodeableConcept;
  /** 
   * How often the effect is seen
   */
  frequencyOfOccurrence?: CodeableConcept;
}
export interface ClinicalUseDefinitionWarning {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * A textual definition of this warning, with formatting
   */
  description?: markdown;
  /** 
   * A textual definition of this warning, with formatting
   */
  _description?: Element
  /** 
   * A coded or unformatted textual definition of this warning
   */
  code?: CodeableConcept;
}
export interface ClinicalUseDefinition {
resourceType: "ClinicalUseDefinition"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business identifier for this issue
   */
  identifier?: Array<Identifier>;
  /** 
   * indication | contraindication | interaction | undesirable-effect | warning
   */
  type: code;
  /** 
   * indication | contraindication | interaction | undesirable-effect | warning
   */
  _type?: Element
  /** 
   * A categorisation of the issue, primarily for dividing warnings into subject heading areas such as "Pregnancy", "Overdose"
   */
  category?: Array<CodeableConcept>;
  /** 
   * The medication or procedure for which this is an indication
   */
  subject?: Array<Reference>;
  /** 
   * Whether this is a current issue or one that has been retired etc
   */
  status?: CodeableConcept;
  /** 
   * Specifics for when this is a contraindication
   */
  contraindication?: ClinicalUseDefinitionContraindication;
  /** 
   * Specifics for when this is an indication
   */
  indication?: ClinicalUseDefinitionIndication;
  /** 
   * Specifics for when this is an interaction
   */
  interaction?: ClinicalUseDefinitionInteraction;
  /** 
   * The population group to which this applies
   */
  population?: Array<Reference>;
  /** 
   * A possible negative outcome from the use of this treatment
   */
  undesirableEffect?: ClinicalUseDefinitionUndesirableEffect;
  /** 
   * Critical environmental, health or physical risks or hazards. For example 'Do not operate heavy machinery', 'May cause drowsiness'
   */
  warning?: ClinicalUseDefinitionWarning;
}

export interface CodeSystemFilter {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Code that identifies the filter
   */
  code: code;
  /** 
   * Code that identifies the filter
   */
  _code?: Element
  /** 
   * How or why the filter is used
   */
  description?: string;
  /** 
   * How or why the filter is used
   */
  _description?: Element
  /** 
   * = | is-a | descendent-of | is-not-a | regex | in | not-in | generalizes | exists
   */
  operator: Array<code>;
  /** 
   * = | is-a | descendent-of | is-not-a | regex | in | not-in | generalizes | exists
   */
  _operator?: Array<Element>
  /** 
   * What to use for the value
   */
  value: string;
  /** 
   * What to use for the value
   */
  _value?: Element
}
export interface CodeSystemProperty {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Identifies the property on the concepts, and when referred to in operations
   */
  code: code;
  /** 
   * Identifies the property on the concepts, and when referred to in operations
   */
  _code?: Element
  /** 
   * Formal identifier for the property
   */
  uri?: uri;
  /** 
   * Formal identifier for the property
   */
  _uri?: Element
  /** 
   * Why the property is defined, and/or what it conveys
   */
  description?: string;
  /** 
   * Why the property is defined, and/or what it conveys
   */
  _description?: Element
  /** 
   * code | Coding | string | integer | boolean | dateTime | decimal
   */
  type: code;
  /** 
   * code | Coding | string | integer | boolean | dateTime | decimal
   */
  _type?: Element
}
export interface CodeSystemConceptDesignation {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Human language of the designation
   */
  language?: code;
  /** 
   * Human language of the designation
   */
  _language?: Element
  /** 
   * Details how this designation would be used
   */
  use?: Coding;
  /** 
   * The text value for this designation
   */
  value: string;
  /** 
   * The text value for this designation
   */
  _value?: Element
}
export interface CodeSystemConceptProperty {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Reference to CodeSystem.property.code
   */
  code: code;
  /** 
   * Reference to CodeSystem.property.code
   */
  _code?: Element
  /** 
   * Value of the property for this concept
   */
  valueCode?: code;
  /** 
   * Value of the property for this concept
   */
  _valueCode?: Element
  /** 
   * Value of the property for this concept
   */
  valueCoding?: Coding;
  /** 
   * Value of the property for this concept
   */
  valueString?: string;
  /** 
   * Value of the property for this concept
   */
  _valueString?: Element
  /** 
   * Value of the property for this concept
   */
  valueInteger?: integer;
  /** 
   * Value of the property for this concept
   */
  _valueInteger?: Element
  /** 
   * Value of the property for this concept
   */
  valueBoolean?: boolean;
  /** 
   * Value of the property for this concept
   */
  _valueBoolean?: Element
  /** 
   * Value of the property for this concept
   */
  valueDateTime?: dateTime;
  /** 
   * Value of the property for this concept
   */
  _valueDateTime?: Element
  /** 
   * Value of the property for this concept
   */
  valueDecimal?: decimal;
  /** 
   * Value of the property for this concept
   */
  _valueDecimal?: Element
}
export interface CodeSystemConcept {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Code that identifies concept
   */
  code: code;
  /** 
   * Code that identifies concept
   */
  _code?: Element
  /** 
   * Text to display to the user
   */
  display?: string;
  /** 
   * Text to display to the user
   */
  _display?: Element
  /** 
   * Formal definition
   */
  definition?: string;
  /** 
   * Formal definition
   */
  _definition?: Element
  /** 
   * Additional representations for the concept
   */
  designation?: Array<CodeSystemConceptDesignation>;
  /** 
   * Property value for the concept
   */
  property?: Array<CodeSystemConceptProperty>;
  /** 
   * Child Concepts (is-a/contains/categorizes)
   */
  concept?: Array<CodeSystemConcept>;
}
export interface CodeSystem {
resourceType: "CodeSystem"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Canonical identifier for this code system, represented as a URI (globally unique) (Coding.system)
   */
  url?: uri;
  /** 
   * Canonical identifier for this code system, represented as a URI (globally unique) (Coding.system)
   */
  _url?: Element
  /** 
   * Additional identifier for the code system (business identifier)
   */
  identifier?: Array<Identifier>;
  /** 
   * Business version of the code system (Coding.version)
   */
  version?: string;
  /** 
   * Business version of the code system (Coding.version)
   */
  _version?: Element
  /** 
   * Name for this code system (computer friendly)
   */
  name?: string;
  /** 
   * Name for this code system (computer friendly)
   */
  _name?: Element
  /** 
   * Name for this code system (human friendly)
   */
  title?: string;
  /** 
   * Name for this code system (human friendly)
   */
  _title?: Element
  /** 
   * draft | active | retired | unknown
   */
  status: code;
  /** 
   * draft | active | retired | unknown
   */
  _status?: Element
  /** 
   * For testing purposes, not real usage
   */
  experimental?: boolean;
  /** 
   * For testing purposes, not real usage
   */
  _experimental?: Element
  /** 
   * Date last changed
   */
  date?: dateTime;
  /** 
   * Date last changed
   */
  _date?: Element
  /** 
   * Name of the publisher (organization or individual)
   */
  publisher?: string;
  /** 
   * Name of the publisher (organization or individual)
   */
  _publisher?: Element
  /** 
   * Contact details for the publisher
   */
  contact?: Array<ContactDetail>;
  /** 
   * Natural language description of the code system
   */
  description?: markdown;
  /** 
   * Natural language description of the code system
   */
  _description?: Element
  /** 
   * The context that the content is intended to support
   */
  useContext?: Array<UsageContext>;
  /** 
   * Intended jurisdiction for code system (if applicable)
   */
  jurisdiction?: Array<CodeableConcept>;
  /** 
   * Why this code system is defined
   */
  purpose?: markdown;
  /** 
   * Why this code system is defined
   */
  _purpose?: Element
  /** 
   * Use and/or publishing restrictions
   */
  copyright?: markdown;
  /** 
   * Use and/or publishing restrictions
   */
  _copyright?: Element
  /** 
   * If code comparison is case sensitive
   */
  caseSensitive?: boolean;
  /** 
   * If code comparison is case sensitive
   */
  _caseSensitive?: Element
  /** 
   * Canonical reference to the value set with entire code system
   */
  valueSet?: canonical;
  /** 
   * Canonical reference to the value set with entire code system
   */
  _valueSet?: Element
  /** 
   * grouped-by | is-a | part-of | classified-with
   */
  hierarchyMeaning?: code;
  /** 
   * grouped-by | is-a | part-of | classified-with
   */
  _hierarchyMeaning?: Element
  /** 
   * If code system defines a compositional grammar
   */
  compositional?: boolean;
  /** 
   * If code system defines a compositional grammar
   */
  _compositional?: Element
  /** 
   * If definitions are not stable
   */
  versionNeeded?: boolean;
  /** 
   * If definitions are not stable
   */
  _versionNeeded?: Element
  /** 
   * not-present | example | fragment | complete | supplement
   */
  content: code;
  /** 
   * not-present | example | fragment | complete | supplement
   */
  _content?: Element
  /** 
   * Canonical URL of Code System this adds designations and properties to
   */
  supplements?: canonical;
  /** 
   * Canonical URL of Code System this adds designations and properties to
   */
  _supplements?: Element
  /** 
   * Total concepts in the code system
   */
  count?: unsignedInt;
  /** 
   * Total concepts in the code system
   */
  _count?: Element
  /** 
   * Filter that can be used in a value set
   */
  filter?: Array<CodeSystemFilter>;
  /** 
   * Additional information supplied about each concept
   */
  property?: Array<CodeSystemProperty>;
  /** 
   * Concepts in the code system
   */
  concept?: Array<CodeSystemConcept>;
}

export interface CommunicationPayload {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Message part content
   */
  contentString?: string;
  /** 
   * Message part content
   */
  _contentString?: Element
  /** 
   * Message part content
   */
  contentAttachment?: Attachment;
  /** 
   * Message part content
   */
  contentReference?: Reference;
}
export interface Communication {
resourceType: "Communication"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Unique identifier
   */
  identifier?: Array<Identifier>;
  /** 
   * Instantiates FHIR protocol or definition
   */
  instantiatesCanonical?: Array<canonical>;
  /** 
   * Instantiates FHIR protocol or definition
   */
  _instantiatesCanonical?: Array<Element>
  /** 
   * Instantiates external protocol or definition
   */
  instantiatesUri?: Array<uri>;
  /** 
   * Instantiates external protocol or definition
   */
  _instantiatesUri?: Array<Element>
  /** 
   * Request fulfilled by this communication
   */
  basedOn?: Array<Reference>;
  /** 
   * Part of this action
   */
  partOf?: Array<Reference>;
  /** 
   * Reply to
   */
  inResponseTo?: Array<Reference>;
  /** 
   * preparation | in-progress | not-done | on-hold | stopped | completed | entered-in-error | unknown
   */
  status: code;
  /** 
   * preparation | in-progress | not-done | on-hold | stopped | completed | entered-in-error | unknown
   */
  _status?: Element
  /** 
   * Reason for current status
   */
  statusReason?: CodeableConcept;
  /** 
   * Message category
   */
  category?: Array<CodeableConcept>;
  /** 
   * routine | urgent | asap | stat
   */
  priority?: code;
  /** 
   * routine | urgent | asap | stat
   */
  _priority?: Element
  /** 
   * A channel of communication
   */
  medium?: Array<CodeableConcept>;
  /** 
   * Focus of message
   */
  subject?: Reference;
  /** 
   * Description of the purpose/content
   */
  topic?: CodeableConcept;
  /** 
   * Resources that pertain to this communication
   */
  about?: Array<Reference>;
  /** 
   * Encounter created as part of
   */
  encounter?: Reference;
  /** 
   * When sent
   */
  sent?: dateTime;
  /** 
   * When sent
   */
  _sent?: Element
  /** 
   * When received
   */
  received?: dateTime;
  /** 
   * When received
   */
  _received?: Element
  /** 
   * Message recipient
   */
  recipient?: Array<Reference>;
  /** 
   * Message sender
   */
  sender?: Reference;
  /** 
   * Indication for message
   */
  reasonCode?: Array<CodeableConcept>;
  /** 
   * Why was communication done?
   */
  reasonReference?: Array<Reference>;
  /** 
   * Message payload
   */
  payload?: Array<CommunicationPayload>;
  /** 
   * Comments made about the communication
   */
  note?: Array<Annotation>;
}

export interface CommunicationRequestPayload {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Message part content
   */
  contentString?: string;
  /** 
   * Message part content
   */
  _contentString?: Element
  /** 
   * Message part content
   */
  contentAttachment?: Attachment;
  /** 
   * Message part content
   */
  contentReference?: Reference;
}
export interface CommunicationRequest {
resourceType: "CommunicationRequest"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Unique identifier
   */
  identifier?: Array<Identifier>;
  /** 
   * Fulfills plan or proposal
   */
  basedOn?: Array<Reference>;
  /** 
   * Request(s) replaced by this request
   */
  replaces?: Array<Reference>;
  /** 
   * Composite request this is part of
   */
  groupIdentifier?: Identifier;
  /** 
   * draft | active | on-hold | revoked | completed | entered-in-error | unknown
   */
  status: code;
  /** 
   * draft | active | on-hold | revoked | completed | entered-in-error | unknown
   */
  _status?: Element
  /** 
   * Reason for current status
   */
  statusReason?: CodeableConcept;
  /** 
   * Message category
   */
  category?: Array<CodeableConcept>;
  /** 
   * routine | urgent | asap | stat
   */
  priority?: code;
  /** 
   * routine | urgent | asap | stat
   */
  _priority?: Element
  /** 
   * True if request is prohibiting action
   */
  doNotPerform?: boolean;
  /** 
   * True if request is prohibiting action
   */
  _doNotPerform?: Element
  /** 
   * A channel of communication
   */
  medium?: Array<CodeableConcept>;
  /** 
   * Focus of message
   */
  subject?: Reference;
  /** 
   * Resources that pertain to this communication request
   */
  about?: Array<Reference>;
  /** 
   * Encounter created as part of
   */
  encounter?: Reference;
  /** 
   * Message payload
   */
  payload?: Array<CommunicationRequestPayload>;
  /** 
   * When scheduled
   */
  occurrenceDateTime?: dateTime;
  /** 
   * When scheduled
   */
  _occurrenceDateTime?: Element
  /** 
   * When scheduled
   */
  occurrencePeriod?: Period;
  /** 
   * When request transitioned to being actionable
   */
  authoredOn?: dateTime;
  /** 
   * When request transitioned to being actionable
   */
  _authoredOn?: Element
  /** 
   * Who/what is requesting service
   */
  requester?: Reference;
  /** 
   * Message recipient
   */
  recipient?: Array<Reference>;
  /** 
   * Message sender
   */
  sender?: Reference;
  /** 
   * Why is communication needed?
   */
  reasonCode?: Array<CodeableConcept>;
  /** 
   * Why is communication needed?
   */
  reasonReference?: Array<Reference>;
  /** 
   * Comments made about communication request
   */
  note?: Array<Annotation>;
}

export interface CompartmentDefinitionResource {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Name of resource type
   */
  code: code;
  /** 
   * Name of resource type
   */
  _code?: Element
  /** 
   * Search Parameter Name, or chained parameters
   */
  param?: Array<string>;
  /** 
   * Search Parameter Name, or chained parameters
   */
  _param?: Array<Element>
  /** 
   * Additional documentation about the resource and compartment
   */
  documentation?: string;
  /** 
   * Additional documentation about the resource and compartment
   */
  _documentation?: Element
}
export interface CompartmentDefinition {
resourceType: "CompartmentDefinition"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Canonical identifier for this compartment definition, represented as a URI (globally unique)
   */
  url: uri;
  /** 
   * Canonical identifier for this compartment definition, represented as a URI (globally unique)
   */
  _url?: Element
  /** 
   * Business version of the compartment definition
   */
  version?: string;
  /** 
   * Business version of the compartment definition
   */
  _version?: Element
  /** 
   * Name for this compartment definition (computer friendly)
   */
  name: string;
  /** 
   * Name for this compartment definition (computer friendly)
   */
  _name?: Element
  /** 
   * draft | active | retired | unknown
   */
  status: code;
  /** 
   * draft | active | retired | unknown
   */
  _status?: Element
  /** 
   * For testing purposes, not real usage
   */
  experimental?: boolean;
  /** 
   * For testing purposes, not real usage
   */
  _experimental?: Element
  /** 
   * Date last changed
   */
  date?: dateTime;
  /** 
   * Date last changed
   */
  _date?: Element
  /** 
   * Name of the publisher (organization or individual)
   */
  publisher?: string;
  /** 
   * Name of the publisher (organization or individual)
   */
  _publisher?: Element
  /** 
   * Contact details for the publisher
   */
  contact?: Array<ContactDetail>;
  /** 
   * Natural language description of the compartment definition
   */
  description?: markdown;
  /** 
   * Natural language description of the compartment definition
   */
  _description?: Element
  /** 
   * The context that the content is intended to support
   */
  useContext?: Array<UsageContext>;
  /** 
   * Why this compartment definition is defined
   */
  purpose?: markdown;
  /** 
   * Why this compartment definition is defined
   */
  _purpose?: Element
  /** 
   * Patient | Encounter | RelatedPerson | Practitioner | Device
   */
  code: code;
  /** 
   * Patient | Encounter | RelatedPerson | Practitioner | Device
   */
  _code?: Element
  /** 
   * Whether the search syntax is supported
   */
  search: boolean;
  /** 
   * Whether the search syntax is supported
   */
  _search?: Element
  /** 
   * How a resource is related to the compartment
   */
  resource?: Array<CompartmentDefinitionResource>;
}

export interface CompositionAttester {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * personal | professional | legal | official
   */
  mode: code;
  /** 
   * personal | professional | legal | official
   */
  _mode?: Element
  /** 
   * When the composition was attested
   */
  time?: dateTime;
  /** 
   * When the composition was attested
   */
  _time?: Element
  /** 
   * Who attested the composition
   */
  party?: Reference;
}
export interface CompositionRelatesTo {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * replaces | transforms | signs | appends
   */
  code: code;
  /** 
   * replaces | transforms | signs | appends
   */
  _code?: Element
  /** 
   * Target of the relationship
   */
  targetIdentifier?: Identifier;
  /** 
   * Target of the relationship
   */
  targetReference?: Reference;
}
export interface CompositionEvent {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Code(s) that apply to the event being documented
   */
  code?: Array<CodeableConcept>;
  /** 
   * The period covered by the documentation
   */
  period?: Period;
  /** 
   * The event(s) being documented
   */
  detail?: Array<Reference>;
}
export interface CompositionSection {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Label for section (e.g. for ToC)
   */
  title?: string;
  /** 
   * Label for section (e.g. for ToC)
   */
  _title?: Element
  /** 
   * Classification of section (recommended)
   */
  code?: CodeableConcept;
  /** 
   * Who and/or what authored the section
   */
  author?: Array<Reference>;
  /** 
   * Who/what the section is about, when it is not about the subject of composition
   */
  focus?: Reference;
  /** 
   * Text summary of the section, for human interpretation
   */
  text?: Narrative;
  /** 
   * working | snapshot | changes
   */
  mode?: code;
  /** 
   * working | snapshot | changes
   */
  _mode?: Element
  /** 
   * Order of section entries
   */
  orderedBy?: CodeableConcept;
  /** 
   * A reference to data that supports this section
   */
  entry?: Array<Reference>;
  /** 
   * Why the section is empty
   */
  emptyReason?: CodeableConcept;
  /** 
   * Nested Section
   */
  section?: Array<CompositionSection>;
}
export interface Composition {
resourceType: "Composition"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Version-independent identifier for the Composition
   */
  identifier?: Identifier;
  /** 
   * preliminary | final | amended | entered-in-error
   */
  status: code;
  /** 
   * preliminary | final | amended | entered-in-error
   */
  _status?: Element
  /** 
   * Kind of composition (LOINC if possible)
   */
  type: CodeableConcept;
  /** 
   * Categorization of Composition
   */
  category?: Array<CodeableConcept>;
  /** 
   * Who and/or what the composition is about
   */
  subject?: Reference;
  /** 
   * Context of the Composition
   */
  encounter?: Reference;
  /** 
   * Composition editing time
   */
  date: dateTime;
  /** 
   * Composition editing time
   */
  _date?: Element
  /** 
   * Who and/or what authored the composition
   */
  author: Array<Reference>;
  /** 
   * Human Readable name/title
   */
  title: string;
  /** 
   * Human Readable name/title
   */
  _title?: Element
  /** 
   * As defined by affinity domain
   */
  confidentiality?: code;
  /** 
   * As defined by affinity domain
   */
  _confidentiality?: Element
  /** 
   * Attests to accuracy of composition
   */
  attester?: Array<CompositionAttester>;
  /** 
   * Organization which maintains the composition
   */
  custodian?: Reference;
  /** 
   * Relationships to other compositions/documents
   */
  relatesTo?: Array<CompositionRelatesTo>;
  /** 
   * The clinical service(s) being documented
   */
  event?: Array<CompositionEvent>;
  /** 
   * Composition is broken into sections
   */
  section?: Array<CompositionSection>;
}

export interface ConceptMapGroupElementTargetDependsOn {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Reference to property mapping depends on
   */
  property: uri;
  /** 
   * Reference to property mapping depends on
   */
  _property?: Element
  /** 
   * Code System (if necessary)
   */
  system?: canonical;
  /** 
   * Code System (if necessary)
   */
  _system?: Element
  /** 
   * Value of the referenced element
   */
  value: string;
  /** 
   * Value of the referenced element
   */
  _value?: Element
  /** 
   * Display for the code (if value is a code)
   */
  display?: string;
  /** 
   * Display for the code (if value is a code)
   */
  _display?: Element
}
export interface ConceptMapGroupElementTarget {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Code that identifies the target element
   */
  code?: code;
  /** 
   * Code that identifies the target element
   */
  _code?: Element
  /** 
   * Display for the code
   */
  display?: string;
  /** 
   * Display for the code
   */
  _display?: Element
  /** 
   * relatedto | equivalent | equal | wider | subsumes | narrower | specializes | inexact | unmatched | disjoint
   */
  equivalence: code;
  /** 
   * relatedto | equivalent | equal | wider | subsumes | narrower | specializes | inexact | unmatched | disjoint
   */
  _equivalence?: Element
  /** 
   * Description of status/issues in mapping
   */
  comment?: string;
  /** 
   * Description of status/issues in mapping
   */
  _comment?: Element
  /** 
   * Other elements required for this mapping (from context)
   */
  dependsOn?: Array<ConceptMapGroupElementTargetDependsOn>;
  /** 
   * Other concepts that this mapping also produces
   */
  product?: Array<ConceptMapGroupElementTargetDependsOn>;
}
export interface ConceptMapGroupElement {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Identifies element being mapped
   */
  code?: code;
  /** 
   * Identifies element being mapped
   */
  _code?: Element
  /** 
   * Display for the code
   */
  display?: string;
  /** 
   * Display for the code
   */
  _display?: Element
  /** 
   * Concept in target system for element
   */
  target?: Array<ConceptMapGroupElementTarget>;
}
export interface ConceptMapGroupUnmapped {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * provided | fixed | other-map
   */
  mode: code;
  /** 
   * provided | fixed | other-map
   */
  _mode?: Element
  /** 
   * Fixed code when mode = fixed
   */
  code?: code;
  /** 
   * Fixed code when mode = fixed
   */
  _code?: Element
  /** 
   * Display for the code
   */
  display?: string;
  /** 
   * Display for the code
   */
  _display?: Element
  /** 
   * canonical reference to an additional ConceptMap to use for mapping if the source concept is unmapped
   */
  url?: canonical;
  /** 
   * canonical reference to an additional ConceptMap to use for mapping if the source concept is unmapped
   */
  _url?: Element
}
export interface ConceptMapGroup {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Source system where concepts to be mapped are defined
   */
  source?: uri;
  /** 
   * Source system where concepts to be mapped are defined
   */
  _source?: Element
  /** 
   * Specific version of the  code system
   */
  sourceVersion?: string;
  /** 
   * Specific version of the  code system
   */
  _sourceVersion?: Element
  /** 
   * Target system that the concepts are to be mapped to
   */
  target?: uri;
  /** 
   * Target system that the concepts are to be mapped to
   */
  _target?: Element
  /** 
   * Specific version of the  code system
   */
  targetVersion?: string;
  /** 
   * Specific version of the  code system
   */
  _targetVersion?: Element
  /** 
   * Mappings for a concept from the source set
   */
  element: Array<ConceptMapGroupElement>;
  /** 
   * What to do when there is no mapping for the source concept
   */
  unmapped?: ConceptMapGroupUnmapped;
}
export interface ConceptMap {
resourceType: "ConceptMap"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Canonical identifier for this concept map, represented as a URI (globally unique)
   */
  url?: uri;
  /** 
   * Canonical identifier for this concept map, represented as a URI (globally unique)
   */
  _url?: Element
  /** 
   * Additional identifier for the concept map
   */
  identifier?: Identifier;
  /** 
   * Business version of the concept map
   */
  version?: string;
  /** 
   * Business version of the concept map
   */
  _version?: Element
  /** 
   * Name for this concept map (computer friendly)
   */
  name?: string;
  /** 
   * Name for this concept map (computer friendly)
   */
  _name?: Element
  /** 
   * Name for this concept map (human friendly)
   */
  title?: string;
  /** 
   * Name for this concept map (human friendly)
   */
  _title?: Element
  /** 
   * draft | active | retired | unknown
   */
  status: code;
  /** 
   * draft | active | retired | unknown
   */
  _status?: Element
  /** 
   * For testing purposes, not real usage
   */
  experimental?: boolean;
  /** 
   * For testing purposes, not real usage
   */
  _experimental?: Element
  /** 
   * Date last changed
   */
  date?: dateTime;
  /** 
   * Date last changed
   */
  _date?: Element
  /** 
   * Name of the publisher (organization or individual)
   */
  publisher?: string;
  /** 
   * Name of the publisher (organization or individual)
   */
  _publisher?: Element
  /** 
   * Contact details for the publisher
   */
  contact?: Array<ContactDetail>;
  /** 
   * Natural language description of the concept map
   */
  description?: markdown;
  /** 
   * Natural language description of the concept map
   */
  _description?: Element
  /** 
   * The context that the content is intended to support
   */
  useContext?: Array<UsageContext>;
  /** 
   * Intended jurisdiction for concept map (if applicable)
   */
  jurisdiction?: Array<CodeableConcept>;
  /** 
   * Why this concept map is defined
   */
  purpose?: markdown;
  /** 
   * Why this concept map is defined
   */
  _purpose?: Element
  /** 
   * Use and/or publishing restrictions
   */
  copyright?: markdown;
  /** 
   * Use and/or publishing restrictions
   */
  _copyright?: Element
  /** 
   * The source value set that contains the concepts that are being mapped
   */
  sourceUri?: uri;
  /** 
   * The source value set that contains the concepts that are being mapped
   */
  _sourceUri?: Element
  /** 
   * The source value set that contains the concepts that are being mapped
   */
  sourceCanonical?: canonical;
  /** 
   * The source value set that contains the concepts that are being mapped
   */
  _sourceCanonical?: Element
  /** 
   * The target value set which provides context for the mappings
   */
  targetUri?: uri;
  /** 
   * The target value set which provides context for the mappings
   */
  _targetUri?: Element
  /** 
   * The target value set which provides context for the mappings
   */
  targetCanonical?: canonical;
  /** 
   * The target value set which provides context for the mappings
   */
  _targetCanonical?: Element
  /** 
   * Same source and target systems
   */
  group?: Array<ConceptMapGroup>;
}

export interface ConditionStage {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Simple summary (disease specific)
   */
  summary?: CodeableConcept;
  /** 
   * Formal record of assessment
   */
  assessment?: Array<Reference>;
  /** 
   * Kind of staging
   */
  type?: CodeableConcept;
}
export interface ConditionEvidence {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Manifestation/symptom
   */
  code?: Array<CodeableConcept>;
  /** 
   * Supporting information found elsewhere
   */
  detail?: Array<Reference>;
}
export interface Condition {
resourceType: "Condition"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * External Ids for this condition
   */
  identifier?: Array<Identifier>;
  /** 
   * active | recurrence | relapse | inactive | remission | resolved
   */
  clinicalStatus?: CodeableConcept;
  /** 
   * unconfirmed | provisional | differential | confirmed | refuted | entered-in-error
   */
  verificationStatus?: CodeableConcept;
  /** 
   * problem-list-item | encounter-diagnosis
   */
  category?: Array<CodeableConcept>;
  /** 
   * Subjective severity of condition
   */
  severity?: CodeableConcept;
  /** 
   * Identification of the condition, problem or diagnosis
   */
  code?: CodeableConcept;
  /** 
   * Anatomical location, if relevant
   */
  bodySite?: Array<CodeableConcept>;
  /** 
   * Who has the condition?
   */
  subject: Reference;
  /** 
   * Encounter created as part of
   */
  encounter?: Reference;
  /** 
   * Estimated or actual date,  date-time, or age
   */
  onsetDateTime?: dateTime;
  /** 
   * Estimated or actual date,  date-time, or age
   */
  _onsetDateTime?: Element
  /** 
   * Estimated or actual date,  date-time, or age
   */
  onsetAge?: Age;
  /** 
   * Estimated or actual date,  date-time, or age
   */
  onsetPeriod?: Period;
  /** 
   * Estimated or actual date,  date-time, or age
   */
  onsetRange?: Range;
  /** 
   * Estimated or actual date,  date-time, or age
   */
  onsetString?: string;
  /** 
   * Estimated or actual date,  date-time, or age
   */
  _onsetString?: Element
  /** 
   * When in resolution/remission
   */
  abatementDateTime?: dateTime;
  /** 
   * When in resolution/remission
   */
  _abatementDateTime?: Element
  /** 
   * When in resolution/remission
   */
  abatementAge?: Age;
  /** 
   * When in resolution/remission
   */
  abatementPeriod?: Period;
  /** 
   * When in resolution/remission
   */
  abatementRange?: Range;
  /** 
   * When in resolution/remission
   */
  abatementString?: string;
  /** 
   * When in resolution/remission
   */
  _abatementString?: Element
  /** 
   * Date record was first recorded
   */
  recordedDate?: dateTime;
  /** 
   * Date record was first recorded
   */
  _recordedDate?: Element
  /** 
   * Who recorded the condition
   */
  recorder?: Reference;
  /** 
   * Person who asserts this condition
   */
  asserter?: Reference;
  /** 
   * Stage/grade, usually assessed formally
   */
  stage?: Array<ConditionStage>;
  /** 
   * Supporting evidence
   */
  evidence?: Array<ConditionEvidence>;
  /** 
   * Additional information about the Condition
   */
  note?: Array<Annotation>;
}

export interface ConsentPolicy {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Enforcement source for policy
   */
  authority?: uri;
  /** 
   * Enforcement source for policy
   */
  _authority?: Element
  /** 
   * Specific policy covered by this consent
   */
  uri?: uri;
  /** 
   * Specific policy covered by this consent
   */
  _uri?: Element
}
export interface ConsentVerification {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Has been verified
   */
  verified: boolean;
  /** 
   * Has been verified
   */
  _verified?: Element
  /** 
   * Person who verified
   */
  verifiedWith?: Reference;
  /** 
   * When consent verified
   */
  verificationDate?: dateTime;
  /** 
   * When consent verified
   */
  _verificationDate?: Element
}
export interface ConsentProvisionActor {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * How the actor is involved
   */
  role: CodeableConcept;
  /** 
   * Resource for the actor (or group, by role)
   */
  reference: Reference;
}
export interface ConsentProvisionData {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * instance | related | dependents | authoredby
   */
  meaning: code;
  /** 
   * instance | related | dependents | authoredby
   */
  _meaning?: Element
  /** 
   * The actual data reference
   */
  reference: Reference;
}
export interface ConsentProvision {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * deny | permit
   */
  type?: code;
  /** 
   * deny | permit
   */
  _type?: Element
  /** 
   * Timeframe for this rule
   */
  period?: Period;
  /** 
   * Who|what controlled by this rule (or group, by role)
   */
  actor?: Array<ConsentProvisionActor>;
  /** 
   * Actions controlled by this rule
   */
  action?: Array<CodeableConcept>;
  /** 
   * Security Labels that define affected resources
   */
  securityLabel?: Array<Coding>;
  /** 
   * Context of activities covered by this rule
   */
  purpose?: Array<Coding>;
  /** 
   * e.g. Resource Type, Profile, CDA, etc.
   */
  class?: Array<Coding>;
  /** 
   * e.g. LOINC or SNOMED CT code, etc. in the content
   */
  code?: Array<CodeableConcept>;
  /** 
   * Timeframe for data controlled by this rule
   */
  dataPeriod?: Period;
  /** 
   * Data controlled by this rule
   */
  data?: Array<ConsentProvisionData>;
  /** 
   * Nested Exception Rules
   */
  provision?: Array<ConsentProvision>;
}
export interface Consent {
resourceType: "Consent"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Identifier for this record (external references)
   */
  identifier?: Array<Identifier>;
  /** 
   * draft | proposed | active | rejected | inactive | entered-in-error
   */
  status: code;
  /** 
   * draft | proposed | active | rejected | inactive | entered-in-error
   */
  _status?: Element
  /** 
   * Which of the four areas this resource covers (extensible)
   */
  scope: CodeableConcept;
  /** 
   * Classification of the consent statement - for indexing/retrieval
   */
  category: Array<CodeableConcept>;
  /** 
   * Who the consent applies to
   */
  patient?: Reference;
  /** 
   * When this Consent was created or indexed
   */
  dateTime?: dateTime;
  /** 
   * When this Consent was created or indexed
   */
  _dateTime?: Element
  /** 
   * Who is agreeing to the policy and rules
   */
  performer?: Array<Reference>;
  /** 
   * Custodian of the consent
   */
  organization?: Array<Reference>;
  /** 
   * Source from which this consent is taken
   */
  sourceAttachment?: Attachment;
  /** 
   * Source from which this consent is taken
   */
  sourceReference?: Reference;
  /** 
   * Policies covered by this consent
   */
  policy?: Array<ConsentPolicy>;
  /** 
   * Regulation that this consents to
   */
  policyRule?: CodeableConcept;
  /** 
   * Consent Verified by patient or family
   */
  verification?: Array<ConsentVerification>;
  /** 
   * Constraints to the base Consent.policyRule
   */
  provision?: ConsentProvision;
}

export interface ContractContentDefinition {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Content structure and use
   */
  type: CodeableConcept;
  /** 
   * Detailed Content Type Definition
   */
  subType?: CodeableConcept;
  /** 
   * Publisher Entity
   */
  publisher?: Reference;
  /** 
   * When published
   */
  publicationDate?: dateTime;
  /** 
   * When published
   */
  _publicationDate?: Element
  /** 
   * amended | appended | cancelled | disputed | entered-in-error | executable | executed | negotiable | offered | policy | rejected | renewed | revoked | resolved | terminated
   */
  publicationStatus: code;
  /** 
   * amended | appended | cancelled | disputed | entered-in-error | executable | executed | negotiable | offered | policy | rejected | renewed | revoked | resolved | terminated
   */
  _publicationStatus?: Element
  /** 
   * Publication Ownership
   */
  copyright?: markdown;
  /** 
   * Publication Ownership
   */
  _copyright?: Element
}
export interface ContractTermSecurityLabel {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Link to Security Labels
   */
  number?: Array<unsignedInt>;
  /** 
   * Link to Security Labels
   */
  _number?: Array<Element>
  /** 
   * Confidentiality Protection
   */
  classification: Coding;
  /** 
   * Applicable Policy
   */
  category?: Array<Coding>;
  /** 
   * Handling Instructions
   */
  control?: Array<Coding>;
}
export interface ContractTermOfferParty {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Referenced entity
   */
  reference: Array<Reference>;
  /** 
   * Participant engagement type
   */
  role: CodeableConcept;
}
export interface ContractTermOfferAnswer {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The actual answer response
   */
  valueBoolean?: boolean;
  /** 
   * The actual answer response
   */
  _valueBoolean?: Element
  /** 
   * The actual answer response
   */
  valueDecimal?: decimal;
  /** 
   * The actual answer response
   */
  _valueDecimal?: Element
  /** 
   * The actual answer response
   */
  valueInteger?: integer;
  /** 
   * The actual answer response
   */
  _valueInteger?: Element
  /** 
   * The actual answer response
   */
  valueDate?: date;
  /** 
   * The actual answer response
   */
  _valueDate?: Element
  /** 
   * The actual answer response
   */
  valueDateTime?: dateTime;
  /** 
   * The actual answer response
   */
  _valueDateTime?: Element
  /** 
   * The actual answer response
   */
  valueTime?: time;
  /** 
   * The actual answer response
   */
  _valueTime?: Element
  /** 
   * The actual answer response
   */
  valueString?: string;
  /** 
   * The actual answer response
   */
  _valueString?: Element
  /** 
   * The actual answer response
   */
  valueUri?: uri;
  /** 
   * The actual answer response
   */
  _valueUri?: Element
  /** 
   * The actual answer response
   */
  valueAttachment?: Attachment;
  /** 
   * The actual answer response
   */
  valueCoding?: Coding;
  /** 
   * The actual answer response
   */
  valueQuantity?: Quantity;
  /** 
   * The actual answer response
   */
  valueReference?: Reference;
}
export interface ContractTermOffer {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Offer business ID
   */
  identifier?: Array<Identifier>;
  /** 
   * Offer Recipient
   */
  party?: Array<ContractTermOfferParty>;
  /** 
   * Negotiable offer asset
   */
  topic?: Reference;
  /** 
   * Contract Offer Type or Form
   */
  type?: CodeableConcept;
  /** 
   * Accepting party choice
   */
  decision?: CodeableConcept;
  /** 
   * How decision is conveyed
   */
  decisionMode?: Array<CodeableConcept>;
  /** 
   * Response to offer text
   */
  answer?: Array<ContractTermOfferAnswer>;
  /** 
   * Human readable offer text
   */
  text?: string;
  /** 
   * Human readable offer text
   */
  _text?: Element
  /** 
   * Pointer to text
   */
  linkId?: Array<string>;
  /** 
   * Pointer to text
   */
  _linkId?: Array<Element>
  /** 
   * Offer restriction numbers
   */
  securityLabelNumber?: Array<unsignedInt>;
  /** 
   * Offer restriction numbers
   */
  _securityLabelNumber?: Array<Element>
}
export interface ContractTermAssetContext {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Creator,custodian or owner
   */
  reference?: Reference;
  /** 
   * Codeable asset context
   */
  code?: Array<CodeableConcept>;
  /** 
   * Context description
   */
  text?: string;
  /** 
   * Context description
   */
  _text?: Element
}
export interface ContractTermAssetValuedItem {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Contract Valued Item Type
   */
  entityCodeableConcept?: CodeableConcept;
  /** 
   * Contract Valued Item Type
   */
  entityReference?: Reference;
  /** 
   * Contract Valued Item Number
   */
  identifier?: Identifier;
  /** 
   * Contract Valued Item Effective Tiem
   */
  effectiveTime?: dateTime;
  /** 
   * Contract Valued Item Effective Tiem
   */
  _effectiveTime?: Element
  /** 
   * Count of Contract Valued Items
   */
  quantity?: Quantity;
  /** 
   * Contract Valued Item fee, charge, or cost
   */
  unitPrice?: Money;
  /** 
   * Contract Valued Item Price Scaling Factor
   */
  factor?: decimal;
  /** 
   * Contract Valued Item Price Scaling Factor
   */
  _factor?: Element
  /** 
   * Contract Valued Item Difficulty Scaling Factor
   */
  points?: decimal;
  /** 
   * Contract Valued Item Difficulty Scaling Factor
   */
  _points?: Element
  /** 
   * Total Contract Valued Item Value
   */
  net?: Money;
  /** 
   * Terms of valuation
   */
  payment?: string;
  /** 
   * Terms of valuation
   */
  _payment?: Element
  /** 
   * When payment is due
   */
  paymentDate?: dateTime;
  /** 
   * When payment is due
   */
  _paymentDate?: Element
  /** 
   * Who will make payment
   */
  responsible?: Reference;
  /** 
   * Who will receive payment
   */
  recipient?: Reference;
  /** 
   * Pointer to specific item
   */
  linkId?: Array<string>;
  /** 
   * Pointer to specific item
   */
  _linkId?: Array<Element>
  /** 
   * Security Labels that define affected terms
   */
  securityLabelNumber?: Array<unsignedInt>;
  /** 
   * Security Labels that define affected terms
   */
  _securityLabelNumber?: Array<Element>
}
export interface ContractTermAsset {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Range of asset
   */
  scope?: CodeableConcept;
  /** 
   * Asset category
   */
  type?: Array<CodeableConcept>;
  /** 
   * Associated entities
   */
  typeReference?: Array<Reference>;
  /** 
   * Asset sub-category
   */
  subtype?: Array<CodeableConcept>;
  /** 
   * Kinship of the asset
   */
  relationship?: Coding;
  /** 
   * Circumstance of the asset
   */
  context?: Array<ContractTermAssetContext>;
  /** 
   * Quality desctiption of asset
   */
  condition?: string;
  /** 
   * Quality desctiption of asset
   */
  _condition?: Element
  /** 
   * Asset availability types
   */
  periodType?: Array<CodeableConcept>;
  /** 
   * Time period of the asset
   */
  period?: Array<Period>;
  /** 
   * Time period
   */
  usePeriod?: Array<Period>;
  /** 
   * Asset clause or question text
   */
  text?: string;
  /** 
   * Asset clause or question text
   */
  _text?: Element
  /** 
   * Pointer to asset text
   */
  linkId?: Array<string>;
  /** 
   * Pointer to asset text
   */
  _linkId?: Array<Element>
  /** 
   * Response to assets
   */
  answer?: Array<ContractTermOfferAnswer>;
  /** 
   * Asset restriction numbers
   */
  securityLabelNumber?: Array<unsignedInt>;
  /** 
   * Asset restriction numbers
   */
  _securityLabelNumber?: Array<Element>
  /** 
   * Contract Valued Item List
   */
  valuedItem?: Array<ContractTermAssetValuedItem>;
}
export interface ContractTermActionSubject {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Entity of the action
   */
  reference: Array<Reference>;
  /** 
   * Role type of the agent
   */
  role?: CodeableConcept;
}
export interface ContractTermAction {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * True if the term prohibits the  action
   */
  doNotPerform?: boolean;
  /** 
   * True if the term prohibits the  action
   */
  _doNotPerform?: Element
  /** 
   * Type or form of the action
   */
  type: CodeableConcept;
  /** 
   * Entity of the action
   */
  subject?: Array<ContractTermActionSubject>;
  /** 
   * Purpose for the Contract Term Action
   */
  intent: CodeableConcept;
  /** 
   * Pointer to specific item
   */
  linkId?: Array<string>;
  /** 
   * Pointer to specific item
   */
  _linkId?: Array<Element>
  /** 
   * State of the action
   */
  status: CodeableConcept;
  /** 
   * Episode associated with action
   */
  context?: Reference;
  /** 
   * Pointer to specific item
   */
  contextLinkId?: Array<string>;
  /** 
   * Pointer to specific item
   */
  _contextLinkId?: Array<Element>
  /** 
   * When action happens
   */
  occurrenceDateTime?: dateTime;
  /** 
   * When action happens
   */
  _occurrenceDateTime?: Element
  /** 
   * When action happens
   */
  occurrencePeriod?: Period;
  /** 
   * When action happens
   */
  occurrenceTiming?: Timing;
  /** 
   * Who asked for action
   */
  requester?: Array<Reference>;
  /** 
   * Pointer to specific item
   */
  requesterLinkId?: Array<string>;
  /** 
   * Pointer to specific item
   */
  _requesterLinkId?: Array<Element>
  /** 
   * Kind of service performer
   */
  performerType?: Array<CodeableConcept>;
  /** 
   * Competency of the performer
   */
  performerRole?: CodeableConcept;
  /** 
   * Actor that wil execute (or not) the action
   */
  performer?: Reference;
  /** 
   * Pointer to specific item
   */
  performerLinkId?: Array<string>;
  /** 
   * Pointer to specific item
   */
  _performerLinkId?: Array<Element>
  /** 
   * Why is action (not) needed?
   */
  reasonCode?: Array<CodeableConcept>;
  /** 
   * Why is action (not) needed?
   */
  reasonReference?: Array<Reference>;
  /** 
   * Why action is to be performed
   */
  reason?: Array<string>;
  /** 
   * Why action is to be performed
   */
  _reason?: Array<Element>
  /** 
   * Pointer to specific item
   */
  reasonLinkId?: Array<string>;
  /** 
   * Pointer to specific item
   */
  _reasonLinkId?: Array<Element>
  /** 
   * Comments about the action
   */
  note?: Array<Annotation>;
  /** 
   * Action restriction numbers
   */
  securityLabelNumber?: Array<unsignedInt>;
  /** 
   * Action restriction numbers
   */
  _securityLabelNumber?: Array<Element>
}
export interface ContractTerm {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Contract Term Number
   */
  identifier?: Identifier;
  /** 
   * Contract Term Issue Date Time
   */
  issued?: dateTime;
  /** 
   * Contract Term Issue Date Time
   */
  _issued?: Element
  /** 
   * Contract Term Effective Time
   */
  applies?: Period;
  /** 
   * Term Concern
   */
  topicCodeableConcept?: CodeableConcept;
  /** 
   * Term Concern
   */
  topicReference?: Reference;
  /** 
   * Contract Term Type or Form
   */
  type?: CodeableConcept;
  /** 
   * Contract Term Type specific classification
   */
  subType?: CodeableConcept;
  /** 
   * Term Statement
   */
  text?: string;
  /** 
   * Term Statement
   */
  _text?: Element
  /** 
   * Protection for the Term
   */
  securityLabel?: Array<ContractTermSecurityLabel>;
  /** 
   * Context of the Contract term
   */
  offer: ContractTermOffer;
  /** 
   * Contract Term Asset List
   */
  asset?: Array<ContractTermAsset>;
  /** 
   * Entity being ascribed responsibility
   */
  action?: Array<ContractTermAction>;
  /** 
   * Nested Contract Term Group
   */
  group?: Array<ContractTerm>;
}
export interface ContractSigner {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Contract Signatory Role
   */
  type: Coding;
  /** 
   * Contract Signatory Party
   */
  party: Reference;
  /** 
   * Contract Documentation Signature
   */
  signature: Array<Signature>;
}
export interface ContractFriendly {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Easily comprehended representation of this Contract
   */
  contentAttachment?: Attachment;
  /** 
   * Easily comprehended representation of this Contract
   */
  contentReference?: Reference;
}
export interface ContractLegal {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Contract Legal Text
   */
  contentAttachment?: Attachment;
  /** 
   * Contract Legal Text
   */
  contentReference?: Reference;
}
export interface ContractRule {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Computable Contract Rules
   */
  contentAttachment?: Attachment;
  /** 
   * Computable Contract Rules
   */
  contentReference?: Reference;
}
export interface Contract {
resourceType: "Contract"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Contract number
   */
  identifier?: Array<Identifier>;
  /** 
   * Basal definition
   */
  url?: uri;
  /** 
   * Basal definition
   */
  _url?: Element
  /** 
   * Business edition
   */
  version?: string;
  /** 
   * Business edition
   */
  _version?: Element
  /** 
   * amended | appended | cancelled | disputed | entered-in-error | executable | executed | negotiable | offered | policy | rejected | renewed | revoked | resolved | terminated
   */
  status?: code;
  /** 
   * amended | appended | cancelled | disputed | entered-in-error | executable | executed | negotiable | offered | policy | rejected | renewed | revoked | resolved | terminated
   */
  _status?: Element
  /** 
   * Negotiation status
   */
  legalState?: CodeableConcept;
  /** 
   * Source Contract Definition
   */
  instantiatesCanonical?: Reference;
  /** 
   * External Contract Definition
   */
  instantiatesUri?: uri;
  /** 
   * External Contract Definition
   */
  _instantiatesUri?: Element
  /** 
   * Content derived from the basal information
   */
  contentDerivative?: CodeableConcept;
  /** 
   * When this Contract was issued
   */
  issued?: dateTime;
  /** 
   * When this Contract was issued
   */
  _issued?: Element
  /** 
   * Effective time
   */
  applies?: Period;
  /** 
   * Contract cessation cause
   */
  expirationType?: CodeableConcept;
  /** 
   * Contract Target Entity
   */
  subject?: Array<Reference>;
  /** 
   * Authority under which this Contract has standing
   */
  authority?: Array<Reference>;
  /** 
   * A sphere of control governed by an authoritative jurisdiction, organization, or person
   */
  domain?: Array<Reference>;
  /** 
   * Specific Location
   */
  site?: Array<Reference>;
  /** 
   * Computer friendly designation
   */
  name?: string;
  /** 
   * Computer friendly designation
   */
  _name?: Element
  /** 
   * Human Friendly name
   */
  title?: string;
  /** 
   * Human Friendly name
   */
  _title?: Element
  /** 
   * Subordinate Friendly name
   */
  subtitle?: string;
  /** 
   * Subordinate Friendly name
   */
  _subtitle?: Element
  /** 
   * Acronym or short name
   */
  alias?: Array<string>;
  /** 
   * Acronym or short name
   */
  _alias?: Array<Element>
  /** 
   * Source of Contract
   */
  author?: Reference;
  /** 
   * Range of Legal Concerns
   */
  scope?: CodeableConcept;
  /** 
   * Focus of contract interest
   */
  topicCodeableConcept?: CodeableConcept;
  /** 
   * Focus of contract interest
   */
  topicReference?: Reference;
  /** 
   * Legal instrument category
   */
  type?: CodeableConcept;
  /** 
   * Subtype within the context of type
   */
  subType?: Array<CodeableConcept>;
  /** 
   * Contract precursor content
   */
  contentDefinition?: ContractContentDefinition;
  /** 
   * Contract Term List
   */
  term?: Array<ContractTerm>;
  /** 
   * Extra Information
   */
  supportingInfo?: Array<Reference>;
  /** 
   * Key event in Contract History
   */
  relevantHistory?: Array<Reference>;
  /** 
   * Contract Signatory
   */
  signer?: Array<ContractSigner>;
  /** 
   * Contract Friendly Language
   */
  friendly?: Array<ContractFriendly>;
  /** 
   * Contract Legal Language
   */
  legal?: Array<ContractLegal>;
  /** 
   * Computable Contract Language
   */
  rule?: Array<ContractRule>;
  /** 
   * Binding Contract
   */
  legallyBindingAttachment?: Attachment;
  /** 
   * Binding Contract
   */
  legallyBindingReference?: Reference;
}

export interface CoverageClass {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Type of class such as 'group' or 'plan'
   */
  type: CodeableConcept;
  /** 
   * Value associated with the type
   */
  value: string;
  /** 
   * Value associated with the type
   */
  _value?: Element
  /** 
   * Human readable description of the type and value
   */
  name?: string;
  /** 
   * Human readable description of the type and value
   */
  _name?: Element
}
export interface CoverageCostToBeneficiaryException {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Exception category
   */
  type: CodeableConcept;
  /** 
   * The effective period of the exception
   */
  period?: Period;
}
export interface CoverageCostToBeneficiary {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Cost category
   */
  type?: CodeableConcept;
  /** 
   * The amount or percentage due from the beneficiary
   */
  valueQuantity?: Quantity;
  /** 
   * The amount or percentage due from the beneficiary
   */
  valueMoney?: Money;
  /** 
   * Exceptions for patient payments
   */
  exception?: Array<CoverageCostToBeneficiaryException>;
}
export interface Coverage {
resourceType: "Coverage"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business Identifier for the coverage
   */
  identifier?: Array<Identifier>;
  /** 
   * active | cancelled | draft | entered-in-error
   */
  status: code;
  /** 
   * active | cancelled | draft | entered-in-error
   */
  _status?: Element
  /** 
   * Coverage category such as medical or accident
   */
  type?: CodeableConcept;
  /** 
   * Owner of the policy
   */
  policyHolder?: Reference;
  /** 
   * Subscriber to the policy
   */
  subscriber?: Reference;
  /** 
   * ID assigned to the subscriber
   */
  subscriberId?: string;
  /** 
   * ID assigned to the subscriber
   */
  _subscriberId?: Element
  /** 
   * Plan beneficiary
   */
  beneficiary: Reference;
  /** 
   * Dependent number
   */
  dependent?: string;
  /** 
   * Dependent number
   */
  _dependent?: Element
  /** 
   * Beneficiary relationship to the subscriber
   */
  relationship?: CodeableConcept;
  /** 
   * Coverage start and end dates
   */
  period?: Period;
  /** 
   * Issuer of the policy
   */
  payor: Array<Reference>;
  /** 
   * Additional coverage classifications
   */
  class?: Array<CoverageClass>;
  /** 
   * Relative order of the coverage
   */
  order?: positiveInt;
  /** 
   * Relative order of the coverage
   */
  _order?: Element
  /** 
   * Insurer network
   */
  network?: string;
  /** 
   * Insurer network
   */
  _network?: Element
  /** 
   * Patient payments for services/products
   */
  costToBeneficiary?: Array<CoverageCostToBeneficiary>;
  /** 
   * Reimbursement to insurer
   */
  subrogation?: boolean;
  /** 
   * Reimbursement to insurer
   */
  _subrogation?: Element
  /** 
   * Contract details
   */
  contract?: Array<Reference>;
}

export interface CoverageEligibilityRequestSupportingInfo {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Information instance identifier
   */
  sequence: positiveInt;
  /** 
   * Information instance identifier
   */
  _sequence?: Element
  /** 
   * Data to be provided
   */
  information: Reference;
  /** 
   * Applies to all items
   */
  appliesToAll?: boolean;
  /** 
   * Applies to all items
   */
  _appliesToAll?: Element
}
export interface CoverageEligibilityRequestInsurance {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Applicable coverage
   */
  focal?: boolean;
  /** 
   * Applicable coverage
   */
  _focal?: Element
  /** 
   * Insurance information
   */
  coverage: Reference;
  /** 
   * Additional provider contract number
   */
  businessArrangement?: string;
  /** 
   * Additional provider contract number
   */
  _businessArrangement?: Element
}
export interface CoverageEligibilityRequestItemDiagnosis {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Nature of illness or problem
   */
  diagnosisCodeableConcept?: CodeableConcept;
  /** 
   * Nature of illness or problem
   */
  diagnosisReference?: Reference;
}
export interface CoverageEligibilityRequestItem {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Applicable exception or supporting information
   */
  supportingInfoSequence?: Array<positiveInt>;
  /** 
   * Applicable exception or supporting information
   */
  _supportingInfoSequence?: Array<Element>
  /** 
   * Benefit classification
   */
  category?: CodeableConcept;
  /** 
   * Billing, service, product, or drug code
   */
  productOrService?: CodeableConcept;
  /** 
   * Product or service billing modifiers
   */
  modifier?: Array<CodeableConcept>;
  /** 
   * Perfoming practitioner
   */
  provider?: Reference;
  /** 
   * Count of products or services
   */
  quantity?: Quantity;
  /** 
   * Fee, charge or cost per item
   */
  unitPrice?: Money;
  /** 
   * Servicing facility
   */
  facility?: Reference;
  /** 
   * Applicable diagnosis
   */
  diagnosis?: Array<CoverageEligibilityRequestItemDiagnosis>;
  /** 
   * Product or service details
   */
  detail?: Array<Reference>;
}
export interface CoverageEligibilityRequest {
resourceType: "CoverageEligibilityRequest"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business Identifier for coverage eligiblity request
   */
  identifier?: Array<Identifier>;
  /** 
   * active | cancelled | draft | entered-in-error
   */
  status: code;
  /** 
   * active | cancelled | draft | entered-in-error
   */
  _status?: Element
  /** 
   * Desired processing priority
   */
  priority?: CodeableConcept;
  /** 
   * auth-requirements | benefits | discovery | validation
   */
  purpose: Array<code>;
  /** 
   * auth-requirements | benefits | discovery | validation
   */
  _purpose?: Array<Element>
  /** 
   * Intended recipient of products and services
   */
  patient: Reference;
  /** 
   * Estimated date or dates of service
   */
  servicedDate?: date;
  /** 
   * Estimated date or dates of service
   */
  _servicedDate?: Element
  /** 
   * Estimated date or dates of service
   */
  servicedPeriod?: Period;
  /** 
   * Creation date
   */
  created: dateTime;
  /** 
   * Creation date
   */
  _created?: Element
  /** 
   * Author
   */
  enterer?: Reference;
  /** 
   * Party responsible for the request
   */
  provider?: Reference;
  /** 
   * Coverage issuer
   */
  insurer: Reference;
  /** 
   * Servicing facility
   */
  facility?: Reference;
  /** 
   * Supporting information
   */
  supportingInfo?: Array<CoverageEligibilityRequestSupportingInfo>;
  /** 
   * Patient insurance information
   */
  insurance?: Array<CoverageEligibilityRequestInsurance>;
  /** 
   * Item to be evaluated for eligibiity
   */
  item?: Array<CoverageEligibilityRequestItem>;
}

export interface CoverageEligibilityResponseInsuranceItemBenefit {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Benefit classification
   */
  type: CodeableConcept;
  /** 
   * Benefits allowed
   */
  allowedUnsignedInt?: unsignedInt;
  /** 
   * Benefits allowed
   */
  _allowedUnsignedInt?: Element
  /** 
   * Benefits allowed
   */
  allowedString?: string;
  /** 
   * Benefits allowed
   */
  _allowedString?: Element
  /** 
   * Benefits allowed
   */
  allowedMoney?: Money;
  /** 
   * Benefits used
   */
  usedUnsignedInt?: unsignedInt;
  /** 
   * Benefits used
   */
  _usedUnsignedInt?: Element
  /** 
   * Benefits used
   */
  usedString?: string;
  /** 
   * Benefits used
   */
  _usedString?: Element
  /** 
   * Benefits used
   */
  usedMoney?: Money;
}
export interface CoverageEligibilityResponseInsuranceItem {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Benefit classification
   */
  category?: CodeableConcept;
  /** 
   * Billing, service, product, or drug code
   */
  productOrService?: CodeableConcept;
  /** 
   * Product or service billing modifiers
   */
  modifier?: Array<CodeableConcept>;
  /** 
   * Performing practitioner
   */
  provider?: Reference;
  /** 
   * Excluded from the plan
   */
  excluded?: boolean;
  /** 
   * Excluded from the plan
   */
  _excluded?: Element
  /** 
   * Short name for the benefit
   */
  name?: string;
  /** 
   * Short name for the benefit
   */
  _name?: Element
  /** 
   * Description of the benefit or services covered
   */
  description?: string;
  /** 
   * Description of the benefit or services covered
   */
  _description?: Element
  /** 
   * In or out of network
   */
  network?: CodeableConcept;
  /** 
   * Individual or family
   */
  unit?: CodeableConcept;
  /** 
   * Annual or lifetime
   */
  term?: CodeableConcept;
  /** 
   * Benefit Summary
   */
  benefit?: Array<CoverageEligibilityResponseInsuranceItemBenefit>;
  /** 
   * Authorization required flag
   */
  authorizationRequired?: boolean;
  /** 
   * Authorization required flag
   */
  _authorizationRequired?: Element
  /** 
   * Type of required supporting materials
   */
  authorizationSupporting?: Array<CodeableConcept>;
  /** 
   * Preauthorization requirements endpoint
   */
  authorizationUrl?: uri;
  /** 
   * Preauthorization requirements endpoint
   */
  _authorizationUrl?: Element
}
export interface CoverageEligibilityResponseInsurance {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Insurance information
   */
  coverage: Reference;
  /** 
   * Coverage inforce indicator
   */
  inforce?: boolean;
  /** 
   * Coverage inforce indicator
   */
  _inforce?: Element
  /** 
   * When the benefits are applicable
   */
  benefitPeriod?: Period;
  /** 
   * Benefits and authorization details
   */
  item?: Array<CoverageEligibilityResponseInsuranceItem>;
}
export interface CoverageEligibilityResponseError {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Error code detailing processing issues
   */
  code: CodeableConcept;
}
export interface CoverageEligibilityResponse {
resourceType: "CoverageEligibilityResponse"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business Identifier for coverage eligiblity request
   */
  identifier?: Array<Identifier>;
  /** 
   * active | cancelled | draft | entered-in-error
   */
  status: code;
  /** 
   * active | cancelled | draft | entered-in-error
   */
  _status?: Element
  /** 
   * auth-requirements | benefits | discovery | validation
   */
  purpose: Array<code>;
  /** 
   * auth-requirements | benefits | discovery | validation
   */
  _purpose?: Array<Element>
  /** 
   * Intended recipient of products and services
   */
  patient: Reference;
  /** 
   * Estimated date or dates of service
   */
  servicedDate?: date;
  /** 
   * Estimated date or dates of service
   */
  _servicedDate?: Element
  /** 
   * Estimated date or dates of service
   */
  servicedPeriod?: Period;
  /** 
   * Response creation date
   */
  created: dateTime;
  /** 
   * Response creation date
   */
  _created?: Element
  /** 
   * Party responsible for the request
   */
  requestor?: Reference;
  /** 
   * Eligibility request reference
   */
  request: Reference;
  /** 
   * queued | complete | error | partial
   */
  outcome: code;
  /** 
   * queued | complete | error | partial
   */
  _outcome?: Element
  /** 
   * Disposition Message
   */
  disposition?: string;
  /** 
   * Disposition Message
   */
  _disposition?: Element
  /** 
   * Coverage issuer
   */
  insurer: Reference;
  /** 
   * Patient insurance information
   */
  insurance?: Array<CoverageEligibilityResponseInsurance>;
  /** 
   * Preauthorization reference
   */
  preAuthRef?: string;
  /** 
   * Preauthorization reference
   */
  _preAuthRef?: Element
  /** 
   * Printed form identifier
   */
  form?: CodeableConcept;
  /** 
   * Processing errors
   */
  error?: Array<CoverageEligibilityResponseError>;
}

export interface DetectedIssueEvidence {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Manifestation
   */
  code?: Array<CodeableConcept>;
  /** 
   * Supporting information
   */
  detail?: Array<Reference>;
}
export interface DetectedIssueMitigation {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * What mitigation?
   */
  action: CodeableConcept;
  /** 
   * Date committed
   */
  date?: dateTime;
  /** 
   * Date committed
   */
  _date?: Element
  /** 
   * Who is committing?
   */
  author?: Reference;
}
export interface DetectedIssue {
resourceType: "DetectedIssue"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Unique id for the detected issue
   */
  identifier?: Array<Identifier>;
  /** 
   * registered | preliminary | final | amended +
   */
  status: code;
  /** 
   * registered | preliminary | final | amended +
   */
  _status?: Element
  /** 
   * Issue Category, e.g. drug-drug, duplicate therapy, etc.
   */
  code?: CodeableConcept;
  /** 
   * high | moderate | low
   */
  severity?: code;
  /** 
   * high | moderate | low
   */
  _severity?: Element
  /** 
   * Associated patient
   */
  patient?: Reference;
  /** 
   * When identified
   */
  identifiedDateTime?: dateTime;
  /** 
   * When identified
   */
  _identifiedDateTime?: Element
  /** 
   * When identified
   */
  identifiedPeriod?: Period;
  /** 
   * The provider or device that identified the issue
   */
  author?: Reference;
  /** 
   * Problem resource
   */
  implicated?: Array<Reference>;
  /** 
   * Supporting evidence
   */
  evidence?: Array<DetectedIssueEvidence>;
  /** 
   * Description and context
   */
  detail?: string;
  /** 
   * Description and context
   */
  _detail?: Element
  /** 
   * Authority for issue
   */
  reference?: uri;
  /** 
   * Authority for issue
   */
  _reference?: Element
  /** 
   * Step taken to address
   */
  mitigation?: Array<DetectedIssueMitigation>;
}

export interface DeviceUdiCarrier {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Mandatory fixed portion of UDI
   */
  deviceIdentifier?: string;
  /** 
   * Mandatory fixed portion of UDI
   */
  _deviceIdentifier?: Element
  /** 
   * UDI Issuing Organization
   */
  issuer?: uri;
  /** 
   * UDI Issuing Organization
   */
  _issuer?: Element
  /** 
   * Regional UDI authority
   */
  jurisdiction?: uri;
  /** 
   * Regional UDI authority
   */
  _jurisdiction?: Element
  /** 
   * UDI Machine Readable Barcode String
   */
  carrierAIDC?: base64Binary;
  /** 
   * UDI Machine Readable Barcode String
   */
  _carrierAIDC?: Element
  /** 
   * UDI Human Readable Barcode String
   */
  carrierHRF?: string;
  /** 
   * UDI Human Readable Barcode String
   */
  _carrierHRF?: Element
  /** 
   * barcode | rfid | manual +
   */
  entryType?: code;
  /** 
   * barcode | rfid | manual +
   */
  _entryType?: Element
}
export interface DeviceDeviceName {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The name that identifies the device
   */
  name: string;
  /** 
   * The name that identifies the device
   */
  _name?: Element
  /** 
   * udi-label-name | user-friendly-name | patient-reported-name | manufacturer-name | model-name | other
   */
  type: code;
  /** 
   * udi-label-name | user-friendly-name | patient-reported-name | manufacturer-name | model-name | other
   */
  _type?: Element
}
export interface DeviceSpecialization {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The standard that is used to operate and communicate
   */
  systemType: CodeableConcept;
  /** 
   * The version of the standard that is used to operate and communicate
   */
  version?: string;
  /** 
   * The version of the standard that is used to operate and communicate
   */
  _version?: Element
}
export interface DeviceVersion {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The type of the device version, e.g. manufacturer, approved, internal
   */
  type?: CodeableConcept;
  /** 
   * A single component of the device version
   */
  component?: Identifier;
  /** 
   * The version text
   */
  value: string;
  /** 
   * The version text
   */
  _value?: Element
}
export interface DeviceProperty {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Code that specifies the property DeviceDefinitionPropetyCode (Extensible)
   */
  type: CodeableConcept;
  /** 
   * Property value as a quantity
   */
  valueQuantity?: Array<Quantity>;
  /** 
   * Property value as a code, e.g., NTP4 (synced to NTP)
   */
  valueCode?: Array<CodeableConcept>;
}
export interface Device {
resourceType: "Device"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Instance identifier
   */
  identifier?: Array<Identifier>;
  /** 
   * The reference to the definition for the device
   */
  definition?: Reference;
  /** 
   * Unique Device Identifier (UDI) Barcode string
   */
  udiCarrier?: Array<DeviceUdiCarrier>;
  /** 
   * active | inactive | entered-in-error | unknown
   */
  status?: code;
  /** 
   * active | inactive | entered-in-error | unknown
   */
  _status?: Element
  /** 
   * online | paused | standby | offline | not-ready | transduc-discon | hw-discon | off
   */
  statusReason?: Array<CodeableConcept>;
  /** 
   * The distinct identification string
   */
  distinctIdentifier?: string;
  /** 
   * The distinct identification string
   */
  _distinctIdentifier?: Element
  /** 
   * Name of device manufacturer
   */
  manufacturer?: string;
  /** 
   * Name of device manufacturer
   */
  _manufacturer?: Element
  /** 
   * Date when the device was made
   */
  manufactureDate?: dateTime;
  /** 
   * Date when the device was made
   */
  _manufactureDate?: Element
  /** 
   * Date and time of expiry of this device (if applicable)
   */
  expirationDate?: dateTime;
  /** 
   * Date and time of expiry of this device (if applicable)
   */
  _expirationDate?: Element
  /** 
   * Lot number of manufacture
   */
  lotNumber?: string;
  /** 
   * Lot number of manufacture
   */
  _lotNumber?: Element
  /** 
   * Serial number assigned by the manufacturer
   */
  serialNumber?: string;
  /** 
   * Serial number assigned by the manufacturer
   */
  _serialNumber?: Element
  /** 
   * The name of the device as given by the manufacturer
   */
  deviceName?: Array<DeviceDeviceName>;
  /** 
   * The manufacturer's model number for the device
   */
  modelNumber?: string;
  /** 
   * The manufacturer's model number for the device
   */
  _modelNumber?: Element
  /** 
   * The part number or catalog number of the device
   */
  partNumber?: string;
  /** 
   * The part number or catalog number of the device
   */
  _partNumber?: Element
  /** 
   * The kind or type of device
   */
  type?: CodeableConcept;
  /** 
   * The capabilities supported on a  device, the standards to which the device conforms for a particular purpose, and used for the communication
   */
  specialization?: Array<DeviceSpecialization>;
  /** 
   * The actual design of the device or software version running on the device
   */
  version?: Array<DeviceVersion>;
  /** 
   * The actual configuration settings of a device as it actually operates, e.g., regulation status, time properties
   */
  property?: Array<DeviceProperty>;
  /** 
   * Patient to whom Device is affixed
   */
  patient?: Reference;
  /** 
   * Organization responsible for device
   */
  owner?: Reference;
  /** 
   * Details for human/organization for support
   */
  contact?: Array<ContactPoint>;
  /** 
   * Where the device is found
   */
  location?: Reference;
  /** 
   * Network address to contact device
   */
  url?: uri;
  /** 
   * Network address to contact device
   */
  _url?: Element
  /** 
   * Device notes and comments
   */
  note?: Array<Annotation>;
  /** 
   * Safety Characteristics of Device
   */
  safety?: Array<CodeableConcept>;
  /** 
   * The device that this device is attached to or is part of
   */
  parent?: Reference;
}

export interface DeviceDefinitionUdiDeviceIdentifier {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The identifier that is to be associated with every Device that references this DeviceDefintiion for the issuer and jurisdication porvided in the DeviceDefinition.udiDeviceIdentifier
   */
  deviceIdentifier: string;
  /** 
   * The identifier that is to be associated with every Device that references this DeviceDefintiion for the issuer and jurisdication porvided in the DeviceDefinition.udiDeviceIdentifier
   */
  _deviceIdentifier?: Element
  /** 
   * The organization that assigns the identifier algorithm
   */
  issuer: uri;
  /** 
   * The organization that assigns the identifier algorithm
   */
  _issuer?: Element
  /** 
   * The jurisdiction to which the deviceIdentifier applies
   */
  jurisdiction: uri;
  /** 
   * The jurisdiction to which the deviceIdentifier applies
   */
  _jurisdiction?: Element
}
export interface DeviceDefinitionDeviceName {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The name of the device
   */
  name: string;
  /** 
   * The name of the device
   */
  _name?: Element
  /** 
   * udi-label-name | user-friendly-name | patient-reported-name | manufacturer-name | model-name | other
   */
  type: code;
  /** 
   * udi-label-name | user-friendly-name | patient-reported-name | manufacturer-name | model-name | other
   */
  _type?: Element
}
export interface DeviceDefinitionSpecialization {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The standard that is used to operate and communicate
   */
  systemType: string;
  /** 
   * The standard that is used to operate and communicate
   */
  _systemType?: Element
  /** 
   * The version of the standard that is used to operate and communicate
   */
  version?: string;
  /** 
   * The version of the standard that is used to operate and communicate
   */
  _version?: Element
}
export interface DeviceDefinitionCapability {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Type of capability
   */
  type: CodeableConcept;
  /** 
   * Description of capability
   */
  description?: Array<CodeableConcept>;
}
export interface DeviceDefinitionProperty {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Code that specifies the property DeviceDefinitionPropetyCode (Extensible)
   */
  type: CodeableConcept;
  /** 
   * Property value as a quantity
   */
  valueQuantity?: Array<Quantity>;
  /** 
   * Property value as a code, e.g., NTP4 (synced to NTP)
   */
  valueCode?: Array<CodeableConcept>;
}
export interface DeviceDefinitionMaterial {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The substance
   */
  substance: CodeableConcept;
  /** 
   * Indicates an alternative material of the device
   */
  alternate?: boolean;
  /** 
   * Indicates an alternative material of the device
   */
  _alternate?: Element
  /** 
   * Whether the substance is a known or suspected allergen
   */
  allergenicIndicator?: boolean;
  /** 
   * Whether the substance is a known or suspected allergen
   */
  _allergenicIndicator?: Element
}
export interface DeviceDefinition {
resourceType: "DeviceDefinition"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Instance identifier
   */
  identifier?: Array<Identifier>;
  /** 
   * Unique Device Identifier (UDI) Barcode string
   */
  udiDeviceIdentifier?: Array<DeviceDefinitionUdiDeviceIdentifier>;
  /** 
   * Name of device manufacturer
   */
  manufacturerString?: string;
  /** 
   * Name of device manufacturer
   */
  _manufacturerString?: Element
  /** 
   * Name of device manufacturer
   */
  manufacturerReference?: Reference;
  /** 
   * A name given to the device to identify it
   */
  deviceName?: Array<DeviceDefinitionDeviceName>;
  /** 
   * The model number for the device
   */
  modelNumber?: string;
  /** 
   * The model number for the device
   */
  _modelNumber?: Element
  /** 
   * What kind of device or device system this is
   */
  type?: CodeableConcept;
  /** 
   * The capabilities supported on a  device, the standards to which the device conforms for a particular purpose, and used for the communication
   */
  specialization?: Array<DeviceDefinitionSpecialization>;
  /** 
   * Available versions
   */
  version?: Array<string>;
  /** 
   * Available versions
   */
  _version?: Array<Element>
  /** 
   * Safety characteristics of the device
   */
  safety?: Array<CodeableConcept>;
  /** 
   * Shelf Life and storage information
   */
  shelfLifeStorage?: Array<ProductShelfLife>;
  /** 
   * Dimensions, color etc.
   */
  physicalCharacteristics?: ProdCharacteristic;
  /** 
   * Language code for the human-readable text strings produced by the device (all supported)
   */
  languageCode?: Array<CodeableConcept>;
  /** 
   * Device capabilities
   */
  capability?: Array<DeviceDefinitionCapability>;
  /** 
   * The actual configuration settings of a device as it actually operates, e.g., regulation status, time properties
   */
  property?: Array<DeviceDefinitionProperty>;
  /** 
   * Organization responsible for device
   */
  owner?: Reference;
  /** 
   * Details for human/organization for support
   */
  contact?: Array<ContactPoint>;
  /** 
   * Network address to contact device
   */
  url?: uri;
  /** 
   * Network address to contact device
   */
  _url?: Element
  /** 
   * Access to on-line information
   */
  onlineInformation?: uri;
  /** 
   * Access to on-line information
   */
  _onlineInformation?: Element
  /** 
   * Device notes and comments
   */
  note?: Array<Annotation>;
  /** 
   * The quantity of the device present in the packaging (e.g. the number of devices present in a pack, or the number of devices in the same package of the medicinal product)
   */
  quantity?: Quantity;
  /** 
   * The parent device it can be part of
   */
  parentDevice?: Reference;
  /** 
   * A substance used to create the material(s) of which the device is made
   */
  material?: Array<DeviceDefinitionMaterial>;
}

export interface DeviceMetricCalibration {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * unspecified | offset | gain | two-point
   */
  type?: code;
  /** 
   * unspecified | offset | gain | two-point
   */
  _type?: Element
  /** 
   * not-calibrated | calibration-required | calibrated | unspecified
   */
  state?: code;
  /** 
   * not-calibrated | calibration-required | calibrated | unspecified
   */
  _state?: Element
  /** 
   * Describes the time last calibration has been performed
   */
  time?: instant;
  /** 
   * Describes the time last calibration has been performed
   */
  _time?: Element
}
export interface DeviceMetric {
resourceType: "DeviceMetric"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Instance identifier
   */
  identifier?: Array<Identifier>;
  /** 
   * Identity of metric, for example Heart Rate or PEEP Setting
   */
  type: CodeableConcept;
  /** 
   * Unit of Measure for the Metric
   */
  unit?: CodeableConcept;
  /** 
   * Describes the link to the source Device
   */
  source?: Reference;
  /** 
   * Describes the link to the parent Device
   */
  parent?: Reference;
  /** 
   * on | off | standby | entered-in-error
   */
  operationalStatus?: code;
  /** 
   * on | off | standby | entered-in-error
   */
  _operationalStatus?: Element
  /** 
   * black | red | green | yellow | blue | magenta | cyan | white
   */
  color?: code;
  /** 
   * black | red | green | yellow | blue | magenta | cyan | white
   */
  _color?: Element
  /** 
   * measurement | setting | calculation | unspecified
   */
  category: code;
  /** 
   * measurement | setting | calculation | unspecified
   */
  _category?: Element
  /** 
   * Describes the measurement repetition time
   */
  measurementPeriod?: Timing;
  /** 
   * Describes the calibrations that have been performed or that are required to be performed
   */
  calibration?: Array<DeviceMetricCalibration>;
}

export interface DeviceRequestParameter {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Device detail
   */
  code?: CodeableConcept;
  /** 
   * Value of detail
   */
  valueCodeableConcept?: CodeableConcept;
  /** 
   * Value of detail
   */
  valueQuantity?: Quantity;
  /** 
   * Value of detail
   */
  valueRange?: Range;
  /** 
   * Value of detail
   */
  valueBoolean?: boolean;
  /** 
   * Value of detail
   */
  _valueBoolean?: Element
}
export interface DeviceRequest {
resourceType: "DeviceRequest"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * External Request identifier
   */
  identifier?: Array<Identifier>;
  /** 
   * Instantiates FHIR protocol or definition
   */
  instantiatesCanonical?: Array<canonical>;
  /** 
   * Instantiates FHIR protocol or definition
   */
  _instantiatesCanonical?: Array<Element>
  /** 
   * Instantiates external protocol or definition
   */
  instantiatesUri?: Array<uri>;
  /** 
   * Instantiates external protocol or definition
   */
  _instantiatesUri?: Array<Element>
  /** 
   * What request fulfills
   */
  basedOn?: Array<Reference>;
  /** 
   * What request replaces
   */
  priorRequest?: Array<Reference>;
  /** 
   * Identifier of composite request
   */
  groupIdentifier?: Identifier;
  /** 
   * draft | active | on-hold | revoked | completed | entered-in-error | unknown
   */
  status?: code;
  /** 
   * draft | active | on-hold | revoked | completed | entered-in-error | unknown
   */
  _status?: Element
  /** 
   * proposal | plan | directive | order | original-order | reflex-order | filler-order | instance-order | option
   */
  intent: code;
  /** 
   * proposal | plan | directive | order | original-order | reflex-order | filler-order | instance-order | option
   */
  _intent?: Element
  /** 
   * routine | urgent | asap | stat
   */
  priority?: code;
  /** 
   * routine | urgent | asap | stat
   */
  _priority?: Element
  /** 
   * Device requested
   */
  codeReference?: Reference;
  /** 
   * Device requested
   */
  codeCodeableConcept?: CodeableConcept;
  /** 
   * Device details
   */
  parameter?: Array<DeviceRequestParameter>;
  /** 
   * Focus of request
   */
  subject: Reference;
  /** 
   * Encounter motivating request
   */
  encounter?: Reference;
  /** 
   * Desired time or schedule for use
   */
  occurrenceDateTime?: dateTime;
  /** 
   * Desired time or schedule for use
   */
  _occurrenceDateTime?: Element
  /** 
   * Desired time or schedule for use
   */
  occurrencePeriod?: Period;
  /** 
   * Desired time or schedule for use
   */
  occurrenceTiming?: Timing;
  /** 
   * When recorded
   */
  authoredOn?: dateTime;
  /** 
   * When recorded
   */
  _authoredOn?: Element
  /** 
   * Who/what is requesting diagnostics
   */
  requester?: Reference;
  /** 
   * Filler role
   */
  performerType?: CodeableConcept;
  /** 
   * Requested Filler
   */
  performer?: Reference;
  /** 
   * Coded Reason for request
   */
  reasonCode?: Array<CodeableConcept>;
  /** 
   * Linked Reason for request
   */
  reasonReference?: Array<Reference>;
  /** 
   * Associated insurance coverage
   */
  insurance?: Array<Reference>;
  /** 
   * Additional clinical information
   */
  supportingInfo?: Array<Reference>;
  /** 
   * Notes or comments
   */
  note?: Array<Annotation>;
  /** 
   * Request provenance
   */
  relevantHistory?: Array<Reference>;
}

export interface DeviceUseStatement {
resourceType: "DeviceUseStatement"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * External identifier for this record
   */
  identifier?: Array<Identifier>;
  /** 
   * Fulfills plan, proposal or order
   */
  basedOn?: Array<Reference>;
  /** 
   * active | completed | entered-in-error +
   */
  status: code;
  /** 
   * active | completed | entered-in-error +
   */
  _status?: Element
  /** 
   * Patient using device
   */
  subject: Reference;
  /** 
   * Supporting information
   */
  derivedFrom?: Array<Reference>;
  /** 
   * How often  the device was used
   */
  timingTiming?: Timing;
  /** 
   * How often  the device was used
   */
  timingPeriod?: Period;
  /** 
   * How often  the device was used
   */
  timingDateTime?: dateTime;
  /** 
   * How often  the device was used
   */
  _timingDateTime?: Element
  /** 
   * When statement was recorded
   */
  recordedOn?: dateTime;
  /** 
   * When statement was recorded
   */
  _recordedOn?: Element
  /** 
   * Who made the statement
   */
  source?: Reference;
  /** 
   * Reference to device used
   */
  device: Reference;
  /** 
   * Why device was used
   */
  reasonCode?: Array<CodeableConcept>;
  /** 
   * Why was DeviceUseStatement performed?
   */
  reasonReference?: Array<Reference>;
  /** 
   * Target body site
   */
  bodySite?: CodeableConcept;
  /** 
   * Addition details (comments, instructions)
   */
  note?: Array<Annotation>;
}

export interface DiagnosticReportMedia {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Comment about the image (e.g. explanation)
   */
  comment?: string;
  /** 
   * Comment about the image (e.g. explanation)
   */
  _comment?: Element
  /** 
   * Reference to the image source
   */
  link: Reference;
}
export interface DiagnosticReport {
resourceType: "DiagnosticReport"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business identifier for report
   */
  identifier?: Array<Identifier>;
  /** 
   * What was requested
   */
  basedOn?: Array<Reference>;
  /** 
   * registered | partial | preliminary | final +
   */
  status: code;
  /** 
   * registered | partial | preliminary | final +
   */
  _status?: Element
  /** 
   * Service category
   */
  category?: Array<CodeableConcept>;
  /** 
   * Name/Code for this diagnostic report
   */
  code: CodeableConcept;
  /** 
   * The subject of the report - usually, but not always, the patient
   */
  subject?: Reference;
  /** 
   * Health care event when test ordered
   */
  encounter?: Reference;
  /** 
   * Clinically relevant time/time-period for report
   */
  effectiveDateTime?: dateTime;
  /** 
   * Clinically relevant time/time-period for report
   */
  _effectiveDateTime?: Element
  /** 
   * Clinically relevant time/time-period for report
   */
  effectivePeriod?: Period;
  /** 
   * DateTime this version was made
   */
  issued?: instant;
  /** 
   * DateTime this version was made
   */
  _issued?: Element
  /** 
   * Responsible Diagnostic Service
   */
  performer?: Array<Reference>;
  /** 
   * Primary result interpreter
   */
  resultsInterpreter?: Array<Reference>;
  /** 
   * Specimens this report is based on
   */
  specimen?: Array<Reference>;
  /** 
   * Observations
   */
  result?: Array<Reference>;
  /** 
   * Reference to full details of imaging associated with the diagnostic report
   */
  imagingStudy?: Array<Reference>;
  /** 
   * Key images associated with this report
   */
  media?: Array<DiagnosticReportMedia>;
  /** 
   * Clinical conclusion (interpretation) of test results
   */
  conclusion?: string;
  /** 
   * Clinical conclusion (interpretation) of test results
   */
  _conclusion?: Element
  /** 
   * Codes for the clinical conclusion of test results
   */
  conclusionCode?: Array<CodeableConcept>;
  /** 
   * Entire report as issued
   */
  presentedForm?: Array<Attachment>;
}

export interface DocumentManifestRelated {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Identifiers of things that are related
   */
  identifier?: Identifier;
  /** 
   * Related Resource
   */
  ref?: Reference;
}
export interface DocumentManifest {
resourceType: "DocumentManifest"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Unique Identifier for the set of documents
   */
  masterIdentifier?: Identifier;
  /** 
   * Other identifiers for the manifest
   */
  identifier?: Array<Identifier>;
  /** 
   * current | superseded | entered-in-error
   */
  status: code;
  /** 
   * current | superseded | entered-in-error
   */
  _status?: Element
  /** 
   * Kind of document set
   */
  type?: CodeableConcept;
  /** 
   * The subject of the set of documents
   */
  subject?: Reference;
  /** 
   * When this document manifest created
   */
  created?: dateTime;
  /** 
   * When this document manifest created
   */
  _created?: Element
  /** 
   * Who and/or what authored the DocumentManifest
   */
  author?: Array<Reference>;
  /** 
   * Intended to get notified about this set of documents
   */
  recipient?: Array<Reference>;
  /** 
   * The source system/application/software
   */
  source?: uri;
  /** 
   * The source system/application/software
   */
  _source?: Element
  /** 
   * Human-readable description (title)
   */
  description?: string;
  /** 
   * Human-readable description (title)
   */
  _description?: Element
  /** 
   * Items in manifest
   */
  content: Array<Reference>;
  /** 
   * Related things
   */
  related?: Array<DocumentManifestRelated>;
}

export interface DocumentReferenceRelatesTo {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * replaces | transforms | signs | appends
   */
  code: code;
  /** 
   * replaces | transforms | signs | appends
   */
  _code?: Element
  /** 
   * Target of the relationship
   */
  target: Reference;
}
export interface DocumentReferenceContent {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Where to access the document
   */
  attachment: Attachment;
  /** 
   * Format/content rules for the document
   */
  format?: Coding;
}
export interface DocumentReferenceContext {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Context of the document  content
   */
  encounter?: Array<Reference>;
  /** 
   * Main clinical acts documented
   */
  event?: Array<CodeableConcept>;
  /** 
   * Time of service that is being documented
   */
  period?: Period;
  /** 
   * Kind of facility where patient was seen
   */
  facilityType?: CodeableConcept;
  /** 
   * Additional details about where the content was created (e.g. clinical specialty)
   */
  practiceSetting?: CodeableConcept;
  /** 
   * Patient demographics from source
   */
  sourcePatientInfo?: Reference;
  /** 
   * Related identifiers or resources
   */
  related?: Array<Reference>;
}
export interface DocumentReference {
resourceType: "DocumentReference"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Master Version Specific Identifier
   */
  masterIdentifier?: Identifier;
  /** 
   * Other identifiers for the document
   */
  identifier?: Array<Identifier>;
  /** 
   * current | superseded | entered-in-error
   */
  status: code;
  /** 
   * current | superseded | entered-in-error
   */
  _status?: Element
  /** 
   * preliminary | final | amended | entered-in-error
   */
  docStatus?: code;
  /** 
   * preliminary | final | amended | entered-in-error
   */
  _docStatus?: Element
  /** 
   * Kind of document (LOINC if possible)
   */
  type?: CodeableConcept;
  /** 
   * Categorization of document
   */
  category?: Array<CodeableConcept>;
  /** 
   * Who/what is the subject of the document
   */
  subject?: Reference;
  /** 
   * When this document reference was created
   */
  date?: instant;
  /** 
   * When this document reference was created
   */
  _date?: Element
  /** 
   * Who and/or what authored the document
   */
  author?: Array<Reference>;
  /** 
   * Who/what authenticated the document
   */
  authenticator?: Reference;
  /** 
   * Organization which maintains the document
   */
  custodian?: Reference;
  /** 
   * Relationships to other documents
   */
  relatesTo?: Array<DocumentReferenceRelatesTo>;
  /** 
   * Human-readable description
   */
  description?: string;
  /** 
   * Human-readable description
   */
  _description?: Element
  /** 
   * Document security-tags
   */
  securityLabel?: Array<CodeableConcept>;
  /** 
   * Document referenced
   */
  content: Array<DocumentReferenceContent>;
  /** 
   * Clinical context of document
   */
  context?: DocumentReferenceContext;
}

export interface EncounterStatusHistory {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * planned | arrived | triaged | in-progress | onleave | finished | cancelled +
   */
  status: code;
  /** 
   * planned | arrived | triaged | in-progress | onleave | finished | cancelled +
   */
  _status?: Element
  /** 
   * The time that the episode was in the specified status
   */
  period: Period;
}
export interface EncounterClassHistory {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * inpatient | outpatient | ambulatory | emergency +
   */
  class: Coding;
  /** 
   * The time that the episode was in the specified class
   */
  period: Period;
}
export interface EncounterParticipant {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Role of participant in encounter
   */
  type?: Array<CodeableConcept>;
  /** 
   * Period of time during the encounter that the participant participated
   */
  period?: Period;
  /** 
   * Persons involved in the encounter other than the patient
   */
  individual?: Reference;
}
export interface EncounterDiagnosis {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The diagnosis or procedure relevant to the encounter
   */
  condition: Reference;
  /** 
   * Role that this diagnosis has within the encounter (e.g. admission, billing, discharge …)
   */
  use?: CodeableConcept;
  /** 
   * Ranking of the diagnosis (for each role type)
   */
  rank?: positiveInt;
  /** 
   * Ranking of the diagnosis (for each role type)
   */
  _rank?: Element
}
export interface EncounterHospitalization {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Pre-admission identifier
   */
  preAdmissionIdentifier?: Identifier;
  /** 
   * The location/organization from which the patient came before admission
   */
  origin?: Reference;
  /** 
   * From where patient was admitted (physician referral, transfer)
   */
  admitSource?: CodeableConcept;
  /** 
   * The type of hospital re-admission that has occurred (if any). If the value is absent, then this is not identified as a readmission
   */
  reAdmission?: CodeableConcept;
  /** 
   * Diet preferences reported by the patient
   */
  dietPreference?: Array<CodeableConcept>;
  /** 
   * Special courtesies (VIP, board member)
   */
  specialCourtesy?: Array<CodeableConcept>;
  /** 
   * Wheelchair, translator, stretcher, etc.
   */
  specialArrangement?: Array<CodeableConcept>;
  /** 
   * Location/organization to which the patient is discharged
   */
  destination?: Reference;
  /** 
   * Category or kind of location after discharge
   */
  dischargeDisposition?: CodeableConcept;
}
export interface EncounterLocation {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Location the encounter takes place
   */
  location: Reference;
  /** 
   * planned | active | reserved | completed
   */
  status?: code;
  /** 
   * planned | active | reserved | completed
   */
  _status?: Element
  /** 
   * The physical type of the location (usually the level in the location hierachy - bed room ward etc.)
   */
  physicalType?: CodeableConcept;
  /** 
   * Time period during which the patient was present at the location
   */
  period?: Period;
}
export interface Encounter {
resourceType: "Encounter"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Identifier(s) by which this encounter is known
   */
  identifier?: Array<Identifier>;
  /** 
   * planned | arrived | triaged | in-progress | onleave | finished | cancelled +
   */
  status: code;
  /** 
   * planned | arrived | triaged | in-progress | onleave | finished | cancelled +
   */
  _status?: Element
  /** 
   * List of past encounter statuses
   */
  statusHistory?: Array<EncounterStatusHistory>;
  /** 
   * Classification of patient encounter
   */
  class: Coding;
  /** 
   * List of past encounter classes
   */
  classHistory?: Array<EncounterClassHistory>;
  /** 
   * Specific type of encounter
   */
  type?: Array<CodeableConcept>;
  /** 
   * Specific type of service
   */
  serviceType?: CodeableConcept;
  /** 
   * Indicates the urgency of the encounter
   */
  priority?: CodeableConcept;
  /** 
   * The patient or group present at the encounter
   */
  subject?: Reference;
  /** 
   * Episode(s) of care that this encounter should be recorded against
   */
  episodeOfCare?: Array<Reference>;
  /** 
   * The ServiceRequest that initiated this encounter
   */
  basedOn?: Array<Reference>;
  /** 
   * List of participants involved in the encounter
   */
  participant?: Array<EncounterParticipant>;
  /** 
   * The appointment that scheduled this encounter
   */
  appointment?: Array<Reference>;
  /** 
   * The start and end time of the encounter
   */
  period?: Period;
  /** 
   * Quantity of time the encounter lasted (less time absent)
   */
  length?: Duration;
  /** 
   * Coded reason the encounter takes place
   */
  reasonCode?: Array<CodeableConcept>;
  /** 
   * Reason the encounter takes place (reference)
   */
  reasonReference?: Array<Reference>;
  /** 
   * The list of diagnosis relevant to this encounter
   */
  diagnosis?: Array<EncounterDiagnosis>;
  /** 
   * The set of accounts that may be used for billing for this Encounter
   */
  account?: Array<Reference>;
  /** 
   * Details about the admission to a healthcare service
   */
  hospitalization?: EncounterHospitalization;
  /** 
   * List of locations where the patient has been
   */
  location?: Array<EncounterLocation>;
  /** 
   * The organization (facility) responsible for this encounter
   */
  serviceProvider?: Reference;
  /** 
   * Another Encounter this encounter is part of
   */
  partOf?: Reference;
}

export interface Endpoint {
resourceType: "Endpoint"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Identifies this endpoint across multiple systems
   */
  identifier?: Array<Identifier>;
  /** 
   * active | suspended | error | off | entered-in-error | test
   */
  status: code;
  /** 
   * active | suspended | error | off | entered-in-error | test
   */
  _status?: Element
  /** 
   * Protocol/Profile/Standard to be used with this endpoint connection
   */
  connectionType: Coding;
  /** 
   * A name that this endpoint can be identified by
   */
  name?: string;
  /** 
   * A name that this endpoint can be identified by
   */
  _name?: Element
  /** 
   * Organization that manages this endpoint (might not be the organization that exposes the endpoint)
   */
  managingOrganization?: Reference;
  /** 
   * Contact details for source (e.g. troubleshooting)
   */
  contact?: Array<ContactPoint>;
  /** 
   * Interval the endpoint is expected to be operational
   */
  period?: Period;
  /** 
   * The type of content that may be used at this endpoint (e.g. XDS Discharge summaries)
   */
  payloadType: Array<CodeableConcept>;
  /** 
   * Mimetype to send. If not specified, the content could be anything (including no payload, if the connectionType defined this)
   */
  payloadMimeType?: Array<code>;
  /** 
   * Mimetype to send. If not specified, the content could be anything (including no payload, if the connectionType defined this)
   */
  _payloadMimeType?: Array<Element>
  /** 
   * The technical base address for connecting to this endpoint
   */
  address: url;
  /** 
   * The technical base address for connecting to this endpoint
   */
  _address?: Element
  /** 
   * Usage depends on the channel type
   */
  header?: Array<string>;
  /** 
   * Usage depends on the channel type
   */
  _header?: Array<Element>
}

export interface EnrollmentRequest {
resourceType: "EnrollmentRequest"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business Identifier
   */
  identifier?: Array<Identifier>;
  /** 
   * active | cancelled | draft | entered-in-error
   */
  status?: code;
  /** 
   * active | cancelled | draft | entered-in-error
   */
  _status?: Element
  /** 
   * Creation date
   */
  created?: dateTime;
  /** 
   * Creation date
   */
  _created?: Element
  /** 
   * Target
   */
  insurer?: Reference;
  /** 
   * Responsible practitioner
   */
  provider?: Reference;
  /** 
   * The subject to be enrolled
   */
  candidate?: Reference;
  /** 
   * Insurance information
   */
  coverage?: Reference;
}

export interface EnrollmentResponse {
resourceType: "EnrollmentResponse"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business Identifier
   */
  identifier?: Array<Identifier>;
  /** 
   * active | cancelled | draft | entered-in-error
   */
  status?: code;
  /** 
   * active | cancelled | draft | entered-in-error
   */
  _status?: Element
  /** 
   * Claim reference
   */
  request?: Reference;
  /** 
   * queued | complete | error | partial
   */
  outcome?: code;
  /** 
   * queued | complete | error | partial
   */
  _outcome?: Element
  /** 
   * Disposition Message
   */
  disposition?: string;
  /** 
   * Disposition Message
   */
  _disposition?: Element
  /** 
   * Creation date
   */
  created?: dateTime;
  /** 
   * Creation date
   */
  _created?: Element
  /** 
   * Insurer
   */
  organization?: Reference;
  /** 
   * Responsible practitioner
   */
  requestProvider?: Reference;
}

export interface EpisodeOfCareStatusHistory {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * planned | waitlist | active | onhold | finished | cancelled | entered-in-error
   */
  status: code;
  /** 
   * planned | waitlist | active | onhold | finished | cancelled | entered-in-error
   */
  _status?: Element
  /** 
   * Duration the EpisodeOfCare was in the specified status
   */
  period: Period;
}
export interface EpisodeOfCareDiagnosis {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Conditions/problems/diagnoses this episode of care is for
   */
  condition: Reference;
  /** 
   * Role that this diagnosis has within the episode of care (e.g. admission, billing, discharge …)
   */
  role?: CodeableConcept;
  /** 
   * Ranking of the diagnosis (for each role type)
   */
  rank?: positiveInt;
  /** 
   * Ranking of the diagnosis (for each role type)
   */
  _rank?: Element
}
export interface EpisodeOfCare {
resourceType: "EpisodeOfCare"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business Identifier(s) relevant for this EpisodeOfCare
   */
  identifier?: Array<Identifier>;
  /** 
   * planned | waitlist | active | onhold | finished | cancelled | entered-in-error
   */
  status: code;
  /** 
   * planned | waitlist | active | onhold | finished | cancelled | entered-in-error
   */
  _status?: Element
  /** 
   * Past list of status codes (the current status may be included to cover the start date of the status)
   */
  statusHistory?: Array<EpisodeOfCareStatusHistory>;
  /** 
   * Type/class  - e.g. specialist referral, disease management
   */
  type?: Array<CodeableConcept>;
  /** 
   * The list of diagnosis relevant to this episode of care
   */
  diagnosis?: Array<EpisodeOfCareDiagnosis>;
  /** 
   * The patient who is the focus of this episode of care
   */
  patient: Reference;
  /** 
   * Organization that assumes care
   */
  managingOrganization?: Reference;
  /** 
   * Interval during responsibility is assumed
   */
  period?: Period;
  /** 
   * Originating Referral Request(s)
   */
  referralRequest?: Array<Reference>;
  /** 
   * Care manager/care coordinator for the patient
   */
  careManager?: Reference;
  /** 
   * Other practitioners facilitating this episode of care
   */
  team?: Array<Reference>;
  /** 
   * The set of accounts that may be used for billing for this EpisodeOfCare
   */
  account?: Array<Reference>;
}

export interface EventDefinition {
resourceType: "EventDefinition"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Canonical identifier for this event definition, represented as a URI (globally unique)
   */
  url?: uri;
  /** 
   * Canonical identifier for this event definition, represented as a URI (globally unique)
   */
  _url?: Element
  /** 
   * Additional identifier for the event definition
   */
  identifier?: Array<Identifier>;
  /** 
   * Business version of the event definition
   */
  version?: string;
  /** 
   * Business version of the event definition
   */
  _version?: Element
  /** 
   * Name for this event definition (computer friendly)
   */
  name?: string;
  /** 
   * Name for this event definition (computer friendly)
   */
  _name?: Element
  /** 
   * Name for this event definition (human friendly)
   */
  title?: string;
  /** 
   * Name for this event definition (human friendly)
   */
  _title?: Element
  /** 
   * Subordinate title of the event definition
   */
  subtitle?: string;
  /** 
   * Subordinate title of the event definition
   */
  _subtitle?: Element
  /** 
   * draft | active | retired | unknown
   */
  status: code;
  /** 
   * draft | active | retired | unknown
   */
  _status?: Element
  /** 
   * For testing purposes, not real usage
   */
  experimental?: boolean;
  /** 
   * For testing purposes, not real usage
   */
  _experimental?: Element
  /** 
   * Type of individual the event definition is focused on
   */
  subjectCodeableConcept?: CodeableConcept;
  /** 
   * Type of individual the event definition is focused on
   */
  subjectReference?: Reference;
  /** 
   * Date last changed
   */
  date?: dateTime;
  /** 
   * Date last changed
   */
  _date?: Element
  /** 
   * Name of the publisher (organization or individual)
   */
  publisher?: string;
  /** 
   * Name of the publisher (organization or individual)
   */
  _publisher?: Element
  /** 
   * Contact details for the publisher
   */
  contact?: Array<ContactDetail>;
  /** 
   * Natural language description of the event definition
   */
  description?: markdown;
  /** 
   * Natural language description of the event definition
   */
  _description?: Element
  /** 
   * The context that the content is intended to support
   */
  useContext?: Array<UsageContext>;
  /** 
   * Intended jurisdiction for event definition (if applicable)
   */
  jurisdiction?: Array<CodeableConcept>;
  /** 
   * Why this event definition is defined
   */
  purpose?: markdown;
  /** 
   * Why this event definition is defined
   */
  _purpose?: Element
  /** 
   * Describes the clinical usage of the event definition
   */
  usage?: string;
  /** 
   * Describes the clinical usage of the event definition
   */
  _usage?: Element
  /** 
   * Use and/or publishing restrictions
   */
  copyright?: markdown;
  /** 
   * Use and/or publishing restrictions
   */
  _copyright?: Element
  /** 
   * When the event definition was approved by publisher
   */
  approvalDate?: date;
  /** 
   * When the event definition was approved by publisher
   */
  _approvalDate?: Element
  /** 
   * When the event definition was last reviewed
   */
  lastReviewDate?: date;
  /** 
   * When the event definition was last reviewed
   */
  _lastReviewDate?: Element
  /** 
   * When the event definition is expected to be used
   */
  effectivePeriod?: Period;
  /** 
   * E.g. Education, Treatment, Assessment, etc.
   */
  topic?: Array<CodeableConcept>;
  /** 
   * Who authored the content
   */
  author?: Array<ContactDetail>;
  /** 
   * Who edited the content
   */
  editor?: Array<ContactDetail>;
  /** 
   * Who reviewed the content
   */
  reviewer?: Array<ContactDetail>;
  /** 
   * Who endorsed the content
   */
  endorser?: Array<ContactDetail>;
  /** 
   * Additional documentation, citations, etc.
   */
  relatedArtifact?: Array<RelatedArtifact>;
  /** 
   * "when" the event occurs (multiple = 'or')
   */
  trigger: Array<TriggerDefinition>;
}

export interface EvidenceVariableDefinition {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * A text description or summary of the variable
   */
  description?: markdown;
  /** 
   * A text description or summary of the variable
   */
  _description?: Element
  /** 
   * Footnotes and/or explanatory notes
   */
  note?: Array<Annotation>;
  /** 
   * population | subpopulation | exposure | referenceExposure | measuredVariable | confounder
   */
  variableRole: CodeableConcept;
  /** 
   * Definition of the actual variable related to the statistic(s)
   */
  observed?: Reference;
  /** 
   * Definition of the intended variable related to the Evidence
   */
  intended?: Reference;
  /** 
   * low | moderate | high | exact
   */
  directnessMatch?: CodeableConcept;
}
export interface EvidenceStatisticSampleSize {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Textual description of sample size for statistic
   */
  description?: string;
  /** 
   * Textual description of sample size for statistic
   */
  _description?: Element
  /** 
   * Footnote or explanatory note about the sample size
   */
  note?: Array<Annotation>;
  /** 
   * Number of contributing studies
   */
  numberOfStudies?: unsignedInt;
  /** 
   * Number of contributing studies
   */
  _numberOfStudies?: Element
  /** 
   * Cumulative number of participants
   */
  numberOfParticipants?: unsignedInt;
  /** 
   * Cumulative number of participants
   */
  _numberOfParticipants?: Element
  /** 
   * Number of participants with known results for measured variables
   */
  knownDataCount?: unsignedInt;
  /** 
   * Number of participants with known results for measured variables
   */
  _knownDataCount?: Element
}
export interface EvidenceStatisticAttributeEstimate {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Textual description of the attribute estimate
   */
  description?: string;
  /** 
   * Textual description of the attribute estimate
   */
  _description?: Element
  /** 
   * Footnote or explanatory note about the estimate
   */
  note?: Array<Annotation>;
  /** 
   * The type of attribute estimate, eg confidence interval or p value
   */
  type?: CodeableConcept;
  /** 
   * The singular quantity of the attribute estimate, for attribute estimates represented as single values; also used to report unit of measure
   */
  quantity?: Quantity;
  /** 
   * Level of confidence interval, eg 0.95 for 95% confidence interval
   */
  level?: decimal;
  /** 
   * Level of confidence interval, eg 0.95 for 95% confidence interval
   */
  _level?: Element
  /** 
   * Lower and upper bound values of the attribute estimate
   */
  range?: Range;
  /** 
   * A nested attribute estimate; which is the attribute estimate of an attribute estimate
   */
  attributeEstimate?: Array<EvidenceStatisticAttributeEstimate>;
}
export interface EvidenceStatisticModelCharacteristicVariable {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Description of the variable
   */
  variableDefinition: Reference;
  /** 
   * continuous | dichotomous | ordinal | polychotomous
   */
  handling?: code;
  /** 
   * continuous | dichotomous | ordinal | polychotomous
   */
  _handling?: Element
  /** 
   * Description for grouping of ordinal or polychotomous variables
   */
  valueCategory?: Array<CodeableConcept>;
  /** 
   * Discrete value for grouping of ordinal or polychotomous variables
   */
  valueQuantity?: Array<Quantity>;
  /** 
   * Range of values for grouping of ordinal or polychotomous variables
   */
  valueRange?: Array<Range>;
}
export interface EvidenceStatisticModelCharacteristic {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Model specification
   */
  code: CodeableConcept;
  /** 
   * Numerical value to complete model specification
   */
  value?: Quantity;
  /** 
   * A variable adjusted for in the adjusted analysis
   */
  variable?: Array<EvidenceStatisticModelCharacteristicVariable>;
  /** 
   * An attribute of the statistic used as a model characteristic
   */
  attributeEstimate?: Array<EvidenceStatisticAttributeEstimate>;
}
export interface EvidenceStatistic {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Description of content
   */
  description?: string;
  /** 
   * Description of content
   */
  _description?: Element
  /** 
   * Footnotes and/or explanatory notes
   */
  note?: Array<Annotation>;
  /** 
   * Type of statistic, eg relative risk
   */
  statisticType?: CodeableConcept;
  /** 
   * Associated category for categorical variable
   */
  category?: CodeableConcept;
  /** 
   * Statistic value
   */
  quantity?: Quantity;
  /** 
   * The number of events associated with the statistic
   */
  numberOfEvents?: unsignedInt;
  /** 
   * The number of events associated with the statistic
   */
  _numberOfEvents?: Element
  /** 
   * The number of participants affected
   */
  numberAffected?: unsignedInt;
  /** 
   * The number of participants affected
   */
  _numberAffected?: Element
  /** 
   * Number of samples in the statistic
   */
  sampleSize?: EvidenceStatisticSampleSize;
  /** 
   * An attribute of the Statistic
   */
  attributeEstimate?: Array<EvidenceStatisticAttributeEstimate>;
  /** 
   * An aspect of the statistical model
   */
  modelCharacteristic?: Array<EvidenceStatisticModelCharacteristic>;
}
export interface EvidenceCertainty {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Textual description of certainty
   */
  description?: string;
  /** 
   * Textual description of certainty
   */
  _description?: Element
  /** 
   * Footnotes and/or explanatory notes
   */
  note?: Array<Annotation>;
  /** 
   * Aspect of certainty being rated
   */
  type?: CodeableConcept;
  /** 
   * Assessment or judgement of the aspect
   */
  rating?: CodeableConcept;
  /** 
   * Individual or group who did the rating
   */
  rater?: string;
  /** 
   * Individual or group who did the rating
   */
  _rater?: Element
  /** 
   * A domain or subdomain of certainty
   */
  subcomponent?: Array<EvidenceCertainty>;
}
export interface Evidence {
resourceType: "Evidence"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Canonical identifier for this evidence, represented as a globally unique URI
   */
  url?: uri;
  /** 
   * Canonical identifier for this evidence, represented as a globally unique URI
   */
  _url?: Element
  /** 
   * Additional identifier for the summary
   */
  identifier?: Array<Identifier>;
  /** 
   * Business version of this summary
   */
  version?: string;
  /** 
   * Business version of this summary
   */
  _version?: Element
  /** 
   * Name for this summary (human friendly)
   */
  title?: string;
  /** 
   * Name for this summary (human friendly)
   */
  _title?: Element
  /** 
   * Citation for this evidence
   */
  citeAsReference?: Reference;
  /** 
   * Citation for this evidence
   */
  citeAsMarkdown?: markdown;
  /** 
   * Citation for this evidence
   */
  _citeAsMarkdown?: Element
  /** 
   * draft | active | retired | unknown
   */
  status: code;
  /** 
   * draft | active | retired | unknown
   */
  _status?: Element
  /** 
   * Date last changed
   */
  date?: dateTime;
  /** 
   * Date last changed
   */
  _date?: Element
  /** 
   * The context that the content is intended to support
   */
  useContext?: Array<UsageContext>;
  /** 
   * When the summary was approved by publisher
   */
  approvalDate?: date;
  /** 
   * When the summary was approved by publisher
   */
  _approvalDate?: Element
  /** 
   * When the summary was last reviewed
   */
  lastReviewDate?: date;
  /** 
   * When the summary was last reviewed
   */
  _lastReviewDate?: Element
  /** 
   * Name of the publisher (organization or individual)
   */
  publisher?: string;
  /** 
   * Name of the publisher (organization or individual)
   */
  _publisher?: Element
  /** 
   * Contact details for the publisher
   */
  contact?: Array<ContactDetail>;
  /** 
   * Who authored the content
   */
  author?: Array<ContactDetail>;
  /** 
   * Who edited the content
   */
  editor?: Array<ContactDetail>;
  /** 
   * Who reviewed the content
   */
  reviewer?: Array<ContactDetail>;
  /** 
   * Who endorsed the content
   */
  endorser?: Array<ContactDetail>;
  /** 
   * Link or citation to artifact associated with the summary
   */
  relatedArtifact?: Array<RelatedArtifact>;
  /** 
   * Description of the particular summary
   */
  description?: markdown;
  /** 
   * Description of the particular summary
   */
  _description?: Element
  /** 
   * Declarative description of the Evidence
   */
  assertion?: markdown;
  /** 
   * Declarative description of the Evidence
   */
  _assertion?: Element
  /** 
   * Footnotes and/or explanatory notes
   */
  note?: Array<Annotation>;
  /** 
   * Evidence variable such as population, exposure, or outcome
   */
  variableDefinition: Array<EvidenceVariableDefinition>;
  /** 
   * The method to combine studies
   */
  synthesisType?: CodeableConcept;
  /** 
   * The type of study that produced this evidence
   */
  studyType?: CodeableConcept;
  /** 
   * Values and parameters for a single statistic
   */
  statistic?: Array<EvidenceStatistic>;
  /** 
   * Certainty or quality of the evidence
   */
  certainty?: Array<EvidenceCertainty>;
}

export interface EvidenceReportSubjectCharacteristic {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Characteristic code
   */
  code: CodeableConcept;
  /** 
   * Characteristic value
   */
  valueReference?: Reference;
  /** 
   * Characteristic value
   */
  valueCodeableConcept?: CodeableConcept;
  /** 
   * Characteristic value
   */
  valueBoolean?: boolean;
  /** 
   * Characteristic value
   */
  _valueBoolean?: Element
  /** 
   * Characteristic value
   */
  valueQuantity?: Quantity;
  /** 
   * Characteristic value
   */
  valueRange?: Range;
  /** 
   * Is used to express not the characteristic
   */
  exclude?: boolean;
  /** 
   * Is used to express not the characteristic
   */
  _exclude?: Element
  /** 
   * Timeframe for the characteristic
   */
  period?: Period;
}
export interface EvidenceReportSubject {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Characteristic
   */
  characteristic?: Array<EvidenceReportSubjectCharacteristic>;
  /** 
   * Footnotes and/or explanatory notes
   */
  note?: Array<Annotation>;
}
export interface EvidenceReportRelatesTo {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * replaces | amends | appends | transforms | replacedWith | amendedWith | appendedWith | transformedWith
   */
  code: code;
  /** 
   * replaces | amends | appends | transforms | replacedWith | amendedWith | appendedWith | transformedWith
   */
  _code?: Element
  /** 
   * Target of the relationship
   */
  targetIdentifier?: Identifier;
  /** 
   * Target of the relationship
   */
  targetReference?: Reference;
}
export interface EvidenceReportSection {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Label for section (e.g. for ToC)
   */
  title?: string;
  /** 
   * Label for section (e.g. for ToC)
   */
  _title?: Element
  /** 
   * Classification of section (recommended)
   */
  focus?: CodeableConcept;
  /** 
   * Classification of section by Resource
   */
  focusReference?: Reference;
  /** 
   * Who and/or what authored the section
   */
  author?: Array<Reference>;
  /** 
   * Text summary of the section, for human interpretation
   */
  text?: Narrative;
  /** 
   * working | snapshot | changes
   */
  mode?: code;
  /** 
   * working | snapshot | changes
   */
  _mode?: Element
  /** 
   * Order of section entries
   */
  orderedBy?: CodeableConcept;
  /** 
   * Extensible classifiers as content
   */
  entryClassifier?: Array<CodeableConcept>;
  /** 
   * Reference to resources as content
   */
  entryReference?: Array<Reference>;
  /** 
   * Quantity as content
   */
  entryQuantity?: Array<Quantity>;
  /** 
   * Why the section is empty
   */
  emptyReason?: CodeableConcept;
  /** 
   * Nested Section
   */
  section?: Array<EvidenceReportSection>;
}
export interface EvidenceReport {
resourceType: "EvidenceReport"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Canonical identifier for this EvidenceReport, represented as a globally unique URI
   */
  url?: uri;
  /** 
   * Canonical identifier for this EvidenceReport, represented as a globally unique URI
   */
  _url?: Element
  /** 
   * draft | active | retired | unknown
   */
  status: code;
  /** 
   * draft | active | retired | unknown
   */
  _status?: Element
  /** 
   * The context that the content is intended to support
   */
  useContext?: Array<UsageContext>;
  /** 
   * Unique identifier for the evidence report
   */
  identifier?: Array<Identifier>;
  /** 
   * Identifiers for articles that may relate to more than one evidence report
   */
  relatedIdentifier?: Array<Identifier>;
  /** 
   * Citation for this report
   */
  citeAsReference?: Reference;
  /** 
   * Citation for this report
   */
  citeAsMarkdown?: markdown;
  /** 
   * Citation for this report
   */
  _citeAsMarkdown?: Element
  /** 
   * Kind of report
   */
  type?: CodeableConcept;
  /** 
   * Used for footnotes and annotations
   */
  note?: Array<Annotation>;
  /** 
   * Link, description or reference to artifact associated with the report
   */
  relatedArtifact?: Array<RelatedArtifact>;
  /** 
   * Focus of the report
   */
  subject: EvidenceReportSubject;
  /** 
   * Name of the publisher (organization or individual)
   */
  publisher?: string;
  /** 
   * Name of the publisher (organization or individual)
   */
  _publisher?: Element
  /** 
   * Contact details for the publisher
   */
  contact?: Array<ContactDetail>;
  /** 
   * Who authored the content
   */
  author?: Array<ContactDetail>;
  /** 
   * Who edited the content
   */
  editor?: Array<ContactDetail>;
  /** 
   * Who reviewed the content
   */
  reviewer?: Array<ContactDetail>;
  /** 
   * Who endorsed the content
   */
  endorser?: Array<ContactDetail>;
  /** 
   * Relationships to other compositions/documents
   */
  relatesTo?: Array<EvidenceReportRelatesTo>;
  /** 
   * Composition is broken into sections
   */
  section?: Array<EvidenceReportSection>;
}

export interface EvidenceVariableCharacteristicTimeFromStart {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Human readable description
   */
  description?: string;
  /** 
   * Human readable description
   */
  _description?: Element
  /** 
   * Used to express the observation at a defined amount of time after the study start
   */
  quantity?: Quantity;
  /** 
   * Used to express the observation within a period after the study start
   */
  range?: Range;
  /** 
   * Used for footnotes or explanatory notes
   */
  note?: Array<Annotation>;
}
export interface EvidenceVariableCharacteristic {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Natural language description of the characteristic
   */
  description?: string;
  /** 
   * Natural language description of the characteristic
   */
  _description?: Element
  /** 
   * What code or expression defines members?
   */
  definitionReference?: Reference;
  /** 
   * What code or expression defines members?
   */
  definitionCanonical?: canonical;
  /** 
   * What code or expression defines members?
   */
  _definitionCanonical?: Element
  /** 
   * What code or expression defines members?
   */
  definitionCodeableConcept?: CodeableConcept;
  /** 
   * What code or expression defines members?
   */
  definitionExpression?: Expression;
  /** 
   * Method used for describing characteristic
   */
  method?: CodeableConcept;
  /** 
   * Device used for determining characteristic
   */
  device?: Reference;
  /** 
   * Whether the characteristic includes or excludes members
   */
  exclude?: boolean;
  /** 
   * Whether the characteristic includes or excludes members
   */
  _exclude?: Element
  /** 
   * Observation time from study start
   */
  timeFromStart?: EvidenceVariableCharacteristicTimeFromStart;
  /** 
   * mean | median | mean-of-mean | mean-of-median | median-of-mean | median-of-median
   */
  groupMeasure?: code;
  /** 
   * mean | median | mean-of-mean | mean-of-median | median-of-mean | median-of-median
   */
  _groupMeasure?: Element
}
export interface EvidenceVariableCategory {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Description of the grouping
   */
  name?: string;
  /** 
   * Description of the grouping
   */
  _name?: Element
  /** 
   * Definition of the grouping
   */
  valueCodeableConcept?: CodeableConcept;
  /** 
   * Definition of the grouping
   */
  valueQuantity?: Quantity;
  /** 
   * Definition of the grouping
   */
  valueRange?: Range;
}
export interface EvidenceVariable {
resourceType: "EvidenceVariable"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Canonical identifier for this evidence variable, represented as a URI (globally unique)
   */
  url?: uri;
  /** 
   * Canonical identifier for this evidence variable, represented as a URI (globally unique)
   */
  _url?: Element
  /** 
   * Additional identifier for the evidence variable
   */
  identifier?: Array<Identifier>;
  /** 
   * Business version of the evidence variable
   */
  version?: string;
  /** 
   * Business version of the evidence variable
   */
  _version?: Element
  /** 
   * Name for this evidence variable (computer friendly)
   */
  name?: string;
  /** 
   * Name for this evidence variable (computer friendly)
   */
  _name?: Element
  /** 
   * Name for this evidence variable (human friendly)
   */
  title?: string;
  /** 
   * Name for this evidence variable (human friendly)
   */
  _title?: Element
  /** 
   * Title for use in informal contexts
   */
  shortTitle?: string;
  /** 
   * Title for use in informal contexts
   */
  _shortTitle?: Element
  /** 
   * Subordinate title of the EvidenceVariable
   */
  subtitle?: string;
  /** 
   * Subordinate title of the EvidenceVariable
   */
  _subtitle?: Element
  /** 
   * draft | active | retired | unknown
   */
  status: code;
  /** 
   * draft | active | retired | unknown
   */
  _status?: Element
  /** 
   * Date last changed
   */
  date?: dateTime;
  /** 
   * Date last changed
   */
  _date?: Element
  /** 
   * Natural language description of the evidence variable
   */
  description?: markdown;
  /** 
   * Natural language description of the evidence variable
   */
  _description?: Element
  /** 
   * Used for footnotes or explanatory notes
   */
  note?: Array<Annotation>;
  /** 
   * The context that the content is intended to support
   */
  useContext?: Array<UsageContext>;
  /** 
   * Name of the publisher (organization or individual)
   */
  publisher?: string;
  /** 
   * Name of the publisher (organization or individual)
   */
  _publisher?: Element
  /** 
   * Contact details for the publisher
   */
  contact?: Array<ContactDetail>;
  /** 
   * Who authored the content
   */
  author?: Array<ContactDetail>;
  /** 
   * Who edited the content
   */
  editor?: Array<ContactDetail>;
  /** 
   * Who reviewed the content
   */
  reviewer?: Array<ContactDetail>;
  /** 
   * Who endorsed the content
   */
  endorser?: Array<ContactDetail>;
  /** 
   * Additional documentation, citations, etc.
   */
  relatedArtifact?: Array<RelatedArtifact>;
  /** 
   * Actual or conceptual
   */
  actual?: boolean;
  /** 
   * Actual or conceptual
   */
  _actual?: Element
  /** 
   * intersection | union
   */
  characteristicCombination?: code;
  /** 
   * intersection | union
   */
  _characteristicCombination?: Element
  /** 
   * What defines the members of the evidence element
   */
  characteristic?: Array<EvidenceVariableCharacteristic>;
  /** 
   * continuous | dichotomous | ordinal | polychotomous
   */
  handling?: code;
  /** 
   * continuous | dichotomous | ordinal | polychotomous
   */
  _handling?: Element
  /** 
   * A grouping for ordinal or polychotomous variables
   */
  category?: Array<EvidenceVariableCategory>;
}

export interface ExampleScenarioActor {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * ID or acronym of the actor
   */
  actorId: string;
  /** 
   * ID or acronym of the actor
   */
  _actorId?: Element
  /** 
   * person | entity
   */
  type: code;
  /** 
   * person | entity
   */
  _type?: Element
  /** 
   * The name of the actor as shown in the page
   */
  name?: string;
  /** 
   * The name of the actor as shown in the page
   */
  _name?: Element
  /** 
   * The description of the actor
   */
  description?: markdown;
  /** 
   * The description of the actor
   */
  _description?: Element
}
export interface ExampleScenarioInstanceVersion {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The identifier of a specific version of a resource
   */
  versionId: string;
  /** 
   * The identifier of a specific version of a resource
   */
  _versionId?: Element
  /** 
   * The description of the resource version
   */
  description: markdown;
  /** 
   * The description of the resource version
   */
  _description?: Element
}
export interface ExampleScenarioInstanceContainedInstance {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Each resource contained in the instance
   */
  resourceId: string;
  /** 
   * Each resource contained in the instance
   */
  _resourceId?: Element
  /** 
   * A specific version of a resource contained in the instance
   */
  versionId?: string;
  /** 
   * A specific version of a resource contained in the instance
   */
  _versionId?: Element
}
export interface ExampleScenarioInstance {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The id of the resource for referencing
   */
  resourceId: string;
  /** 
   * The id of the resource for referencing
   */
  _resourceId?: Element
  /** 
   * The type of the resource
   */
  resourceType: code;
  /** 
   * The type of the resource
   */
  _resourceType?: Element
  /** 
   * A short name for the resource instance
   */
  name?: string;
  /** 
   * A short name for the resource instance
   */
  _name?: Element
  /** 
   * Human-friendly description of the resource instance
   */
  description?: markdown;
  /** 
   * Human-friendly description of the resource instance
   */
  _description?: Element
  /** 
   * A specific version of the resource
   */
  version?: Array<ExampleScenarioInstanceVersion>;
  /** 
   * Resources contained in the instance
   */
  containedInstance?: Array<ExampleScenarioInstanceContainedInstance>;
}
export interface ExampleScenarioProcessStepOperation {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The sequential number of the interaction
   */
  number: string;
  /** 
   * The sequential number of the interaction
   */
  _number?: Element
  /** 
   * The type of operation - CRUD
   */
  type?: string;
  /** 
   * The type of operation - CRUD
   */
  _type?: Element
  /** 
   * The human-friendly name of the interaction
   */
  name?: string;
  /** 
   * The human-friendly name of the interaction
   */
  _name?: Element
  /** 
   * Who starts the transaction
   */
  initiator?: string;
  /** 
   * Who starts the transaction
   */
  _initiator?: Element
  /** 
   * Who receives the transaction
   */
  receiver?: string;
  /** 
   * Who receives the transaction
   */
  _receiver?: Element
  /** 
   * A comment to be inserted in the diagram
   */
  description?: markdown;
  /** 
   * A comment to be inserted in the diagram
   */
  _description?: Element
  /** 
   * Whether the initiator is deactivated right after the transaction
   */
  initiatorActive?: boolean;
  /** 
   * Whether the initiator is deactivated right after the transaction
   */
  _initiatorActive?: Element
  /** 
   * Whether the receiver is deactivated right after the transaction
   */
  receiverActive?: boolean;
  /** 
   * Whether the receiver is deactivated right after the transaction
   */
  _receiverActive?: Element
  /** 
   * Each resource instance used by the initiator
   */
  request?: ExampleScenarioInstanceContainedInstance;
  /** 
   * Each resource instance used by the responder
   */
  response?: ExampleScenarioInstanceContainedInstance;
}
export interface ExampleScenarioProcessStepAlternative {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Label for alternative
   */
  title: string;
  /** 
   * Label for alternative
   */
  _title?: Element
  /** 
   * A human-readable description of each option
   */
  description?: markdown;
  /** 
   * A human-readable description of each option
   */
  _description?: Element
  /** 
   * What happens in each alternative option
   */
  step?: Array<ExampleScenarioProcessStep>;
}
export interface ExampleScenarioProcessStep {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Nested process
   */
  process?: Array<ExampleScenarioProcess>;
  /** 
   * If there is a pause in the flow
   */
  pause?: boolean;
  /** 
   * If there is a pause in the flow
   */
  _pause?: Element
  /** 
   * Each interaction or action
   */
  operation?: ExampleScenarioProcessStepOperation;
  /** 
   * Alternate non-typical step action
   */
  alternative?: Array<ExampleScenarioProcessStepAlternative>;
}
export interface ExampleScenarioProcess {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The diagram title of the group of operations
   */
  title: string;
  /** 
   * The diagram title of the group of operations
   */
  _title?: Element
  /** 
   * A longer description of the group of operations
   */
  description?: markdown;
  /** 
   * A longer description of the group of operations
   */
  _description?: Element
  /** 
   * Description of initial status before the process starts
   */
  preConditions?: markdown;
  /** 
   * Description of initial status before the process starts
   */
  _preConditions?: Element
  /** 
   * Description of final status after the process ends
   */
  postConditions?: markdown;
  /** 
   * Description of final status after the process ends
   */
  _postConditions?: Element
  /** 
   * Each step of the process
   */
  step?: Array<ExampleScenarioProcessStep>;
}
export interface ExampleScenario {
resourceType: "ExampleScenario"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Canonical identifier for this example scenario, represented as a URI (globally unique)
   */
  url?: uri;
  /** 
   * Canonical identifier for this example scenario, represented as a URI (globally unique)
   */
  _url?: Element
  /** 
   * Additional identifier for the example scenario
   */
  identifier?: Array<Identifier>;
  /** 
   * Business version of the example scenario
   */
  version?: string;
  /** 
   * Business version of the example scenario
   */
  _version?: Element
  /** 
   * Name for this example scenario (computer friendly)
   */
  name?: string;
  /** 
   * Name for this example scenario (computer friendly)
   */
  _name?: Element
  /** 
   * draft | active | retired | unknown
   */
  status: code;
  /** 
   * draft | active | retired | unknown
   */
  _status?: Element
  /** 
   * For testing purposes, not real usage
   */
  experimental?: boolean;
  /** 
   * For testing purposes, not real usage
   */
  _experimental?: Element
  /** 
   * Date last changed
   */
  date?: dateTime;
  /** 
   * Date last changed
   */
  _date?: Element
  /** 
   * Name of the publisher (organization or individual)
   */
  publisher?: string;
  /** 
   * Name of the publisher (organization or individual)
   */
  _publisher?: Element
  /** 
   * Contact details for the publisher
   */
  contact?: Array<ContactDetail>;
  /** 
   * The context that the content is intended to support
   */
  useContext?: Array<UsageContext>;
  /** 
   * Intended jurisdiction for example scenario (if applicable)
   */
  jurisdiction?: Array<CodeableConcept>;
  /** 
   * Use and/or publishing restrictions
   */
  copyright?: markdown;
  /** 
   * Use and/or publishing restrictions
   */
  _copyright?: Element
  /** 
   * The purpose of the example, e.g. to illustrate a scenario
   */
  purpose?: markdown;
  /** 
   * The purpose of the example, e.g. to illustrate a scenario
   */
  _purpose?: Element
  /** 
   * Actor participating in the resource
   */
  actor?: Array<ExampleScenarioActor>;
  /** 
   * Each resource and each version that is present in the workflow
   */
  instance?: Array<ExampleScenarioInstance>;
  /** 
   * Each major process - a group of operations
   */
  process?: Array<ExampleScenarioProcess>;
  /** 
   * Another nested workflow
   */
  workflow?: Array<canonical>;
  /** 
   * Another nested workflow
   */
  _workflow?: Array<Element>
}

export interface ExplanationOfBenefitRelated {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Reference to the related claim
   */
  claim?: Reference;
  /** 
   * How the reference claim is related
   */
  relationship?: CodeableConcept;
  /** 
   * File or case reference
   */
  reference?: Identifier;
}
export interface ExplanationOfBenefitPayee {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Category of recipient
   */
  type?: CodeableConcept;
  /** 
   * Recipient reference
   */
  party?: Reference;
}
export interface ExplanationOfBenefitCareTeam {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Order of care team
   */
  sequence: positiveInt;
  /** 
   * Order of care team
   */
  _sequence?: Element
  /** 
   * Practitioner or organization
   */
  provider: Reference;
  /** 
   * Indicator of the lead practitioner
   */
  responsible?: boolean;
  /** 
   * Indicator of the lead practitioner
   */
  _responsible?: Element
  /** 
   * Function within the team
   */
  role?: CodeableConcept;
  /** 
   * Practitioner credential or specialization
   */
  qualification?: CodeableConcept;
}
export interface ExplanationOfBenefitSupportingInfo {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Information instance identifier
   */
  sequence: positiveInt;
  /** 
   * Information instance identifier
   */
  _sequence?: Element
  /** 
   * Classification of the supplied information
   */
  category: CodeableConcept;
  /** 
   * Type of information
   */
  code?: CodeableConcept;
  /** 
   * When it occurred
   */
  timingDate?: date;
  /** 
   * When it occurred
   */
  _timingDate?: Element
  /** 
   * When it occurred
   */
  timingPeriod?: Period;
  /** 
   * Data to be provided
   */
  valueBoolean?: boolean;
  /** 
   * Data to be provided
   */
  _valueBoolean?: Element
  /** 
   * Data to be provided
   */
  valueString?: string;
  /** 
   * Data to be provided
   */
  _valueString?: Element
  /** 
   * Data to be provided
   */
  valueQuantity?: Quantity;
  /** 
   * Data to be provided
   */
  valueAttachment?: Attachment;
  /** 
   * Data to be provided
   */
  valueReference?: Reference;
  /** 
   * Explanation for the information
   */
  reason?: Coding;
}
export interface ExplanationOfBenefitDiagnosis {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Diagnosis instance identifier
   */
  sequence: positiveInt;
  /** 
   * Diagnosis instance identifier
   */
  _sequence?: Element
  /** 
   * Nature of illness or problem
   */
  diagnosisCodeableConcept?: CodeableConcept;
  /** 
   * Nature of illness or problem
   */
  diagnosisReference?: Reference;
  /** 
   * Timing or nature of the diagnosis
   */
  type?: Array<CodeableConcept>;
  /** 
   * Present on admission
   */
  onAdmission?: CodeableConcept;
  /** 
   * Package billing code
   */
  packageCode?: CodeableConcept;
}
export interface ExplanationOfBenefitProcedure {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Procedure instance identifier
   */
  sequence: positiveInt;
  /** 
   * Procedure instance identifier
   */
  _sequence?: Element
  /** 
   * Category of Procedure
   */
  type?: Array<CodeableConcept>;
  /** 
   * When the procedure was performed
   */
  date?: dateTime;
  /** 
   * When the procedure was performed
   */
  _date?: Element
  /** 
   * Specific clinical procedure
   */
  procedureCodeableConcept?: CodeableConcept;
  /** 
   * Specific clinical procedure
   */
  procedureReference?: Reference;
  /** 
   * Unique device identifier
   */
  udi?: Array<Reference>;
}
export interface ExplanationOfBenefitInsurance {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Coverage to be used for adjudication
   */
  focal: boolean;
  /** 
   * Coverage to be used for adjudication
   */
  _focal?: Element
  /** 
   * Insurance information
   */
  coverage: Reference;
  /** 
   * Prior authorization reference number
   */
  preAuthRef?: Array<string>;
  /** 
   * Prior authorization reference number
   */
  _preAuthRef?: Array<Element>
}
export interface ExplanationOfBenefitAccident {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * When the incident occurred
   */
  date?: date;
  /** 
   * When the incident occurred
   */
  _date?: Element
  /** 
   * The nature of the accident
   */
  type?: CodeableConcept;
  /** 
   * Where the event occurred
   */
  locationAddress?: Address;
  /** 
   * Where the event occurred
   */
  locationReference?: Reference;
}
export interface ExplanationOfBenefitItemAdjudication {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Type of adjudication information
   */
  category: CodeableConcept;
  /** 
   * Explanation of adjudication outcome
   */
  reason?: CodeableConcept;
  /** 
   * Monetary amount
   */
  amount?: Money;
  /** 
   * Non-monitary value
   */
  value?: decimal;
  /** 
   * Non-monitary value
   */
  _value?: Element
}
export interface ExplanationOfBenefitItemDetailSubDetail {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Product or service provided
   */
  sequence: positiveInt;
  /** 
   * Product or service provided
   */
  _sequence?: Element
  /** 
   * Revenue or cost center code
   */
  revenue?: CodeableConcept;
  /** 
   * Benefit classification
   */
  category?: CodeableConcept;
  /** 
   * Billing, service, product, or drug code
   */
  productOrService: CodeableConcept;
  /** 
   * Service/Product billing modifiers
   */
  modifier?: Array<CodeableConcept>;
  /** 
   * Program the product or service is provided under
   */
  programCode?: Array<CodeableConcept>;
  /** 
   * Count of products or services
   */
  quantity?: Quantity;
  /** 
   * Fee, charge or cost per item
   */
  unitPrice?: Money;
  /** 
   * Price scaling factor
   */
  factor?: decimal;
  /** 
   * Price scaling factor
   */
  _factor?: Element
  /** 
   * Total item cost
   */
  net?: Money;
  /** 
   * Unique device identifier
   */
  udi?: Array<Reference>;
  /** 
   * Applicable note numbers
   */
  noteNumber?: Array<positiveInt>;
  /** 
   * Applicable note numbers
   */
  _noteNumber?: Array<Element>
  /** 
   * Subdetail level adjudication details
   */
  adjudication?: Array<ExplanationOfBenefitItemAdjudication>;
}
export interface ExplanationOfBenefitItemDetail {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Product or service provided
   */
  sequence: positiveInt;
  /** 
   * Product or service provided
   */
  _sequence?: Element
  /** 
   * Revenue or cost center code
   */
  revenue?: CodeableConcept;
  /** 
   * Benefit classification
   */
  category?: CodeableConcept;
  /** 
   * Billing, service, product, or drug code
   */
  productOrService: CodeableConcept;
  /** 
   * Service/Product billing modifiers
   */
  modifier?: Array<CodeableConcept>;
  /** 
   * Program the product or service is provided under
   */
  programCode?: Array<CodeableConcept>;
  /** 
   * Count of products or services
   */
  quantity?: Quantity;
  /** 
   * Fee, charge or cost per item
   */
  unitPrice?: Money;
  /** 
   * Price scaling factor
   */
  factor?: decimal;
  /** 
   * Price scaling factor
   */
  _factor?: Element
  /** 
   * Total item cost
   */
  net?: Money;
  /** 
   * Unique device identifier
   */
  udi?: Array<Reference>;
  /** 
   * Applicable note numbers
   */
  noteNumber?: Array<positiveInt>;
  /** 
   * Applicable note numbers
   */
  _noteNumber?: Array<Element>
  /** 
   * Detail level adjudication details
   */
  adjudication?: Array<ExplanationOfBenefitItemAdjudication>;
  /** 
   * Additional items
   */
  subDetail?: Array<ExplanationOfBenefitItemDetailSubDetail>;
}
export interface ExplanationOfBenefitItem {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Item instance identifier
   */
  sequence: positiveInt;
  /** 
   * Item instance identifier
   */
  _sequence?: Element
  /** 
   * Applicable care team members
   */
  careTeamSequence?: Array<positiveInt>;
  /** 
   * Applicable care team members
   */
  _careTeamSequence?: Array<Element>
  /** 
   * Applicable diagnoses
   */
  diagnosisSequence?: Array<positiveInt>;
  /** 
   * Applicable diagnoses
   */
  _diagnosisSequence?: Array<Element>
  /** 
   * Applicable procedures
   */
  procedureSequence?: Array<positiveInt>;
  /** 
   * Applicable procedures
   */
  _procedureSequence?: Array<Element>
  /** 
   * Applicable exception and supporting information
   */
  informationSequence?: Array<positiveInt>;
  /** 
   * Applicable exception and supporting information
   */
  _informationSequence?: Array<Element>
  /** 
   * Revenue or cost center code
   */
  revenue?: CodeableConcept;
  /** 
   * Benefit classification
   */
  category?: CodeableConcept;
  /** 
   * Billing, service, product, or drug code
   */
  productOrService: CodeableConcept;
  /** 
   * Product or service billing modifiers
   */
  modifier?: Array<CodeableConcept>;
  /** 
   * Program the product or service is provided under
   */
  programCode?: Array<CodeableConcept>;
  /** 
   * Date or dates of service or product delivery
   */
  servicedDate?: date;
  /** 
   * Date or dates of service or product delivery
   */
  _servicedDate?: Element
  /** 
   * Date or dates of service or product delivery
   */
  servicedPeriod?: Period;
  /** 
   * Place of service or where product was supplied
   */
  locationCodeableConcept?: CodeableConcept;
  /** 
   * Place of service or where product was supplied
   */
  locationAddress?: Address;
  /** 
   * Place of service or where product was supplied
   */
  locationReference?: Reference;
  /** 
   * Count of products or services
   */
  quantity?: Quantity;
  /** 
   * Fee, charge or cost per item
   */
  unitPrice?: Money;
  /** 
   * Price scaling factor
   */
  factor?: decimal;
  /** 
   * Price scaling factor
   */
  _factor?: Element
  /** 
   * Total item cost
   */
  net?: Money;
  /** 
   * Unique device identifier
   */
  udi?: Array<Reference>;
  /** 
   * Anatomical location
   */
  bodySite?: CodeableConcept;
  /** 
   * Anatomical sub-location
   */
  subSite?: Array<CodeableConcept>;
  /** 
   * Encounters related to this billed item
   */
  encounter?: Array<Reference>;
  /** 
   * Applicable note numbers
   */
  noteNumber?: Array<positiveInt>;
  /** 
   * Applicable note numbers
   */
  _noteNumber?: Array<Element>
  /** 
   * Adjudication details
   */
  adjudication?: Array<ExplanationOfBenefitItemAdjudication>;
  /** 
   * Additional items
   */
  detail?: Array<ExplanationOfBenefitItemDetail>;
}
export interface ExplanationOfBenefitAddItemDetailSubDetail {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Billing, service, product, or drug code
   */
  productOrService: CodeableConcept;
  /** 
   * Service/Product billing modifiers
   */
  modifier?: Array<CodeableConcept>;
  /** 
   * Count of products or services
   */
  quantity?: Quantity;
  /** 
   * Fee, charge or cost per item
   */
  unitPrice?: Money;
  /** 
   * Price scaling factor
   */
  factor?: decimal;
  /** 
   * Price scaling factor
   */
  _factor?: Element
  /** 
   * Total item cost
   */
  net?: Money;
  /** 
   * Applicable note numbers
   */
  noteNumber?: Array<positiveInt>;
  /** 
   * Applicable note numbers
   */
  _noteNumber?: Array<Element>
  /** 
   * Added items adjudication
   */
  adjudication?: Array<ExplanationOfBenefitItemAdjudication>;
}
export interface ExplanationOfBenefitAddItemDetail {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Billing, service, product, or drug code
   */
  productOrService: CodeableConcept;
  /** 
   * Service/Product billing modifiers
   */
  modifier?: Array<CodeableConcept>;
  /** 
   * Count of products or services
   */
  quantity?: Quantity;
  /** 
   * Fee, charge or cost per item
   */
  unitPrice?: Money;
  /** 
   * Price scaling factor
   */
  factor?: decimal;
  /** 
   * Price scaling factor
   */
  _factor?: Element
  /** 
   * Total item cost
   */
  net?: Money;
  /** 
   * Applicable note numbers
   */
  noteNumber?: Array<positiveInt>;
  /** 
   * Applicable note numbers
   */
  _noteNumber?: Array<Element>
  /** 
   * Added items adjudication
   */
  adjudication?: Array<ExplanationOfBenefitItemAdjudication>;
  /** 
   * Insurer added line items
   */
  subDetail?: Array<ExplanationOfBenefitAddItemDetailSubDetail>;
}
export interface ExplanationOfBenefitAddItem {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Item sequence number
   */
  itemSequence?: Array<positiveInt>;
  /** 
   * Item sequence number
   */
  _itemSequence?: Array<Element>
  /** 
   * Detail sequence number
   */
  detailSequence?: Array<positiveInt>;
  /** 
   * Detail sequence number
   */
  _detailSequence?: Array<Element>
  /** 
   * Subdetail sequence number
   */
  subDetailSequence?: Array<positiveInt>;
  /** 
   * Subdetail sequence number
   */
  _subDetailSequence?: Array<Element>
  /** 
   * Authorized providers
   */
  provider?: Array<Reference>;
  /** 
   * Billing, service, product, or drug code
   */
  productOrService: CodeableConcept;
  /** 
   * Service/Product billing modifiers
   */
  modifier?: Array<CodeableConcept>;
  /** 
   * Program the product or service is provided under
   */
  programCode?: Array<CodeableConcept>;
  /** 
   * Date or dates of service or product delivery
   */
  servicedDate?: date;
  /** 
   * Date or dates of service or product delivery
   */
  _servicedDate?: Element
  /** 
   * Date or dates of service or product delivery
   */
  servicedPeriod?: Period;
  /** 
   * Place of service or where product was supplied
   */
  locationCodeableConcept?: CodeableConcept;
  /** 
   * Place of service or where product was supplied
   */
  locationAddress?: Address;
  /** 
   * Place of service or where product was supplied
   */
  locationReference?: Reference;
  /** 
   * Count of products or services
   */
  quantity?: Quantity;
  /** 
   * Fee, charge or cost per item
   */
  unitPrice?: Money;
  /** 
   * Price scaling factor
   */
  factor?: decimal;
  /** 
   * Price scaling factor
   */
  _factor?: Element
  /** 
   * Total item cost
   */
  net?: Money;
  /** 
   * Anatomical location
   */
  bodySite?: CodeableConcept;
  /** 
   * Anatomical sub-location
   */
  subSite?: Array<CodeableConcept>;
  /** 
   * Applicable note numbers
   */
  noteNumber?: Array<positiveInt>;
  /** 
   * Applicable note numbers
   */
  _noteNumber?: Array<Element>
  /** 
   * Added items adjudication
   */
  adjudication?: Array<ExplanationOfBenefitItemAdjudication>;
  /** 
   * Insurer added line items
   */
  detail?: Array<ExplanationOfBenefitAddItemDetail>;
}
export interface ExplanationOfBenefitTotal {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Type of adjudication information
   */
  category: CodeableConcept;
  /** 
   * Financial total for the category
   */
  amount: Money;
}
export interface ExplanationOfBenefitPayment {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Partial or complete payment
   */
  type?: CodeableConcept;
  /** 
   * Payment adjustment for non-claim issues
   */
  adjustment?: Money;
  /** 
   * Explanation for the variance
   */
  adjustmentReason?: CodeableConcept;
  /** 
   * Expected date of payment
   */
  date?: date;
  /** 
   * Expected date of payment
   */
  _date?: Element
  /** 
   * Payable amount after adjustment
   */
  amount?: Money;
  /** 
   * Business identifier for the payment
   */
  identifier?: Identifier;
}
export interface ExplanationOfBenefitProcessNote {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Note instance identifier
   */
  number?: positiveInt;
  /** 
   * Note instance identifier
   */
  _number?: Element
  /** 
   * display | print | printoper
   */
  type?: code;
  /** 
   * display | print | printoper
   */
  _type?: Element
  /** 
   * Note explanatory text
   */
  text?: string;
  /** 
   * Note explanatory text
   */
  _text?: Element
  /** 
   * Language of the text
   */
  language?: CodeableConcept;
}
export interface ExplanationOfBenefitBenefitBalanceFinancial {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Benefit classification
   */
  type: CodeableConcept;
  /** 
   * Benefits allowed
   */
  allowedUnsignedInt?: unsignedInt;
  /** 
   * Benefits allowed
   */
  _allowedUnsignedInt?: Element
  /** 
   * Benefits allowed
   */
  allowedString?: string;
  /** 
   * Benefits allowed
   */
  _allowedString?: Element
  /** 
   * Benefits allowed
   */
  allowedMoney?: Money;
  /** 
   * Benefits used
   */
  usedUnsignedInt?: unsignedInt;
  /** 
   * Benefits used
   */
  _usedUnsignedInt?: Element
  /** 
   * Benefits used
   */
  usedMoney?: Money;
}
export interface ExplanationOfBenefitBenefitBalance {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Benefit classification
   */
  category: CodeableConcept;
  /** 
   * Excluded from the plan
   */
  excluded?: boolean;
  /** 
   * Excluded from the plan
   */
  _excluded?: Element
  /** 
   * Short name for the benefit
   */
  name?: string;
  /** 
   * Short name for the benefit
   */
  _name?: Element
  /** 
   * Description of the benefit or services covered
   */
  description?: string;
  /** 
   * Description of the benefit or services covered
   */
  _description?: Element
  /** 
   * In or out of network
   */
  network?: CodeableConcept;
  /** 
   * Individual or family
   */
  unit?: CodeableConcept;
  /** 
   * Annual or lifetime
   */
  term?: CodeableConcept;
  /** 
   * Benefit Summary
   */
  financial?: Array<ExplanationOfBenefitBenefitBalanceFinancial>;
}
export interface ExplanationOfBenefit {
resourceType: "ExplanationOfBenefit"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business Identifier for the resource
   */
  identifier?: Array<Identifier>;
  /** 
   * active | cancelled | draft | entered-in-error
   */
  status: code;
  /** 
   * active | cancelled | draft | entered-in-error
   */
  _status?: Element
  /** 
   * Category or discipline
   */
  type: CodeableConcept;
  /** 
   * More granular claim type
   */
  subType?: CodeableConcept;
  /** 
   * claim | preauthorization | predetermination
   */
  use: code;
  /** 
   * claim | preauthorization | predetermination
   */
  _use?: Element
  /** 
   * The recipient of the products and services
   */
  patient: Reference;
  /** 
   * Relevant time frame for the claim
   */
  billablePeriod?: Period;
  /** 
   * Response creation date
   */
  created: dateTime;
  /** 
   * Response creation date
   */
  _created?: Element
  /** 
   * Author of the claim
   */
  enterer?: Reference;
  /** 
   * Party responsible for reimbursement
   */
  insurer: Reference;
  /** 
   * Party responsible for the claim
   */
  provider: Reference;
  /** 
   * Desired processing urgency
   */
  priority?: CodeableConcept;
  /** 
   * For whom to reserve funds
   */
  fundsReserveRequested?: CodeableConcept;
  /** 
   * Funds reserved status
   */
  fundsReserve?: CodeableConcept;
  /** 
   * Prior or corollary claims
   */
  related?: Array<ExplanationOfBenefitRelated>;
  /** 
   * Prescription authorizing services or products
   */
  prescription?: Reference;
  /** 
   * Original prescription if superceded by fulfiller
   */
  originalPrescription?: Reference;
  /** 
   * Recipient of benefits payable
   */
  payee?: ExplanationOfBenefitPayee;
  /** 
   * Treatment Referral
   */
  referral?: Reference;
  /** 
   * Servicing Facility
   */
  facility?: Reference;
  /** 
   * Claim reference
   */
  claim?: Reference;
  /** 
   * Claim response reference
   */
  claimResponse?: Reference;
  /** 
   * queued | complete | error | partial
   */
  outcome: code;
  /** 
   * queued | complete | error | partial
   */
  _outcome?: Element
  /** 
   * Disposition Message
   */
  disposition?: string;
  /** 
   * Disposition Message
   */
  _disposition?: Element
  /** 
   * Preauthorization reference
   */
  preAuthRef?: Array<string>;
  /** 
   * Preauthorization reference
   */
  _preAuthRef?: Array<Element>
  /** 
   * Preauthorization in-effect period
   */
  preAuthRefPeriod?: Array<Period>;
  /** 
   * Care Team members
   */
  careTeam?: Array<ExplanationOfBenefitCareTeam>;
  /** 
   * Supporting information
   */
  supportingInfo?: Array<ExplanationOfBenefitSupportingInfo>;
  /** 
   * Pertinent diagnosis information
   */
  diagnosis?: Array<ExplanationOfBenefitDiagnosis>;
  /** 
   * Clinical procedures performed
   */
  procedure?: Array<ExplanationOfBenefitProcedure>;
  /** 
   * Precedence (primary, secondary, etc.)
   */
  precedence?: positiveInt;
  /** 
   * Precedence (primary, secondary, etc.)
   */
  _precedence?: Element
  /** 
   * Patient insurance information
   */
  insurance: Array<ExplanationOfBenefitInsurance>;
  /** 
   * Details of the event
   */
  accident?: ExplanationOfBenefitAccident;
  /** 
   * Product or service provided
   */
  item?: Array<ExplanationOfBenefitItem>;
  /** 
   * Insurer added line items
   */
  addItem?: Array<ExplanationOfBenefitAddItem>;
  /** 
   * Header-level adjudication
   */
  adjudication?: Array<ExplanationOfBenefitItemAdjudication>;
  /** 
   * Adjudication totals
   */
  total?: Array<ExplanationOfBenefitTotal>;
  /** 
   * Payment Details
   */
  payment?: ExplanationOfBenefitPayment;
  /** 
   * Printed form identifier
   */
  formCode?: CodeableConcept;
  /** 
   * Printed reference or actual form
   */
  form?: Attachment;
  /** 
   * Note concerning adjudication
   */
  processNote?: Array<ExplanationOfBenefitProcessNote>;
  /** 
   * When the benefits are applicable
   */
  benefitPeriod?: Period;
  /** 
   * Balance by Benefit Category
   */
  benefitBalance?: Array<ExplanationOfBenefitBenefitBalance>;
}

export interface FamilyMemberHistoryCondition {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Condition suffered by relation
   */
  code: CodeableConcept;
  /** 
   * deceased | permanent disability | etc.
   */
  outcome?: CodeableConcept;
  /** 
   * Whether the condition contributed to the cause of death
   */
  contributedToDeath?: boolean;
  /** 
   * Whether the condition contributed to the cause of death
   */
  _contributedToDeath?: Element
  /** 
   * When condition first manifested
   */
  onsetAge?: Age;
  /** 
   * When condition first manifested
   */
  onsetRange?: Range;
  /** 
   * When condition first manifested
   */
  onsetPeriod?: Period;
  /** 
   * When condition first manifested
   */
  onsetString?: string;
  /** 
   * When condition first manifested
   */
  _onsetString?: Element
  /** 
   * Extra information about condition
   */
  note?: Array<Annotation>;
}
export interface FamilyMemberHistory {
resourceType: "FamilyMemberHistory"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * External Id(s) for this record
   */
  identifier?: Array<Identifier>;
  /** 
   * Instantiates FHIR protocol or definition
   */
  instantiatesCanonical?: Array<canonical>;
  /** 
   * Instantiates FHIR protocol or definition
   */
  _instantiatesCanonical?: Array<Element>
  /** 
   * Instantiates external protocol or definition
   */
  instantiatesUri?: Array<uri>;
  /** 
   * Instantiates external protocol or definition
   */
  _instantiatesUri?: Array<Element>
  /** 
   * partial | completed | entered-in-error | health-unknown
   */
  status: code;
  /** 
   * partial | completed | entered-in-error | health-unknown
   */
  _status?: Element
  /** 
   * subject-unknown | withheld | unable-to-obtain | deferred
   */
  dataAbsentReason?: CodeableConcept;
  /** 
   * Patient history is about
   */
  patient: Reference;
  /** 
   * When history was recorded or last updated
   */
  date?: dateTime;
  /** 
   * When history was recorded or last updated
   */
  _date?: Element
  /** 
   * The family member described
   */
  name?: string;
  /** 
   * The family member described
   */
  _name?: Element
  /** 
   * Relationship to the subject
   */
  relationship: CodeableConcept;
  /** 
   * male | female | other | unknown
   */
  sex?: CodeableConcept;
  /** 
   * (approximate) date of birth
   */
  bornPeriod?: Period;
  /** 
   * (approximate) date of birth
   */
  bornDate?: date;
  /** 
   * (approximate) date of birth
   */
  _bornDate?: Element
  /** 
   * (approximate) date of birth
   */
  bornString?: string;
  /** 
   * (approximate) date of birth
   */
  _bornString?: Element
  /** 
   * (approximate) age
   */
  ageAge?: Age;
  /** 
   * (approximate) age
   */
  ageRange?: Range;
  /** 
   * (approximate) age
   */
  ageString?: string;
  /** 
   * (approximate) age
   */
  _ageString?: Element
  /** 
   * Age is estimated?
   */
  estimatedAge?: boolean;
  /** 
   * Age is estimated?
   */
  _estimatedAge?: Element
  /** 
   * Dead? How old/when?
   */
  deceasedBoolean?: boolean;
  /** 
   * Dead? How old/when?
   */
  _deceasedBoolean?: Element
  /** 
   * Dead? How old/when?
   */
  deceasedAge?: Age;
  /** 
   * Dead? How old/when?
   */
  deceasedRange?: Range;
  /** 
   * Dead? How old/when?
   */
  deceasedDate?: date;
  /** 
   * Dead? How old/when?
   */
  _deceasedDate?: Element
  /** 
   * Dead? How old/when?
   */
  deceasedString?: string;
  /** 
   * Dead? How old/when?
   */
  _deceasedString?: Element
  /** 
   * Why was family member history performed?
   */
  reasonCode?: Array<CodeableConcept>;
  /** 
   * Why was family member history performed?
   */
  reasonReference?: Array<Reference>;
  /** 
   * General note about related person
   */
  note?: Array<Annotation>;
  /** 
   * Condition that the related person had
   */
  condition?: Array<FamilyMemberHistoryCondition>;
}

export interface Flag {
resourceType: "Flag"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business identifier
   */
  identifier?: Array<Identifier>;
  /** 
   * active | inactive | entered-in-error
   */
  status: code;
  /** 
   * active | inactive | entered-in-error
   */
  _status?: Element
  /** 
   * Clinical, administrative, etc.
   */
  category?: Array<CodeableConcept>;
  /** 
   * Coded or textual message to display to user
   */
  code: CodeableConcept;
  /** 
   * Who/What is flag about?
   */
  subject: Reference;
  /** 
   * Time period when flag is active
   */
  period?: Period;
  /** 
   * Alert relevant during encounter
   */
  encounter?: Reference;
  /** 
   * Flag creator
   */
  author?: Reference;
}

export interface GoalTarget {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The parameter whose value is being tracked
   */
  measure?: CodeableConcept;
  /** 
   * The target value to be achieved
   */
  detailQuantity?: Quantity;
  /** 
   * The target value to be achieved
   */
  detailRange?: Range;
  /** 
   * The target value to be achieved
   */
  detailCodeableConcept?: CodeableConcept;
  /** 
   * The target value to be achieved
   */
  detailString?: string;
  /** 
   * The target value to be achieved
   */
  _detailString?: Element
  /** 
   * The target value to be achieved
   */
  detailBoolean?: boolean;
  /** 
   * The target value to be achieved
   */
  _detailBoolean?: Element
  /** 
   * The target value to be achieved
   */
  detailInteger?: integer;
  /** 
   * The target value to be achieved
   */
  _detailInteger?: Element
  /** 
   * The target value to be achieved
   */
  detailRatio?: Ratio;
  /** 
   * Reach goal on or before
   */
  dueDate?: date;
  /** 
   * Reach goal on or before
   */
  _dueDate?: Element
  /** 
   * Reach goal on or before
   */
  dueDuration?: Duration;
}
export interface Goal {
resourceType: "Goal"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * External Ids for this goal
   */
  identifier?: Array<Identifier>;
  /** 
   * proposed | planned | accepted | active | on-hold | completed | cancelled | entered-in-error | rejected
   */
  lifecycleStatus: code;
  /** 
   * proposed | planned | accepted | active | on-hold | completed | cancelled | entered-in-error | rejected
   */
  _lifecycleStatus?: Element
  /** 
   * in-progress | improving | worsening | no-change | achieved | sustaining | not-achieved | no-progress | not-attainable
   */
  achievementStatus?: CodeableConcept;
  /** 
   * E.g. Treatment, dietary, behavioral, etc.
   */
  category?: Array<CodeableConcept>;
  /** 
   * high-priority | medium-priority | low-priority
   */
  priority?: CodeableConcept;
  /** 
   * Code or text describing goal
   */
  description: CodeableConcept;
  /** 
   * Who this goal is intended for
   */
  subject: Reference;
  /** 
   * When goal pursuit begins
   */
  startDate?: date;
  /** 
   * When goal pursuit begins
   */
  _startDate?: Element
  /** 
   * When goal pursuit begins
   */
  startCodeableConcept?: CodeableConcept;
  /** 
   * Target outcome for the goal
   */
  target?: Array<GoalTarget>;
  /** 
   * When goal status took effect
   */
  statusDate?: date;
  /** 
   * When goal status took effect
   */
  _statusDate?: Element
  /** 
   * Reason for current status
   */
  statusReason?: string;
  /** 
   * Reason for current status
   */
  _statusReason?: Element
  /** 
   * Who's responsible for creating Goal?
   */
  expressedBy?: Reference;
  /** 
   * Issues addressed by this goal
   */
  addresses?: Array<Reference>;
  /** 
   * Comments about the goal
   */
  note?: Array<Annotation>;
  /** 
   * What result was achieved regarding the goal?
   */
  outcomeCode?: Array<CodeableConcept>;
  /** 
   * Observation that resulted from goal
   */
  outcomeReference?: Array<Reference>;
}

export interface GraphDefinitionLinkTargetCompartment {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * condition | requirement
   */
  use: code;
  /** 
   * condition | requirement
   */
  _use?: Element
  /** 
   * Patient | Encounter | RelatedPerson | Practitioner | Device
   */
  code: code;
  /** 
   * Patient | Encounter | RelatedPerson | Practitioner | Device
   */
  _code?: Element
  /** 
   * identical | matching | different | custom
   */
  rule: code;
  /** 
   * identical | matching | different | custom
   */
  _rule?: Element
  /** 
   * Custom rule, as a FHIRPath expression
   */
  expression?: string;
  /** 
   * Custom rule, as a FHIRPath expression
   */
  _expression?: Element
  /** 
   * Documentation for FHIRPath expression
   */
  description?: string;
  /** 
   * Documentation for FHIRPath expression
   */
  _description?: Element
}
export interface GraphDefinitionLinkTarget {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Type of resource this link refers to
   */
  type: code;
  /** 
   * Type of resource this link refers to
   */
  _type?: Element
  /** 
   * Criteria for reverse lookup
   */
  params?: string;
  /** 
   * Criteria for reverse lookup
   */
  _params?: Element
  /** 
   * Profile for the target resource
   */
  profile?: canonical;
  /** 
   * Profile for the target resource
   */
  _profile?: Element
  /** 
   * Compartment Consistency Rules
   */
  compartment?: Array<GraphDefinitionLinkTargetCompartment>;
  /** 
   * Additional links from target resource
   */
  link?: Array<GraphDefinitionLink>;
}
export interface GraphDefinitionLink {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Path in the resource that contains the link
   */
  path?: string;
  /** 
   * Path in the resource that contains the link
   */
  _path?: Element
  /** 
   * Which slice (if profiled)
   */
  sliceName?: string;
  /** 
   * Which slice (if profiled)
   */
  _sliceName?: Element
  /** 
   * Minimum occurrences for this link
   */
  min?: integer;
  /** 
   * Minimum occurrences for this link
   */
  _min?: Element
  /** 
   * Maximum occurrences for this link
   */
  max?: string;
  /** 
   * Maximum occurrences for this link
   */
  _max?: Element
  /** 
   * Why this link is specified
   */
  description?: string;
  /** 
   * Why this link is specified
   */
  _description?: Element
  /** 
   * Potential target for the link
   */
  target?: Array<GraphDefinitionLinkTarget>;
}
export interface GraphDefinition {
resourceType: "GraphDefinition"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Canonical identifier for this graph definition, represented as a URI (globally unique)
   */
  url?: uri;
  /** 
   * Canonical identifier for this graph definition, represented as a URI (globally unique)
   */
  _url?: Element
  /** 
   * Business version of the graph definition
   */
  version?: string;
  /** 
   * Business version of the graph definition
   */
  _version?: Element
  /** 
   * Name for this graph definition (computer friendly)
   */
  name: string;
  /** 
   * Name for this graph definition (computer friendly)
   */
  _name?: Element
  /** 
   * draft | active | retired | unknown
   */
  status: code;
  /** 
   * draft | active | retired | unknown
   */
  _status?: Element
  /** 
   * For testing purposes, not real usage
   */
  experimental?: boolean;
  /** 
   * For testing purposes, not real usage
   */
  _experimental?: Element
  /** 
   * Date last changed
   */
  date?: dateTime;
  /** 
   * Date last changed
   */
  _date?: Element
  /** 
   * Name of the publisher (organization or individual)
   */
  publisher?: string;
  /** 
   * Name of the publisher (organization or individual)
   */
  _publisher?: Element
  /** 
   * Contact details for the publisher
   */
  contact?: Array<ContactDetail>;
  /** 
   * Natural language description of the graph definition
   */
  description?: markdown;
  /** 
   * Natural language description of the graph definition
   */
  _description?: Element
  /** 
   * The context that the content is intended to support
   */
  useContext?: Array<UsageContext>;
  /** 
   * Intended jurisdiction for graph definition (if applicable)
   */
  jurisdiction?: Array<CodeableConcept>;
  /** 
   * Why this graph definition is defined
   */
  purpose?: markdown;
  /** 
   * Why this graph definition is defined
   */
  _purpose?: Element
  /** 
   * Type of resource at which the graph starts
   */
  start: code;
  /** 
   * Type of resource at which the graph starts
   */
  _start?: Element
  /** 
   * Profile on base resource
   */
  profile?: canonical;
  /** 
   * Profile on base resource
   */
  _profile?: Element
  /** 
   * Links this graph makes rules about
   */
  link?: Array<GraphDefinitionLink>;
}

export interface GroupCharacteristic {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Kind of characteristic
   */
  code: CodeableConcept;
  /** 
   * Value held by characteristic
   */
  valueCodeableConcept?: CodeableConcept;
  /** 
   * Value held by characteristic
   */
  valueBoolean?: boolean;
  /** 
   * Value held by characteristic
   */
  _valueBoolean?: Element
  /** 
   * Value held by characteristic
   */
  valueQuantity?: Quantity;
  /** 
   * Value held by characteristic
   */
  valueRange?: Range;
  /** 
   * Value held by characteristic
   */
  valueReference?: Reference;
  /** 
   * Group includes or excludes
   */
  exclude: boolean;
  /** 
   * Group includes or excludes
   */
  _exclude?: Element
  /** 
   * Period over which characteristic is tested
   */
  period?: Period;
}
export interface GroupMember {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Reference to the group member
   */
  entity: Reference;
  /** 
   * Period member belonged to the group
   */
  period?: Period;
  /** 
   * If member is no longer in group
   */
  inactive?: boolean;
  /** 
   * If member is no longer in group
   */
  _inactive?: Element
}
export interface Group {
resourceType: "Group"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Unique id
   */
  identifier?: Array<Identifier>;
  /** 
   * Whether this group's record is in active use
   */
  active?: boolean;
  /** 
   * Whether this group's record is in active use
   */
  _active?: Element
  /** 
   * person | animal | practitioner | device | medication | substance
   */
  type: code;
  /** 
   * person | animal | practitioner | device | medication | substance
   */
  _type?: Element
  /** 
   * Descriptive or actual
   */
  actual: boolean;
  /** 
   * Descriptive or actual
   */
  _actual?: Element
  /** 
   * Kind of Group members
   */
  code?: CodeableConcept;
  /** 
   * Label for Group
   */
  name?: string;
  /** 
   * Label for Group
   */
  _name?: Element
  /** 
   * Number of members
   */
  quantity?: unsignedInt;
  /** 
   * Number of members
   */
  _quantity?: Element
  /** 
   * Entity that is the custodian of the Group's definition
   */
  managingEntity?: Reference;
  /** 
   * Include / Exclude group members by Trait
   */
  characteristic?: Array<GroupCharacteristic>;
  /** 
   * Who or what is in group
   */
  member?: Array<GroupMember>;
}

export interface GuidanceResponse {
resourceType: "GuidanceResponse"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The identifier of the request associated with this response, if any
   */
  requestIdentifier?: Identifier;
  /** 
   * Business identifier
   */
  identifier?: Array<Identifier>;
  /** 
   * What guidance was requested
   */
  moduleUri?: uri;
  /** 
   * What guidance was requested
   */
  _moduleUri?: Element
  /** 
   * What guidance was requested
   */
  moduleCanonical?: canonical;
  /** 
   * What guidance was requested
   */
  _moduleCanonical?: Element
  /** 
   * What guidance was requested
   */
  moduleCodeableConcept?: CodeableConcept;
  /** 
   * success | data-requested | data-required | in-progress | failure | entered-in-error
   */
  status: code;
  /** 
   * success | data-requested | data-required | in-progress | failure | entered-in-error
   */
  _status?: Element
  /** 
   * Patient the request was performed for
   */
  subject?: Reference;
  /** 
   * Encounter during which the response was returned
   */
  encounter?: Reference;
  /** 
   * When the guidance response was processed
   */
  occurrenceDateTime?: dateTime;
  /** 
   * When the guidance response was processed
   */
  _occurrenceDateTime?: Element
  /** 
   * Device returning the guidance
   */
  performer?: Reference;
  /** 
   * Why guidance is needed
   */
  reasonCode?: Array<CodeableConcept>;
  /** 
   * Why guidance is needed
   */
  reasonReference?: Array<Reference>;
  /** 
   * Additional notes about the response
   */
  note?: Array<Annotation>;
  /** 
   * Messages resulting from the evaluation of the artifact or artifacts
   */
  evaluationMessage?: Array<Reference>;
  /** 
   * The output parameters of the evaluation, if any
   */
  outputParameters?: Reference;
  /** 
   * Proposed actions, if any
   */
  result?: Reference;
  /** 
   * Additional required data
   */
  dataRequirement?: Array<DataRequirement>;
}

export interface HealthcareServiceEligibility {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Coded value for the eligibility
   */
  code?: CodeableConcept;
  /** 
   * Describes the eligibility conditions for the service
   */
  comment?: markdown;
  /** 
   * Describes the eligibility conditions for the service
   */
  _comment?: Element
}
export interface HealthcareServiceAvailableTime {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * mon | tue | wed | thu | fri | sat | sun
   */
  daysOfWeek?: Array<code>;
  /** 
   * mon | tue | wed | thu | fri | sat | sun
   */
  _daysOfWeek?: Array<Element>
  /** 
   * Always available? e.g. 24 hour service
   */
  allDay?: boolean;
  /** 
   * Always available? e.g. 24 hour service
   */
  _allDay?: Element
  /** 
   * Opening time of day (ignored if allDay = true)
   */
  availableStartTime?: time;
  /** 
   * Opening time of day (ignored if allDay = true)
   */
  _availableStartTime?: Element
  /** 
   * Closing time of day (ignored if allDay = true)
   */
  availableEndTime?: time;
  /** 
   * Closing time of day (ignored if allDay = true)
   */
  _availableEndTime?: Element
}
export interface HealthcareServiceNotAvailable {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Reason presented to the user explaining why time not available
   */
  description: string;
  /** 
   * Reason presented to the user explaining why time not available
   */
  _description?: Element
  /** 
   * Service not available from this date
   */
  during?: Period;
}
export interface HealthcareService {
resourceType: "HealthcareService"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * External identifiers for this item
   */
  identifier?: Array<Identifier>;
  /** 
   * Whether this HealthcareService record is in active use
   */
  active?: boolean;
  /** 
   * Whether this HealthcareService record is in active use
   */
  _active?: Element
  /** 
   * Organization that provides this service
   */
  providedBy?: Reference;
  /** 
   * Broad category of service being performed or delivered
   */
  category?: Array<CodeableConcept>;
  /** 
   * Type of service that may be delivered or performed
   */
  type?: Array<CodeableConcept>;
  /** 
   * Specialties handled by the HealthcareService
   */
  specialty?: Array<CodeableConcept>;
  /** 
   * Location(s) where service may be provided
   */
  location?: Array<Reference>;
  /** 
   * Description of service as presented to a consumer while searching
   */
  name?: string;
  /** 
   * Description of service as presented to a consumer while searching
   */
  _name?: Element
  /** 
   * Additional description and/or any specific issues not covered elsewhere
   */
  comment?: string;
  /** 
   * Additional description and/or any specific issues not covered elsewhere
   */
  _comment?: Element
  /** 
   * Extra details about the service that can't be placed in the other fields
   */
  extraDetails?: markdown;
  /** 
   * Extra details about the service that can't be placed in the other fields
   */
  _extraDetails?: Element
  /** 
   * Facilitates quick identification of the service
   */
  photo?: Attachment;
  /** 
   * Contacts related to the healthcare service
   */
  telecom?: Array<ContactPoint>;
  /** 
   * Location(s) service is intended for/available to
   */
  coverageArea?: Array<Reference>;
  /** 
   * Conditions under which service is available/offered
   */
  serviceProvisionCode?: Array<CodeableConcept>;
  /** 
   * Specific eligibility requirements required to use the service
   */
  eligibility?: Array<HealthcareServiceEligibility>;
  /** 
   * Programs that this service is applicable to
   */
  program?: Array<CodeableConcept>;
  /** 
   * Collection of characteristics (attributes)
   */
  characteristic?: Array<CodeableConcept>;
  /** 
   * The language that this service is offered in
   */
  communication?: Array<CodeableConcept>;
  /** 
   * Ways that the service accepts referrals
   */
  referralMethod?: Array<CodeableConcept>;
  /** 
   * If an appointment is required for access to this service
   */
  appointmentRequired?: boolean;
  /** 
   * If an appointment is required for access to this service
   */
  _appointmentRequired?: Element
  /** 
   * Times the Service Site is available
   */
  availableTime?: Array<HealthcareServiceAvailableTime>;
  /** 
   * Not available during this time due to provided reason
   */
  notAvailable?: Array<HealthcareServiceNotAvailable>;
  /** 
   * Description of availability exceptions
   */
  availabilityExceptions?: string;
  /** 
   * Description of availability exceptions
   */
  _availabilityExceptions?: Element
  /** 
   * Technical endpoints providing access to electronic services operated for the healthcare service
   */
  endpoint?: Array<Reference>;
}

export interface ImagingStudySeriesPerformer {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Type of performance
   */
  function?: CodeableConcept;
  /** 
   * Who performed the series
   */
  actor: Reference;
}
export interface ImagingStudySeriesInstance {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * DICOM SOP Instance UID
   */
  uid: id;
  /** 
   * DICOM SOP Instance UID
   */
  _uid?: Element
  /** 
   * DICOM class type
   */
  sopClass: Coding;
  /** 
   * The number of this instance in the series
   */
  number?: unsignedInt;
  /** 
   * The number of this instance in the series
   */
  _number?: Element
  /** 
   * Description of instance
   */
  title?: string;
  /** 
   * Description of instance
   */
  _title?: Element
}
export interface ImagingStudySeries {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * DICOM Series Instance UID for the series
   */
  uid: id;
  /** 
   * DICOM Series Instance UID for the series
   */
  _uid?: Element
  /** 
   * Numeric identifier of this series
   */
  number?: unsignedInt;
  /** 
   * Numeric identifier of this series
   */
  _number?: Element
  /** 
   * The modality of the instances in the series
   */
  modality: Coding;
  /** 
   * A short human readable summary of the series
   */
  description?: string;
  /** 
   * A short human readable summary of the series
   */
  _description?: Element
  /** 
   * Number of Series Related Instances
   */
  numberOfInstances?: unsignedInt;
  /** 
   * Number of Series Related Instances
   */
  _numberOfInstances?: Element
  /** 
   * Series access endpoint
   */
  endpoint?: Array<Reference>;
  /** 
   * Body part examined
   */
  bodySite?: Coding;
  /** 
   * Body part laterality
   */
  laterality?: Coding;
  /** 
   * Specimen imaged
   */
  specimen?: Array<Reference>;
  /** 
   * When the series started
   */
  started?: dateTime;
  /** 
   * When the series started
   */
  _started?: Element
  /** 
   * Who performed the series
   */
  performer?: Array<ImagingStudySeriesPerformer>;
  /** 
   * A single SOP instance from the series
   */
  instance?: Array<ImagingStudySeriesInstance>;
}
export interface ImagingStudy {
resourceType: "ImagingStudy"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Identifiers for the whole study
   */
  identifier?: Array<Identifier>;
  /** 
   * registered | available | cancelled | entered-in-error | unknown
   */
  status: code;
  /** 
   * registered | available | cancelled | entered-in-error | unknown
   */
  _status?: Element
  /** 
   * All series modality if actual acquisition modalities
   */
  modality?: Array<Coding>;
  /** 
   * Who or what is the subject of the study
   */
  subject: Reference;
  /** 
   * Encounter with which this imaging study is associated
   */
  encounter?: Reference;
  /** 
   * When the study was started
   */
  started?: dateTime;
  /** 
   * When the study was started
   */
  _started?: Element
  /** 
   * Request fulfilled
   */
  basedOn?: Array<Reference>;
  /** 
   * Referring physician
   */
  referrer?: Reference;
  /** 
   * Who interpreted images
   */
  interpreter?: Array<Reference>;
  /** 
   * Study access endpoint
   */
  endpoint?: Array<Reference>;
  /** 
   * Number of Study Related Series
   */
  numberOfSeries?: unsignedInt;
  /** 
   * Number of Study Related Series
   */
  _numberOfSeries?: Element
  /** 
   * Number of Study Related Instances
   */
  numberOfInstances?: unsignedInt;
  /** 
   * Number of Study Related Instances
   */
  _numberOfInstances?: Element
  /** 
   * The performed Procedure reference
   */
  procedureReference?: Reference;
  /** 
   * The performed procedure code
   */
  procedureCode?: Array<CodeableConcept>;
  /** 
   * Where ImagingStudy occurred
   */
  location?: Reference;
  /** 
   * Why the study was requested
   */
  reasonCode?: Array<CodeableConcept>;
  /** 
   * Why was study performed
   */
  reasonReference?: Array<Reference>;
  /** 
   * User-defined comments
   */
  note?: Array<Annotation>;
  /** 
   * Institution-generated description
   */
  description?: string;
  /** 
   * Institution-generated description
   */
  _description?: Element
  /** 
   * Each study has one or more series of instances
   */
  series?: Array<ImagingStudySeries>;
}

export interface ImmunizationPerformer {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * What type of performance was done
   */
  function?: CodeableConcept;
  /** 
   * Individual or organization who was performing
   */
  actor: Reference;
}
export interface ImmunizationEducation {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Educational material document identifier
   */
  documentType?: string;
  /** 
   * Educational material document identifier
   */
  _documentType?: Element
  /** 
   * Educational material reference pointer
   */
  reference?: uri;
  /** 
   * Educational material reference pointer
   */
  _reference?: Element
  /** 
   * Educational material publication date
   */
  publicationDate?: dateTime;
  /** 
   * Educational material publication date
   */
  _publicationDate?: Element
  /** 
   * Educational material presentation date
   */
  presentationDate?: dateTime;
  /** 
   * Educational material presentation date
   */
  _presentationDate?: Element
}
export interface ImmunizationReaction {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * When reaction started
   */
  date?: dateTime;
  /** 
   * When reaction started
   */
  _date?: Element
  /** 
   * Additional information on reaction
   */
  detail?: Reference;
  /** 
   * Indicates self-reported reaction
   */
  reported?: boolean;
  /** 
   * Indicates self-reported reaction
   */
  _reported?: Element
}
export interface ImmunizationProtocolApplied {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Name of vaccine series
   */
  series?: string;
  /** 
   * Name of vaccine series
   */
  _series?: Element
  /** 
   * Who is responsible for publishing the recommendations
   */
  authority?: Reference;
  /** 
   * Vaccine preventatable disease being targetted
   */
  targetDisease?: Array<CodeableConcept>;
  /** 
   * Dose number within series
   */
  doseNumberPositiveInt?: positiveInt;
  /** 
   * Dose number within series
   */
  _doseNumberPositiveInt?: Element
  /** 
   * Dose number within series
   */
  doseNumberString?: string;
  /** 
   * Dose number within series
   */
  _doseNumberString?: Element
  /** 
   * Recommended number of doses for immunity
   */
  seriesDosesPositiveInt?: positiveInt;
  /** 
   * Recommended number of doses for immunity
   */
  _seriesDosesPositiveInt?: Element
  /** 
   * Recommended number of doses for immunity
   */
  seriesDosesString?: string;
  /** 
   * Recommended number of doses for immunity
   */
  _seriesDosesString?: Element
}
export interface Immunization {
resourceType: "Immunization"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business identifier
   */
  identifier?: Array<Identifier>;
  /** 
   * completed | entered-in-error | not-done
   */
  status: code;
  /** 
   * completed | entered-in-error | not-done
   */
  _status?: Element
  /** 
   * Reason not done
   */
  statusReason?: CodeableConcept;
  /** 
   * Vaccine product administered
   */
  vaccineCode: CodeableConcept;
  /** 
   * Who was immunized
   */
  patient: Reference;
  /** 
   * Encounter immunization was part of
   */
  encounter?: Reference;
  /** 
   * Vaccine administration date
   */
  occurrenceDateTime?: dateTime;
  /** 
   * Vaccine administration date
   */
  _occurrenceDateTime?: Element
  /** 
   * Vaccine administration date
   */
  occurrenceString?: string;
  /** 
   * Vaccine administration date
   */
  _occurrenceString?: Element
  /** 
   * When the immunization was first captured in the subject's record
   */
  recorded?: dateTime;
  /** 
   * When the immunization was first captured in the subject's record
   */
  _recorded?: Element
  /** 
   * Indicates context the data was recorded in
   */
  primarySource?: boolean;
  /** 
   * Indicates context the data was recorded in
   */
  _primarySource?: Element
  /** 
   * Indicates the source of a secondarily reported record
   */
  reportOrigin?: CodeableConcept;
  /** 
   * Where immunization occurred
   */
  location?: Reference;
  /** 
   * Vaccine manufacturer
   */
  manufacturer?: Reference;
  /** 
   * Vaccine lot number
   */
  lotNumber?: string;
  /** 
   * Vaccine lot number
   */
  _lotNumber?: Element
  /** 
   * Vaccine expiration date
   */
  expirationDate?: date;
  /** 
   * Vaccine expiration date
   */
  _expirationDate?: Element
  /** 
   * Body site vaccine  was administered
   */
  site?: CodeableConcept;
  /** 
   * How vaccine entered body
   */
  route?: CodeableConcept;
  /** 
   * Amount of vaccine administered
   */
  doseQuantity?: Quantity;
  /** 
   * Who performed event
   */
  performer?: Array<ImmunizationPerformer>;
  /** 
   * Additional immunization notes
   */
  note?: Array<Annotation>;
  /** 
   * Why immunization occurred
   */
  reasonCode?: Array<CodeableConcept>;
  /** 
   * Why immunization occurred
   */
  reasonReference?: Array<Reference>;
  /** 
   * Dose potency
   */
  isSubpotent?: boolean;
  /** 
   * Dose potency
   */
  _isSubpotent?: Element
  /** 
   * Reason for being subpotent
   */
  subpotentReason?: Array<CodeableConcept>;
  /** 
   * Educational material presented to patient
   */
  education?: Array<ImmunizationEducation>;
  /** 
   * Patient eligibility for a vaccination program
   */
  programEligibility?: Array<CodeableConcept>;
  /** 
   * Funding source for the vaccine
   */
  fundingSource?: CodeableConcept;
  /** 
   * Details of a reaction that follows immunization
   */
  reaction?: Array<ImmunizationReaction>;
  /** 
   * Protocol followed by the provider
   */
  protocolApplied?: Array<ImmunizationProtocolApplied>;
}

export interface ImmunizationEvaluation {
resourceType: "ImmunizationEvaluation"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business identifier
   */
  identifier?: Array<Identifier>;
  /** 
   * completed | entered-in-error
   */
  status: code;
  /** 
   * completed | entered-in-error
   */
  _status?: Element
  /** 
   * Who this evaluation is for
   */
  patient: Reference;
  /** 
   * Date evaluation was performed
   */
  date?: dateTime;
  /** 
   * Date evaluation was performed
   */
  _date?: Element
  /** 
   * Who is responsible for publishing the recommendations
   */
  authority?: Reference;
  /** 
   * Evaluation target disease
   */
  targetDisease: CodeableConcept;
  /** 
   * Immunization being evaluated
   */
  immunizationEvent: Reference;
  /** 
   * Status of the dose relative to published recommendations
   */
  doseStatus: CodeableConcept;
  /** 
   * Reason for the dose status
   */
  doseStatusReason?: Array<CodeableConcept>;
  /** 
   * Evaluation notes
   */
  description?: string;
  /** 
   * Evaluation notes
   */
  _description?: Element
  /** 
   * Name of vaccine series
   */
  series?: string;
  /** 
   * Name of vaccine series
   */
  _series?: Element
  /** 
   * Dose number within series
   */
  doseNumberPositiveInt?: positiveInt;
  /** 
   * Dose number within series
   */
  _doseNumberPositiveInt?: Element
  /** 
   * Dose number within series
   */
  doseNumberString?: string;
  /** 
   * Dose number within series
   */
  _doseNumberString?: Element
  /** 
   * Recommended number of doses for immunity
   */
  seriesDosesPositiveInt?: positiveInt;
  /** 
   * Recommended number of doses for immunity
   */
  _seriesDosesPositiveInt?: Element
  /** 
   * Recommended number of doses for immunity
   */
  seriesDosesString?: string;
  /** 
   * Recommended number of doses for immunity
   */
  _seriesDosesString?: Element
}

export interface ImmunizationRecommendationRecommendationDateCriterion {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Type of date
   */
  code: CodeableConcept;
  /** 
   * Recommended date
   */
  value: dateTime;
  /** 
   * Recommended date
   */
  _value?: Element
}
export interface ImmunizationRecommendationRecommendation {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Vaccine  or vaccine group recommendation applies to
   */
  vaccineCode?: Array<CodeableConcept>;
  /** 
   * Disease to be immunized against
   */
  targetDisease?: CodeableConcept;
  /** 
   * Vaccine which is contraindicated to fulfill the recommendation
   */
  contraindicatedVaccineCode?: Array<CodeableConcept>;
  /** 
   * Vaccine recommendation status
   */
  forecastStatus: CodeableConcept;
  /** 
   * Vaccine administration status reason
   */
  forecastReason?: Array<CodeableConcept>;
  /** 
   * Dates governing proposed immunization
   */
  dateCriterion?: Array<ImmunizationRecommendationRecommendationDateCriterion>;
  /** 
   * Protocol details
   */
  description?: string;
  /** 
   * Protocol details
   */
  _description?: Element
  /** 
   * Name of vaccination series
   */
  series?: string;
  /** 
   * Name of vaccination series
   */
  _series?: Element
  /** 
   * Recommended dose number within series
   */
  doseNumberPositiveInt?: positiveInt;
  /** 
   * Recommended dose number within series
   */
  _doseNumberPositiveInt?: Element
  /** 
   * Recommended dose number within series
   */
  doseNumberString?: string;
  /** 
   * Recommended dose number within series
   */
  _doseNumberString?: Element
  /** 
   * Recommended number of doses for immunity
   */
  seriesDosesPositiveInt?: positiveInt;
  /** 
   * Recommended number of doses for immunity
   */
  _seriesDosesPositiveInt?: Element
  /** 
   * Recommended number of doses for immunity
   */
  seriesDosesString?: string;
  /** 
   * Recommended number of doses for immunity
   */
  _seriesDosesString?: Element
  /** 
   * Past immunizations supporting recommendation
   */
  supportingImmunization?: Array<Reference>;
  /** 
   * Patient observations supporting recommendation
   */
  supportingPatientInformation?: Array<Reference>;
}
export interface ImmunizationRecommendation {
resourceType: "ImmunizationRecommendation"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business identifier
   */
  identifier?: Array<Identifier>;
  /** 
   * Who this profile is for
   */
  patient: Reference;
  /** 
   * Date recommendation(s) created
   */
  date: dateTime;
  /** 
   * Date recommendation(s) created
   */
  _date?: Element
  /** 
   * Who is responsible for protocol
   */
  authority?: Reference;
  /** 
   * Vaccine administration recommendations
   */
  recommendation: Array<ImmunizationRecommendationRecommendation>;
}

export interface ImplementationGuideDependsOn {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Identity of the IG that this depends on
   */
  uri: canonical;
  /** 
   * Identity of the IG that this depends on
   */
  _uri?: Element
  /** 
   * NPM Package name for IG this depends on
   */
  packageId?: id;
  /** 
   * NPM Package name for IG this depends on
   */
  _packageId?: Element
  /** 
   * Version of the IG
   */
  version?: string;
  /** 
   * Version of the IG
   */
  _version?: Element
}
export interface ImplementationGuideGlobal {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Type this profile applies to
   */
  type: code;
  /** 
   * Type this profile applies to
   */
  _type?: Element
  /** 
   * Profile that all resources must conform to
   */
  profile: canonical;
  /** 
   * Profile that all resources must conform to
   */
  _profile?: Element
}
export interface ImplementationGuideDefinitionGrouping {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Descriptive name for the package
   */
  name: string;
  /** 
   * Descriptive name for the package
   */
  _name?: Element
  /** 
   * Human readable text describing the package
   */
  description?: string;
  /** 
   * Human readable text describing the package
   */
  _description?: Element
}
export interface ImplementationGuideDefinitionResource {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Location of the resource
   */
  reference: Reference;
  /** 
   * Versions this applies to (if different to IG)
   */
  fhirVersion?: Array<code>;
  /** 
   * Versions this applies to (if different to IG)
   */
  _fhirVersion?: Array<Element>
  /** 
   * Human Name for the resource
   */
  name?: string;
  /** 
   * Human Name for the resource
   */
  _name?: Element
  /** 
   * Reason why included in guide
   */
  description?: string;
  /** 
   * Reason why included in guide
   */
  _description?: Element
  /** 
   * Is an example/What is this an example of?
   */
  exampleBoolean?: boolean;
  /** 
   * Is an example/What is this an example of?
   */
  _exampleBoolean?: Element
  /** 
   * Is an example/What is this an example of?
   */
  exampleCanonical?: canonical;
  /** 
   * Is an example/What is this an example of?
   */
  _exampleCanonical?: Element
  /** 
   * Grouping this is part of
   */
  groupingId?: id;
  /** 
   * Grouping this is part of
   */
  _groupingId?: Element
}
export interface ImplementationGuideDefinitionPage {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Where to find that page
   */
  nameUrl?: url;
  /** 
   * Where to find that page
   */
  _nameUrl?: Element
  /** 
   * Where to find that page
   */
  nameReference?: Reference;
  /** 
   * Short title shown for navigational assistance
   */
  title: string;
  /** 
   * Short title shown for navigational assistance
   */
  _title?: Element
  /** 
   * html | markdown | xml | generated
   */
  generation: code;
  /** 
   * html | markdown | xml | generated
   */
  _generation?: Element
  /** 
   * Nested Pages / Sections
   */
  page?: Array<ImplementationGuideDefinitionPage>;
}
export interface ImplementationGuideDefinitionParameter {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * apply | path-resource | path-pages | path-tx-cache | expansion-parameter | rule-broken-links | generate-xml | generate-json | generate-turtle | html-template
   */
  code: code;
  /** 
   * apply | path-resource | path-pages | path-tx-cache | expansion-parameter | rule-broken-links | generate-xml | generate-json | generate-turtle | html-template
   */
  _code?: Element
  /** 
   * Value for named type
   */
  value: string;
  /** 
   * Value for named type
   */
  _value?: Element
}
export interface ImplementationGuideDefinitionTemplate {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Type of template specified
   */
  code: code;
  /** 
   * Type of template specified
   */
  _code?: Element
  /** 
   * The source location for the template
   */
  source: string;
  /** 
   * The source location for the template
   */
  _source?: Element
  /** 
   * The scope in which the template applies
   */
  scope?: string;
  /** 
   * The scope in which the template applies
   */
  _scope?: Element
}
export interface ImplementationGuideDefinition {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Grouping used to present related resources in the IG
   */
  grouping?: Array<ImplementationGuideDefinitionGrouping>;
  /** 
   * Resource in the implementation guide
   */
  resource: Array<ImplementationGuideDefinitionResource>;
  /** 
   * Page/Section in the Guide
   */
  page?: ImplementationGuideDefinitionPage;
  /** 
   * Defines how IG is built by tools
   */
  parameter?: Array<ImplementationGuideDefinitionParameter>;
  /** 
   * A template for building resources
   */
  template?: Array<ImplementationGuideDefinitionTemplate>;
}
export interface ImplementationGuideManifestResource {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Location of the resource
   */
  reference: Reference;
  /** 
   * Is an example/What is this an example of?
   */
  exampleBoolean?: boolean;
  /** 
   * Is an example/What is this an example of?
   */
  _exampleBoolean?: Element
  /** 
   * Is an example/What is this an example of?
   */
  exampleCanonical?: canonical;
  /** 
   * Is an example/What is this an example of?
   */
  _exampleCanonical?: Element
  /** 
   * Relative path for page in IG
   */
  relativePath?: url;
  /** 
   * Relative path for page in IG
   */
  _relativePath?: Element
}
export interface ImplementationGuideManifestPage {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * HTML page name
   */
  name: string;
  /** 
   * HTML page name
   */
  _name?: Element
  /** 
   * Title of the page, for references
   */
  title?: string;
  /** 
   * Title of the page, for references
   */
  _title?: Element
  /** 
   * Anchor available on the page
   */
  anchor?: Array<string>;
  /** 
   * Anchor available on the page
   */
  _anchor?: Array<Element>
}
export interface ImplementationGuideManifest {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Location of rendered implementation guide
   */
  rendering?: url;
  /** 
   * Location of rendered implementation guide
   */
  _rendering?: Element
  /** 
   * Resource in the implementation guide
   */
  resource: Array<ImplementationGuideManifestResource>;
  /** 
   * HTML page within the parent IG
   */
  page?: Array<ImplementationGuideManifestPage>;
  /** 
   * Image within the IG
   */
  image?: Array<string>;
  /** 
   * Image within the IG
   */
  _image?: Array<Element>
  /** 
   * Additional linkable file in IG
   */
  other?: Array<string>;
  /** 
   * Additional linkable file in IG
   */
  _other?: Array<Element>
}
export interface ImplementationGuide {
resourceType: "ImplementationGuide"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Canonical identifier for this implementation guide, represented as a URI (globally unique)
   */
  url: uri;
  /** 
   * Canonical identifier for this implementation guide, represented as a URI (globally unique)
   */
  _url?: Element
  /** 
   * Business version of the implementation guide
   */
  version?: string;
  /** 
   * Business version of the implementation guide
   */
  _version?: Element
  /** 
   * Name for this implementation guide (computer friendly)
   */
  name: string;
  /** 
   * Name for this implementation guide (computer friendly)
   */
  _name?: Element
  /** 
   * Name for this implementation guide (human friendly)
   */
  title?: string;
  /** 
   * Name for this implementation guide (human friendly)
   */
  _title?: Element
  /** 
   * draft | active | retired | unknown
   */
  status: code;
  /** 
   * draft | active | retired | unknown
   */
  _status?: Element
  /** 
   * For testing purposes, not real usage
   */
  experimental?: boolean;
  /** 
   * For testing purposes, not real usage
   */
  _experimental?: Element
  /** 
   * Date last changed
   */
  date?: dateTime;
  /** 
   * Date last changed
   */
  _date?: Element
  /** 
   * Name of the publisher (organization or individual)
   */
  publisher?: string;
  /** 
   * Name of the publisher (organization or individual)
   */
  _publisher?: Element
  /** 
   * Contact details for the publisher
   */
  contact?: Array<ContactDetail>;
  /** 
   * Natural language description of the implementation guide
   */
  description?: markdown;
  /** 
   * Natural language description of the implementation guide
   */
  _description?: Element
  /** 
   * The context that the content is intended to support
   */
  useContext?: Array<UsageContext>;
  /** 
   * Intended jurisdiction for implementation guide (if applicable)
   */
  jurisdiction?: Array<CodeableConcept>;
  /** 
   * Use and/or publishing restrictions
   */
  copyright?: markdown;
  /** 
   * Use and/or publishing restrictions
   */
  _copyright?: Element
  /** 
   * NPM Package name for IG
   */
  packageId: id;
  /** 
   * NPM Package name for IG
   */
  _packageId?: Element
  /** 
   * SPDX license code for this IG (or not-open-source)
   */
  license?: code;
  /** 
   * SPDX license code for this IG (or not-open-source)
   */
  _license?: Element
  /** 
   * FHIR Version(s) this Implementation Guide targets
   */
  fhirVersion: Array<code>;
  /** 
   * FHIR Version(s) this Implementation Guide targets
   */
  _fhirVersion?: Array<Element>
  /** 
   * Another Implementation guide this depends on
   */
  dependsOn?: Array<ImplementationGuideDependsOn>;
  /** 
   * Profiles that apply globally
   */
  global?: Array<ImplementationGuideGlobal>;
  /** 
   * Information needed to build the IG
   */
  definition?: ImplementationGuideDefinition;
  /** 
   * Information about an assembled IG
   */
  manifest?: ImplementationGuideManifest;
}

export interface IngredientManufacturer {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * allowed | possible | actual
   */
  role?: code;
  /** 
   * allowed | possible | actual
   */
  _role?: Element
  /** 
   * An organization that manufactures this ingredient
   */
  manufacturer: Reference;
}
export interface IngredientSubstanceStrengthReferenceStrength {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Relevant reference substance
   */
  substance?: CodeableReference;
  /** 
   * Strength expressed in terms of a reference substance
   */
  strengthRatio?: Ratio;
  /** 
   * Strength expressed in terms of a reference substance
   */
  strengthRatioRange?: RatioRange;
  /** 
   * When strength is measured at a particular point or distance
   */
  measurementPoint?: string;
  /** 
   * When strength is measured at a particular point or distance
   */
  _measurementPoint?: Element
  /** 
   * Where the strength range applies
   */
  country?: Array<CodeableConcept>;
}
export interface IngredientSubstanceStrength {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The quantity of substance in the unit of presentation
   */
  presentationRatio?: Ratio;
  /** 
   * The quantity of substance in the unit of presentation
   */
  presentationRatioRange?: RatioRange;
  /** 
   * Text of either the whole presentation strength or a part of it (rest being in Strength.presentation as a ratio)
   */
  textPresentation?: string;
  /** 
   * Text of either the whole presentation strength or a part of it (rest being in Strength.presentation as a ratio)
   */
  _textPresentation?: Element
  /** 
   * The strength per unitary volume (or mass)
   */
  concentrationRatio?: Ratio;
  /** 
   * The strength per unitary volume (or mass)
   */
  concentrationRatioRange?: RatioRange;
  /** 
   * Text of either the whole concentration strength or a part of it (rest being in Strength.concentration as a ratio)
   */
  textConcentration?: string;
  /** 
   * Text of either the whole concentration strength or a part of it (rest being in Strength.concentration as a ratio)
   */
  _textConcentration?: Element
  /** 
   * When strength is measured at a particular point or distance
   */
  measurementPoint?: string;
  /** 
   * When strength is measured at a particular point or distance
   */
  _measurementPoint?: Element
  /** 
   * Where the strength range applies
   */
  country?: Array<CodeableConcept>;
  /** 
   * Strength expressed in terms of a reference substance
   */
  referenceStrength?: Array<IngredientSubstanceStrengthReferenceStrength>;
}
export interface IngredientSubstance {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * A code or full resource that represents the ingredient substance
   */
  code: CodeableReference;
  /** 
   * The quantity of substance, per presentation, or per volume or mass, and type of quantity
   */
  strength?: Array<IngredientSubstanceStrength>;
}
export interface Ingredient {
resourceType: "Ingredient"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * An identifier or code by which the ingredient can be referenced
   */
  identifier?: Identifier;
  /** 
   * draft | active | retired | unknown
   */
  status: code;
  /** 
   * draft | active | retired | unknown
   */
  _status?: Element
  /** 
   * The product which this ingredient is a constituent part of
   */
  for?: Array<Reference>;
  /** 
   * Purpose of the ingredient within the product, e.g. active, inactive
   */
  role: CodeableConcept;
  /** 
   * Precise action within the drug product, e.g. antioxidant, alkalizing agent
   */
  function?: Array<CodeableConcept>;
  /** 
   * If the ingredient is a known or suspected allergen
   */
  allergenicIndicator?: boolean;
  /** 
   * If the ingredient is a known or suspected allergen
   */
  _allergenicIndicator?: Element
  /** 
   * An organization that manufactures this ingredient
   */
  manufacturer?: Array<IngredientManufacturer>;
  /** 
   * The substance that comprises this ingredient
   */
  substance: IngredientSubstance;
}

export interface InsurancePlanContact {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The type of contact
   */
  purpose?: CodeableConcept;
  /** 
   * A name associated with the contact
   */
  name?: HumanName;
  /** 
   * Contact details (telephone, email, etc.)  for a contact
   */
  telecom?: Array<ContactPoint>;
  /** 
   * Visiting or postal addresses for the contact
   */
  address?: Address;
}
export interface InsurancePlanCoverageBenefitLimit {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Maximum value allowed
   */
  value?: Quantity;
  /** 
   * Benefit limit details
   */
  code?: CodeableConcept;
}
export interface InsurancePlanCoverageBenefit {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Type of benefit
   */
  type: CodeableConcept;
  /** 
   * Referral requirements
   */
  requirement?: string;
  /** 
   * Referral requirements
   */
  _requirement?: Element
  /** 
   * Benefit limits
   */
  limit?: Array<InsurancePlanCoverageBenefitLimit>;
}
export interface InsurancePlanCoverage {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Type of coverage
   */
  type: CodeableConcept;
  /** 
   * What networks provide coverage
   */
  network?: Array<Reference>;
  /** 
   * List of benefits
   */
  benefit: Array<InsurancePlanCoverageBenefit>;
}
export interface InsurancePlanPlanGeneralCost {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Type of cost
   */
  type?: CodeableConcept;
  /** 
   * Number of enrollees
   */
  groupSize?: positiveInt;
  /** 
   * Number of enrollees
   */
  _groupSize?: Element
  /** 
   * Cost value
   */
  cost?: Money;
  /** 
   * Additional cost information
   */
  comment?: string;
  /** 
   * Additional cost information
   */
  _comment?: Element
}
export interface InsurancePlanPlanSpecificCostBenefitCost {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Type of cost
   */
  type: CodeableConcept;
  /** 
   * in-network | out-of-network | other
   */
  applicability?: CodeableConcept;
  /** 
   * Additional information about the cost
   */
  qualifiers?: Array<CodeableConcept>;
  /** 
   * The actual cost value
   */
  value?: Quantity;
}
export interface InsurancePlanPlanSpecificCostBenefit {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Type of specific benefit
   */
  type: CodeableConcept;
  /** 
   * List of the costs
   */
  cost?: Array<InsurancePlanPlanSpecificCostBenefitCost>;
}
export interface InsurancePlanPlanSpecificCost {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * General category of benefit
   */
  category: CodeableConcept;
  /** 
   * Benefits list
   */
  benefit?: Array<InsurancePlanPlanSpecificCostBenefit>;
}
export interface InsurancePlanPlan {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business Identifier for Product
   */
  identifier?: Array<Identifier>;
  /** 
   * Type of plan
   */
  type?: CodeableConcept;
  /** 
   * Where product applies
   */
  coverageArea?: Array<Reference>;
  /** 
   * What networks provide coverage
   */
  network?: Array<Reference>;
  /** 
   * Overall costs
   */
  generalCost?: Array<InsurancePlanPlanGeneralCost>;
  /** 
   * Specific costs
   */
  specificCost?: Array<InsurancePlanPlanSpecificCost>;
}
export interface InsurancePlan {
resourceType: "InsurancePlan"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business Identifier for Product
   */
  identifier?: Array<Identifier>;
  /** 
   * draft | active | retired | unknown
   */
  status?: code;
  /** 
   * draft | active | retired | unknown
   */
  _status?: Element
  /** 
   * Kind of product
   */
  type?: Array<CodeableConcept>;
  /** 
   * Official name
   */
  name?: string;
  /** 
   * Official name
   */
  _name?: Element
  /** 
   * Alternate names
   */
  alias?: Array<string>;
  /** 
   * Alternate names
   */
  _alias?: Array<Element>
  /** 
   * When the product is available
   */
  period?: Period;
  /** 
   * Plan issuer
   */
  ownedBy?: Reference;
  /** 
   * Product administrator
   */
  administeredBy?: Reference;
  /** 
   * Where product applies
   */
  coverageArea?: Array<Reference>;
  /** 
   * Contact for the product
   */
  contact?: Array<InsurancePlanContact>;
  /** 
   * Technical endpoint
   */
  endpoint?: Array<Reference>;
  /** 
   * What networks are Included
   */
  network?: Array<Reference>;
  /** 
   * Coverage details
   */
  coverage?: Array<InsurancePlanCoverage>;
  /** 
   * Plan details
   */
  plan?: Array<InsurancePlanPlan>;
}

export interface InvoiceParticipant {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Type of involvement in creation of this Invoice
   */
  role?: CodeableConcept;
  /** 
   * Individual who was involved
   */
  actor: Reference;
}
export interface InvoiceLineItemPriceComponent {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * base | surcharge | deduction | discount | tax | informational
   */
  type: code;
  /** 
   * base | surcharge | deduction | discount | tax | informational
   */
  _type?: Element
  /** 
   * Code identifying the specific component
   */
  code?: CodeableConcept;
  /** 
   * Factor used for calculating this component
   */
  factor?: decimal;
  /** 
   * Factor used for calculating this component
   */
  _factor?: Element
  /** 
   * Monetary amount associated with this component
   */
  amount?: Money;
}
export interface InvoiceLineItem {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Sequence number of line item
   */
  sequence?: positiveInt;
  /** 
   * Sequence number of line item
   */
  _sequence?: Element
  /** 
   * Reference to ChargeItem containing details of this line item or an inline billing code
   */
  chargeItemReference?: Reference;
  /** 
   * Reference to ChargeItem containing details of this line item or an inline billing code
   */
  chargeItemCodeableConcept?: CodeableConcept;
  /** 
   * Components of total line item price
   */
  priceComponent?: Array<InvoiceLineItemPriceComponent>;
}
export interface Invoice {
resourceType: "Invoice"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business Identifier for item
   */
  identifier?: Array<Identifier>;
  /** 
   * draft | issued | balanced | cancelled | entered-in-error
   */
  status: code;
  /** 
   * draft | issued | balanced | cancelled | entered-in-error
   */
  _status?: Element
  /** 
   * Reason for cancellation of this Invoice
   */
  cancelledReason?: string;
  /** 
   * Reason for cancellation of this Invoice
   */
  _cancelledReason?: Element
  /** 
   * Type of Invoice
   */
  type?: CodeableConcept;
  /** 
   * Recipient(s) of goods and services
   */
  subject?: Reference;
  /** 
   * Recipient of this invoice
   */
  recipient?: Reference;
  /** 
   * Invoice date / posting date
   */
  date?: dateTime;
  /** 
   * Invoice date / posting date
   */
  _date?: Element
  /** 
   * Participant in creation of this Invoice
   */
  participant?: Array<InvoiceParticipant>;
  /** 
   * Issuing Organization of Invoice
   */
  issuer?: Reference;
  /** 
   * Account that is being balanced
   */
  account?: Reference;
  /** 
   * Line items of this Invoice
   */
  lineItem?: Array<InvoiceLineItem>;
  /** 
   * Components of Invoice total
   */
  totalPriceComponent?: Array<InvoiceLineItemPriceComponent>;
  /** 
   * Net total of this Invoice
   */
  totalNet?: Money;
  /** 
   * Gross total of this Invoice
   */
  totalGross?: Money;
  /** 
   * Payment details
   */
  paymentTerms?: markdown;
  /** 
   * Payment details
   */
  _paymentTerms?: Element
  /** 
   * Comments made about the invoice
   */
  note?: Array<Annotation>;
}

export interface Library {
resourceType: "Library"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Canonical identifier for this library, represented as a URI (globally unique)
   */
  url?: uri;
  /** 
   * Canonical identifier for this library, represented as a URI (globally unique)
   */
  _url?: Element
  /** 
   * Additional identifier for the library
   */
  identifier?: Array<Identifier>;
  /** 
   * Business version of the library
   */
  version?: string;
  /** 
   * Business version of the library
   */
  _version?: Element
  /** 
   * Name for this library (computer friendly)
   */
  name?: string;
  /** 
   * Name for this library (computer friendly)
   */
  _name?: Element
  /** 
   * Name for this library (human friendly)
   */
  title?: string;
  /** 
   * Name for this library (human friendly)
   */
  _title?: Element
  /** 
   * Subordinate title of the library
   */
  subtitle?: string;
  /** 
   * Subordinate title of the library
   */
  _subtitle?: Element
  /** 
   * draft | active | retired | unknown
   */
  status: code;
  /** 
   * draft | active | retired | unknown
   */
  _status?: Element
  /** 
   * For testing purposes, not real usage
   */
  experimental?: boolean;
  /** 
   * For testing purposes, not real usage
   */
  _experimental?: Element
  /** 
   * logic-library | model-definition | asset-collection | module-definition
   */
  type: CodeableConcept;
  /** 
   * Type of individual the library content is focused on
   */
  subjectCodeableConcept?: CodeableConcept;
  /** 
   * Type of individual the library content is focused on
   */
  subjectReference?: Reference;
  /** 
   * Date last changed
   */
  date?: dateTime;
  /** 
   * Date last changed
   */
  _date?: Element
  /** 
   * Name of the publisher (organization or individual)
   */
  publisher?: string;
  /** 
   * Name of the publisher (organization or individual)
   */
  _publisher?: Element
  /** 
   * Contact details for the publisher
   */
  contact?: Array<ContactDetail>;
  /** 
   * Natural language description of the library
   */
  description?: markdown;
  /** 
   * Natural language description of the library
   */
  _description?: Element
  /** 
   * The context that the content is intended to support
   */
  useContext?: Array<UsageContext>;
  /** 
   * Intended jurisdiction for library (if applicable)
   */
  jurisdiction?: Array<CodeableConcept>;
  /** 
   * Why this library is defined
   */
  purpose?: markdown;
  /** 
   * Why this library is defined
   */
  _purpose?: Element
  /** 
   * Describes the clinical usage of the library
   */
  usage?: string;
  /** 
   * Describes the clinical usage of the library
   */
  _usage?: Element
  /** 
   * Use and/or publishing restrictions
   */
  copyright?: markdown;
  /** 
   * Use and/or publishing restrictions
   */
  _copyright?: Element
  /** 
   * When the library was approved by publisher
   */
  approvalDate?: date;
  /** 
   * When the library was approved by publisher
   */
  _approvalDate?: Element
  /** 
   * When the library was last reviewed
   */
  lastReviewDate?: date;
  /** 
   * When the library was last reviewed
   */
  _lastReviewDate?: Element
  /** 
   * When the library is expected to be used
   */
  effectivePeriod?: Period;
  /** 
   * E.g. Education, Treatment, Assessment, etc.
   */
  topic?: Array<CodeableConcept>;
  /** 
   * Who authored the content
   */
  author?: Array<ContactDetail>;
  /** 
   * Who edited the content
   */
  editor?: Array<ContactDetail>;
  /** 
   * Who reviewed the content
   */
  reviewer?: Array<ContactDetail>;
  /** 
   * Who endorsed the content
   */
  endorser?: Array<ContactDetail>;
  /** 
   * Additional documentation, citations, etc.
   */
  relatedArtifact?: Array<RelatedArtifact>;
  /** 
   * Parameters defined by the library
   */
  parameter?: Array<ParameterDefinition>;
  /** 
   * What data is referenced by this library
   */
  dataRequirement?: Array<DataRequirement>;
  /** 
   * Contents of the library, either embedded or referenced
   */
  content?: Array<Attachment>;
}

export interface LinkageItem {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * source | alternate | historical
   */
  type: code;
  /** 
   * source | alternate | historical
   */
  _type?: Element
  /** 
   * Resource being linked
   */
  resource: Reference;
}
export interface Linkage {
resourceType: "Linkage"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Whether this linkage assertion is active or not
   */
  active?: boolean;
  /** 
   * Whether this linkage assertion is active or not
   */
  _active?: Element
  /** 
   * Who is responsible for linkages
   */
  author?: Reference;
  /** 
   * Item to be linked
   */
  item: Array<LinkageItem>;
}

export interface ListEntry {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Status/Workflow information about this item
   */
  flag?: CodeableConcept;
  /** 
   * If this item is actually marked as deleted
   */
  deleted?: boolean;
  /** 
   * If this item is actually marked as deleted
   */
  _deleted?: Element
  /** 
   * When item added to list
   */
  date?: dateTime;
  /** 
   * When item added to list
   */
  _date?: Element
  /** 
   * Actual entry
   */
  item: Reference;
}
export interface List {
resourceType: "List"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business identifier
   */
  identifier?: Array<Identifier>;
  /** 
   * current | retired | entered-in-error
   */
  status: code;
  /** 
   * current | retired | entered-in-error
   */
  _status?: Element
  /** 
   * working | snapshot | changes
   */
  mode: code;
  /** 
   * working | snapshot | changes
   */
  _mode?: Element
  /** 
   * Descriptive name for the list
   */
  title?: string;
  /** 
   * Descriptive name for the list
   */
  _title?: Element
  /** 
   * What the purpose of this list is
   */
  code?: CodeableConcept;
  /** 
   * If all resources have the same subject
   */
  subject?: Reference;
  /** 
   * Context in which list created
   */
  encounter?: Reference;
  /** 
   * When the list was prepared
   */
  date?: dateTime;
  /** 
   * When the list was prepared
   */
  _date?: Element
  /** 
   * Who and/or what defined the list contents (aka Author)
   */
  source?: Reference;
  /** 
   * What order the list has
   */
  orderedBy?: CodeableConcept;
  /** 
   * Comments about the list
   */
  note?: Array<Annotation>;
  /** 
   * Entries in the list
   */
  entry?: Array<ListEntry>;
  /** 
   * Why list is empty
   */
  emptyReason?: CodeableConcept;
}

export interface LocationPosition {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Longitude with WGS84 datum
   */
  longitude: decimal;
  /** 
   * Longitude with WGS84 datum
   */
  _longitude?: Element
  /** 
   * Latitude with WGS84 datum
   */
  latitude: decimal;
  /** 
   * Latitude with WGS84 datum
   */
  _latitude?: Element
  /** 
   * Altitude with WGS84 datum
   */
  altitude?: decimal;
  /** 
   * Altitude with WGS84 datum
   */
  _altitude?: Element
}
export interface LocationHoursOfOperation {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * mon | tue | wed | thu | fri | sat | sun
   */
  daysOfWeek?: Array<code>;
  /** 
   * mon | tue | wed | thu | fri | sat | sun
   */
  _daysOfWeek?: Array<Element>
  /** 
   * The Location is open all day
   */
  allDay?: boolean;
  /** 
   * The Location is open all day
   */
  _allDay?: Element
  /** 
   * Time that the Location opens
   */
  openingTime?: time;
  /** 
   * Time that the Location opens
   */
  _openingTime?: Element
  /** 
   * Time that the Location closes
   */
  closingTime?: time;
  /** 
   * Time that the Location closes
   */
  _closingTime?: Element
}
export interface Location {
resourceType: "Location"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Unique code or number identifying the location to its users
   */
  identifier?: Array<Identifier>;
  /** 
   * active | suspended | inactive
   */
  status?: code;
  /** 
   * active | suspended | inactive
   */
  _status?: Element
  /** 
   * The operational status of the location (typically only for a bed/room)
   */
  operationalStatus?: Coding;
  /** 
   * Name of the location as used by humans
   */
  name?: string;
  /** 
   * Name of the location as used by humans
   */
  _name?: Element
  /** 
   * A list of alternate names that the location is known as, or was known as, in the past
   */
  alias?: Array<string>;
  /** 
   * A list of alternate names that the location is known as, or was known as, in the past
   */
  _alias?: Array<Element>
  /** 
   * Additional details about the location that could be displayed as further information to identify the location beyond its name
   */
  description?: string;
  /** 
   * Additional details about the location that could be displayed as further information to identify the location beyond its name
   */
  _description?: Element
  /** 
   * instance | kind
   */
  mode?: code;
  /** 
   * instance | kind
   */
  _mode?: Element
  /** 
   * Type of function performed
   */
  type?: Array<CodeableConcept>;
  /** 
   * Contact details of the location
   */
  telecom?: Array<ContactPoint>;
  /** 
   * Physical location
   */
  address?: Address;
  /** 
   * Physical form of the location
   */
  physicalType?: CodeableConcept;
  /** 
   * The absolute geographic location
   */
  position?: LocationPosition;
  /** 
   * Organization responsible for provisioning and upkeep
   */
  managingOrganization?: Reference;
  /** 
   * Another Location this one is physically a part of
   */
  partOf?: Reference;
  /** 
   * What days/times during a week is this location usually open
   */
  hoursOfOperation?: Array<LocationHoursOfOperation>;
  /** 
   * Description of availability exceptions
   */
  availabilityExceptions?: string;
  /** 
   * Description of availability exceptions
   */
  _availabilityExceptions?: Element
  /** 
   * Technical endpoints providing access to services operated for the location
   */
  endpoint?: Array<Reference>;
}

export interface ManufacturedItemDefinitionProperty {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * A code expressing the type of characteristic
   */
  type: CodeableConcept;
  /** 
   * A value for the characteristic
   */
  valueCodeableConcept?: CodeableConcept;
  /** 
   * A value for the characteristic
   */
  valueQuantity?: Quantity;
  /** 
   * A value for the characteristic
   */
  valueDate?: date;
  /** 
   * A value for the characteristic
   */
  _valueDate?: Element
  /** 
   * A value for the characteristic
   */
  valueBoolean?: boolean;
  /** 
   * A value for the characteristic
   */
  _valueBoolean?: Element
  /** 
   * A value for the characteristic
   */
  valueAttachment?: Attachment;
}
export interface ManufacturedItemDefinition {
resourceType: "ManufacturedItemDefinition"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Unique identifier
   */
  identifier?: Array<Identifier>;
  /** 
   * draft | active | retired | unknown
   */
  status: code;
  /** 
   * draft | active | retired | unknown
   */
  _status?: Element
  /** 
   * Dose form as manufactured (before any necessary transformation)
   */
  manufacturedDoseForm: CodeableConcept;
  /** 
   * The “real world” units in which the quantity of the item is described
   */
  unitOfPresentation?: CodeableConcept;
  /** 
   * Manufacturer of the item (Note that this should be named "manufacturer" but it currently causes technical issues)
   */
  manufacturer?: Array<Reference>;
  /** 
   * The ingredients of this manufactured item. Only needed if these are not specified by incoming references from the Ingredient resource
   */
  ingredient?: Array<CodeableConcept>;
  /** 
   * General characteristics of this item
   */
  property?: Array<ManufacturedItemDefinitionProperty>;
}

export interface MeasureGroupPopulation {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * initial-population | numerator | numerator-exclusion | denominator | denominator-exclusion | denominator-exception | measure-population | measure-population-exclusion | measure-observation
   */
  code?: CodeableConcept;
  /** 
   * The human readable description of this population criteria
   */
  description?: string;
  /** 
   * The human readable description of this population criteria
   */
  _description?: Element
  /** 
   * The criteria that defines this population
   */
  criteria: Expression;
}
export interface MeasureGroupStratifierComponent {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Meaning of the stratifier component
   */
  code?: CodeableConcept;
  /** 
   * The human readable description of this stratifier component
   */
  description?: string;
  /** 
   * The human readable description of this stratifier component
   */
  _description?: Element
  /** 
   * Component of how the measure should be stratified
   */
  criteria: Expression;
}
export interface MeasureGroupStratifier {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Meaning of the stratifier
   */
  code?: CodeableConcept;
  /** 
   * The human readable description of this stratifier
   */
  description?: string;
  /** 
   * The human readable description of this stratifier
   */
  _description?: Element
  /** 
   * How the measure should be stratified
   */
  criteria?: Expression;
  /** 
   * Stratifier criteria component for the measure
   */
  component?: Array<MeasureGroupStratifierComponent>;
}
export interface MeasureGroup {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Meaning of the group
   */
  code?: CodeableConcept;
  /** 
   * Summary description
   */
  description?: string;
  /** 
   * Summary description
   */
  _description?: Element
  /** 
   * Population criteria
   */
  population?: Array<MeasureGroupPopulation>;
  /** 
   * Stratifier criteria for the measure
   */
  stratifier?: Array<MeasureGroupStratifier>;
}
export interface MeasureSupplementalData {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Meaning of the supplemental data
   */
  code?: CodeableConcept;
  /** 
   * supplemental-data | risk-adjustment-factor
   */
  usage?: Array<CodeableConcept>;
  /** 
   * The human readable description of this supplemental data
   */
  description?: string;
  /** 
   * The human readable description of this supplemental data
   */
  _description?: Element
  /** 
   * Expression describing additional data to be reported
   */
  criteria: Expression;
}
export interface Measure {
resourceType: "Measure"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Canonical identifier for this measure, represented as a URI (globally unique)
   */
  url?: uri;
  /** 
   * Canonical identifier for this measure, represented as a URI (globally unique)
   */
  _url?: Element
  /** 
   * Additional identifier for the measure
   */
  identifier?: Array<Identifier>;
  /** 
   * Business version of the measure
   */
  version?: string;
  /** 
   * Business version of the measure
   */
  _version?: Element
  /** 
   * Name for this measure (computer friendly)
   */
  name?: string;
  /** 
   * Name for this measure (computer friendly)
   */
  _name?: Element
  /** 
   * Name for this measure (human friendly)
   */
  title?: string;
  /** 
   * Name for this measure (human friendly)
   */
  _title?: Element
  /** 
   * Subordinate title of the measure
   */
  subtitle?: string;
  /** 
   * Subordinate title of the measure
   */
  _subtitle?: Element
  /** 
   * draft | active | retired | unknown
   */
  status: code;
  /** 
   * draft | active | retired | unknown
   */
  _status?: Element
  /** 
   * For testing purposes, not real usage
   */
  experimental?: boolean;
  /** 
   * For testing purposes, not real usage
   */
  _experimental?: Element
  /** 
   * E.g. Patient, Practitioner, RelatedPerson, Organization, Location, Device
   */
  subjectCodeableConcept?: CodeableConcept;
  /** 
   * E.g. Patient, Practitioner, RelatedPerson, Organization, Location, Device
   */
  subjectReference?: Reference;
  /** 
   * Date last changed
   */
  date?: dateTime;
  /** 
   * Date last changed
   */
  _date?: Element
  /** 
   * Name of the publisher (organization or individual)
   */
  publisher?: string;
  /** 
   * Name of the publisher (organization or individual)
   */
  _publisher?: Element
  /** 
   * Contact details for the publisher
   */
  contact?: Array<ContactDetail>;
  /** 
   * Natural language description of the measure
   */
  description?: markdown;
  /** 
   * Natural language description of the measure
   */
  _description?: Element
  /** 
   * The context that the content is intended to support
   */
  useContext?: Array<UsageContext>;
  /** 
   * Intended jurisdiction for measure (if applicable)
   */
  jurisdiction?: Array<CodeableConcept>;
  /** 
   * Why this measure is defined
   */
  purpose?: markdown;
  /** 
   * Why this measure is defined
   */
  _purpose?: Element
  /** 
   * Describes the clinical usage of the measure
   */
  usage?: string;
  /** 
   * Describes the clinical usage of the measure
   */
  _usage?: Element
  /** 
   * Use and/or publishing restrictions
   */
  copyright?: markdown;
  /** 
   * Use and/or publishing restrictions
   */
  _copyright?: Element
  /** 
   * When the measure was approved by publisher
   */
  approvalDate?: date;
  /** 
   * When the measure was approved by publisher
   */
  _approvalDate?: Element
  /** 
   * When the measure was last reviewed
   */
  lastReviewDate?: date;
  /** 
   * When the measure was last reviewed
   */
  _lastReviewDate?: Element
  /** 
   * When the measure is expected to be used
   */
  effectivePeriod?: Period;
  /** 
   * The category of the measure, such as Education, Treatment, Assessment, etc.
   */
  topic?: Array<CodeableConcept>;
  /** 
   * Who authored the content
   */
  author?: Array<ContactDetail>;
  /** 
   * Who edited the content
   */
  editor?: Array<ContactDetail>;
  /** 
   * Who reviewed the content
   */
  reviewer?: Array<ContactDetail>;
  /** 
   * Who endorsed the content
   */
  endorser?: Array<ContactDetail>;
  /** 
   * Additional documentation, citations, etc.
   */
  relatedArtifact?: Array<RelatedArtifact>;
  /** 
   * Logic used by the measure
   */
  library?: Array<canonical>;
  /** 
   * Logic used by the measure
   */
  _library?: Array<Element>
  /** 
   * Disclaimer for use of the measure or its referenced content
   */
  disclaimer?: markdown;
  /** 
   * Disclaimer for use of the measure or its referenced content
   */
  _disclaimer?: Element
  /** 
   * proportion | ratio | continuous-variable | cohort
   */
  scoring?: CodeableConcept;
  /** 
   * opportunity | all-or-nothing | linear | weighted
   */
  compositeScoring?: CodeableConcept;
  /** 
   * process | outcome | structure | patient-reported-outcome | composite
   */
  type?: Array<CodeableConcept>;
  /** 
   * How risk adjustment is applied for this measure
   */
  riskAdjustment?: string;
  /** 
   * How risk adjustment is applied for this measure
   */
  _riskAdjustment?: Element
  /** 
   * How is rate aggregation performed for this measure
   */
  rateAggregation?: string;
  /** 
   * How is rate aggregation performed for this measure
   */
  _rateAggregation?: Element
  /** 
   * Detailed description of why the measure exists
   */
  rationale?: markdown;
  /** 
   * Detailed description of why the measure exists
   */
  _rationale?: Element
  /** 
   * Summary of clinical guidelines
   */
  clinicalRecommendationStatement?: markdown;
  /** 
   * Summary of clinical guidelines
   */
  _clinicalRecommendationStatement?: Element
  /** 
   * increase | decrease
   */
  improvementNotation?: CodeableConcept;
  /** 
   * Defined terms used in the measure documentation
   */
  definition?: Array<markdown>;
  /** 
   * Defined terms used in the measure documentation
   */
  _definition?: Array<Element>
  /** 
   * Additional guidance for implementers
   */
  guidance?: markdown;
  /** 
   * Additional guidance for implementers
   */
  _guidance?: Element
  /** 
   * Population criteria group
   */
  group?: Array<MeasureGroup>;
  /** 
   * What other data should be reported with the measure
   */
  supplementalData?: Array<MeasureSupplementalData>;
}

export interface MeasureReportGroupPopulation {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * initial-population | numerator | numerator-exclusion | denominator | denominator-exclusion | denominator-exception | measure-population | measure-population-exclusion | measure-observation
   */
  code?: CodeableConcept;
  /** 
   * Size of the population
   */
  count?: integer;
  /** 
   * Size of the population
   */
  _count?: Element
  /** 
   * For subject-list reports, the subject results in this population
   */
  subjectResults?: Reference;
}
export interface MeasureReportGroupStratifierStratumComponent {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * What stratifier component of the group
   */
  code: CodeableConcept;
  /** 
   * The stratum component value, e.g. male
   */
  value: CodeableConcept;
}
export interface MeasureReportGroupStratifierStratumPopulation {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * initial-population | numerator | numerator-exclusion | denominator | denominator-exclusion | denominator-exception | measure-population | measure-population-exclusion | measure-observation
   */
  code?: CodeableConcept;
  /** 
   * Size of the population
   */
  count?: integer;
  /** 
   * Size of the population
   */
  _count?: Element
  /** 
   * For subject-list reports, the subject results in this population
   */
  subjectResults?: Reference;
}
export interface MeasureReportGroupStratifierStratum {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The stratum value, e.g. male
   */
  value?: CodeableConcept;
  /** 
   * Stratifier component values
   */
  component?: Array<MeasureReportGroupStratifierStratumComponent>;
  /** 
   * Population results in this stratum
   */
  population?: Array<MeasureReportGroupStratifierStratumPopulation>;
  /** 
   * What score this stratum achieved
   */
  measureScore?: Quantity;
}
export interface MeasureReportGroupStratifier {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * What stratifier of the group
   */
  code?: Array<CodeableConcept>;
  /** 
   * Stratum results, one for each unique value, or set of values, in the stratifier, or stratifier components
   */
  stratum?: Array<MeasureReportGroupStratifierStratum>;
}
export interface MeasureReportGroup {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Meaning of the group
   */
  code?: CodeableConcept;
  /** 
   * The populations in the group
   */
  population?: Array<MeasureReportGroupPopulation>;
  /** 
   * What score this group achieved
   */
  measureScore?: Quantity;
  /** 
   * Stratification results
   */
  stratifier?: Array<MeasureReportGroupStratifier>;
}
export interface MeasureReport {
resourceType: "MeasureReport"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Additional identifier for the MeasureReport
   */
  identifier?: Array<Identifier>;
  /** 
   * complete | pending | error
   */
  status: code;
  /** 
   * complete | pending | error
   */
  _status?: Element
  /** 
   * individual | subject-list | summary | data-collection
   */
  type: code;
  /** 
   * individual | subject-list | summary | data-collection
   */
  _type?: Element
  /** 
   * What measure was calculated
   */
  measure: canonical;
  /** 
   * What measure was calculated
   */
  _measure?: Element
  /** 
   * What individual(s) the report is for
   */
  subject?: Reference;
  /** 
   * When the report was generated
   */
  date?: dateTime;
  /** 
   * When the report was generated
   */
  _date?: Element
  /** 
   * Who is reporting the data
   */
  reporter?: Reference;
  /** 
   * What period the report covers
   */
  period: Period;
  /** 
   * increase | decrease
   */
  improvementNotation?: CodeableConcept;
  /** 
   * Measure results for each group
   */
  group?: Array<MeasureReportGroup>;
  /** 
   * What data was used to calculate the measure score
   */
  evaluatedResource?: Array<Reference>;
}

export interface Media {
resourceType: "Media"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Identifier(s) for the image
   */
  identifier?: Array<Identifier>;
  /** 
   * Procedure that caused this media to be created
   */
  basedOn?: Array<Reference>;
  /** 
   * Part of referenced event
   */
  partOf?: Array<Reference>;
  /** 
   * preparation | in-progress | not-done | on-hold | stopped | completed | entered-in-error | unknown
   */
  status: code;
  /** 
   * preparation | in-progress | not-done | on-hold | stopped | completed | entered-in-error | unknown
   */
  _status?: Element
  /** 
   * Classification of media as image, video, or audio
   */
  type?: CodeableConcept;
  /** 
   * The type of acquisition equipment/process
   */
  modality?: CodeableConcept;
  /** 
   * Imaging view, e.g. Lateral or Antero-posterior
   */
  view?: CodeableConcept;
  /** 
   * Who/What this Media is a record of
   */
  subject?: Reference;
  /** 
   * Encounter associated with media
   */
  encounter?: Reference;
  /** 
   * When Media was collected
   */
  createdDateTime?: dateTime;
  /** 
   * When Media was collected
   */
  _createdDateTime?: Element
  /** 
   * When Media was collected
   */
  createdPeriod?: Period;
  /** 
   * Date/Time this version was made available
   */
  issued?: instant;
  /** 
   * Date/Time this version was made available
   */
  _issued?: Element
  /** 
   * The person who generated the image
   */
  operator?: Reference;
  /** 
   * Why was event performed?
   */
  reasonCode?: Array<CodeableConcept>;
  /** 
   * Observed body part
   */
  bodySite?: CodeableConcept;
  /** 
   * Name of the device/manufacturer
   */
  deviceName?: string;
  /** 
   * Name of the device/manufacturer
   */
  _deviceName?: Element
  /** 
   * Observing Device
   */
  device?: Reference;
  /** 
   * Height of the image in pixels (photo/video)
   */
  height?: positiveInt;
  /** 
   * Height of the image in pixels (photo/video)
   */
  _height?: Element
  /** 
   * Width of the image in pixels (photo/video)
   */
  width?: positiveInt;
  /** 
   * Width of the image in pixels (photo/video)
   */
  _width?: Element
  /** 
   * Number of frames if > 1 (photo)
   */
  frames?: positiveInt;
  /** 
   * Number of frames if > 1 (photo)
   */
  _frames?: Element
  /** 
   * Length in seconds (audio / video)
   */
  duration?: decimal;
  /** 
   * Length in seconds (audio / video)
   */
  _duration?: Element
  /** 
   * Actual Media - reference or data
   */
  content: Attachment;
  /** 
   * Comments made about the media
   */
  note?: Array<Annotation>;
}

export interface MedicationIngredient {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The actual ingredient or content
   */
  itemCodeableConcept?: CodeableConcept;
  /** 
   * The actual ingredient or content
   */
  itemReference?: Reference;
  /** 
   * Active ingredient indicator
   */
  isActive?: boolean;
  /** 
   * Active ingredient indicator
   */
  _isActive?: Element
  /** 
   * Quantity of ingredient present
   */
  strength?: Ratio;
}
export interface MedicationBatch {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Identifier assigned to batch
   */
  lotNumber?: string;
  /** 
   * Identifier assigned to batch
   */
  _lotNumber?: Element
  /** 
   * When batch will expire
   */
  expirationDate?: dateTime;
  /** 
   * When batch will expire
   */
  _expirationDate?: Element
}
export interface Medication {
resourceType: "Medication"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business identifier for this medication
   */
  identifier?: Array<Identifier>;
  /** 
   * Codes that identify this medication
   */
  code?: CodeableConcept;
  /** 
   * active | inactive | entered-in-error
   */
  status?: code;
  /** 
   * active | inactive | entered-in-error
   */
  _status?: Element
  /** 
   * Manufacturer of the item
   */
  manufacturer?: Reference;
  /** 
   * powder | tablets | capsule +
   */
  form?: CodeableConcept;
  /** 
   * Amount of drug in package
   */
  amount?: Ratio;
  /** 
   * Active or inactive ingredient
   */
  ingredient?: Array<MedicationIngredient>;
  /** 
   * Details about packaged medications
   */
  batch?: MedicationBatch;
}

export interface MedicationAdministrationPerformer {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Type of performance
   */
  function?: CodeableConcept;
  /** 
   * Who performed the medication administration
   */
  actor: Reference;
}
export interface MedicationAdministrationDosage {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Free text dosage instructions e.g. SIG
   */
  text?: string;
  /** 
   * Free text dosage instructions e.g. SIG
   */
  _text?: Element
  /** 
   * Body site administered to
   */
  site?: CodeableConcept;
  /** 
   * Path of substance into body
   */
  route?: CodeableConcept;
  /** 
   * How drug was administered
   */
  method?: CodeableConcept;
  /** 
   * Amount of medication per dose
   */
  dose?: Quantity;
  /** 
   * Dose quantity per unit of time
   */
  rateRatio?: Ratio;
  /** 
   * Dose quantity per unit of time
   */
  rateQuantity?: Quantity;
}
export interface MedicationAdministration {
resourceType: "MedicationAdministration"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * External identifier
   */
  identifier?: Array<Identifier>;
  /** 
   * Instantiates protocol or definition
   */
  instantiates?: Array<uri>;
  /** 
   * Instantiates protocol or definition
   */
  _instantiates?: Array<Element>
  /** 
   * Part of referenced event
   */
  partOf?: Array<Reference>;
  /** 
   * in-progress | not-done | on-hold | completed | entered-in-error | stopped | unknown
   */
  status: code;
  /** 
   * in-progress | not-done | on-hold | completed | entered-in-error | stopped | unknown
   */
  _status?: Element
  /** 
   * Reason administration not performed
   */
  statusReason?: Array<CodeableConcept>;
  /** 
   * Type of medication usage
   */
  category?: CodeableConcept;
  /** 
   * What was administered
   */
  medicationCodeableConcept?: CodeableConcept;
  /** 
   * What was administered
   */
  medicationReference?: Reference;
  /** 
   * Who received medication
   */
  subject: Reference;
  /** 
   * Encounter or Episode of Care administered as part of
   */
  context?: Reference;
  /** 
   * Additional information to support administration
   */
  supportingInformation?: Array<Reference>;
  /** 
   * Start and end time of administration
   */
  effectiveDateTime?: dateTime;
  /** 
   * Start and end time of administration
   */
  _effectiveDateTime?: Element
  /** 
   * Start and end time of administration
   */
  effectivePeriod?: Period;
  /** 
   * Who performed the medication administration and what they did
   */
  performer?: Array<MedicationAdministrationPerformer>;
  /** 
   * Reason administration performed
   */
  reasonCode?: Array<CodeableConcept>;
  /** 
   * Condition or observation that supports why the medication was administered
   */
  reasonReference?: Array<Reference>;
  /** 
   * Request administration performed against
   */
  request?: Reference;
  /** 
   * Device used to administer
   */
  device?: Array<Reference>;
  /** 
   * Information about the administration
   */
  note?: Array<Annotation>;
  /** 
   * Details of how medication was taken
   */
  dosage?: MedicationAdministrationDosage;
  /** 
   * A list of events of interest in the lifecycle
   */
  eventHistory?: Array<Reference>;
}

export interface MedicationDispensePerformer {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Who performed the dispense and what they did
   */
  function?: CodeableConcept;
  /** 
   * Individual who was performing
   */
  actor: Reference;
}
export interface MedicationDispenseSubstitution {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Whether a substitution was or was not performed on the dispense
   */
  wasSubstituted: boolean;
  /** 
   * Whether a substitution was or was not performed on the dispense
   */
  _wasSubstituted?: Element
  /** 
   * Code signifying whether a different drug was dispensed from what was prescribed
   */
  type?: CodeableConcept;
  /** 
   * Why was substitution made
   */
  reason?: Array<CodeableConcept>;
  /** 
   * Who is responsible for the substitution
   */
  responsibleParty?: Array<Reference>;
}
export interface MedicationDispense {
resourceType: "MedicationDispense"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * External identifier
   */
  identifier?: Array<Identifier>;
  /** 
   * Event that dispense is part of
   */
  partOf?: Array<Reference>;
  /** 
   * preparation | in-progress | cancelled | on-hold | completed | entered-in-error | stopped | declined | unknown
   */
  status: code;
  /** 
   * preparation | in-progress | cancelled | on-hold | completed | entered-in-error | stopped | declined | unknown
   */
  _status?: Element
  /** 
   * Why a dispense was not performed
   */
  statusReasonCodeableConcept?: CodeableConcept;
  /** 
   * Why a dispense was not performed
   */
  statusReasonReference?: Reference;
  /** 
   * Type of medication dispense
   */
  category?: CodeableConcept;
  /** 
   * What medication was supplied
   */
  medicationCodeableConcept?: CodeableConcept;
  /** 
   * What medication was supplied
   */
  medicationReference?: Reference;
  /** 
   * Who the dispense is for
   */
  subject?: Reference;
  /** 
   * Encounter / Episode associated with event
   */
  context?: Reference;
  /** 
   * Information that supports the dispensing of the medication
   */
  supportingInformation?: Array<Reference>;
  /** 
   * Who performed event
   */
  performer?: Array<MedicationDispensePerformer>;
  /** 
   * Where the dispense occurred
   */
  location?: Reference;
  /** 
   * Medication order that authorizes the dispense
   */
  authorizingPrescription?: Array<Reference>;
  /** 
   * Trial fill, partial fill, emergency fill, etc.
   */
  type?: CodeableConcept;
  /** 
   * Amount dispensed
   */
  quantity?: Quantity;
  /** 
   * Amount of medication expressed as a timing amount
   */
  daysSupply?: Quantity;
  /** 
   * When product was packaged and reviewed
   */
  whenPrepared?: dateTime;
  /** 
   * When product was packaged and reviewed
   */
  _whenPrepared?: Element
  /** 
   * When product was given out
   */
  whenHandedOver?: dateTime;
  /** 
   * When product was given out
   */
  _whenHandedOver?: Element
  /** 
   * Where the medication was sent
   */
  destination?: Reference;
  /** 
   * Who collected the medication
   */
  receiver?: Array<Reference>;
  /** 
   * Information about the dispense
   */
  note?: Array<Annotation>;
  /** 
   * How the medication is to be used by the patient or administered by the caregiver
   */
  dosageInstruction?: Array<Dosage>;
  /** 
   * Whether a substitution was performed on the dispense
   */
  substitution?: MedicationDispenseSubstitution;
  /** 
   * Clinical issue with action
   */
  detectedIssue?: Array<Reference>;
  /** 
   * A list of relevant lifecycle events
   */
  eventHistory?: Array<Reference>;
}

export interface MedicationKnowledgeRelatedMedicationKnowledge {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Category of medicationKnowledge
   */
  type: CodeableConcept;
  /** 
   * Associated documentation about the associated medication knowledge
   */
  reference: Array<Reference>;
}
export interface MedicationKnowledgeMonograph {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The category of medication document
   */
  type?: CodeableConcept;
  /** 
   * Associated documentation about the medication
   */
  source?: Reference;
}
export interface MedicationKnowledgeIngredient {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Medication(s) or substance(s) contained in the medication
   */
  itemCodeableConcept?: CodeableConcept;
  /** 
   * Medication(s) or substance(s) contained in the medication
   */
  itemReference?: Reference;
  /** 
   * Active ingredient indicator
   */
  isActive?: boolean;
  /** 
   * Active ingredient indicator
   */
  _isActive?: Element
  /** 
   * Quantity of ingredient present
   */
  strength?: Ratio;
}
export interface MedicationKnowledgeCost {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The category of the cost information
   */
  type: CodeableConcept;
  /** 
   * The source or owner for the price information
   */
  source?: string;
  /** 
   * The source or owner for the price information
   */
  _source?: Element
  /** 
   * The price of the medication
   */
  cost: Money;
}
export interface MedicationKnowledgeMonitoringProgram {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Type of program under which the medication is monitored
   */
  type?: CodeableConcept;
  /** 
   * Name of the reviewing program
   */
  name?: string;
  /** 
   * Name of the reviewing program
   */
  _name?: Element
}
export interface MedicationKnowledgeAdministrationGuidelinesDosage {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Type of dosage
   */
  type: CodeableConcept;
  /** 
   * Dosage for the medication for the specific guidelines
   */
  dosage: Array<Dosage>;
}
export interface MedicationKnowledgeAdministrationGuidelinesPatientCharacteristics {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Specific characteristic that is relevant to the administration guideline
   */
  characteristicCodeableConcept?: CodeableConcept;
  /** 
   * Specific characteristic that is relevant to the administration guideline
   */
  characteristicQuantity?: Quantity;
  /** 
   * The specific characteristic
   */
  value?: Array<string>;
  /** 
   * The specific characteristic
   */
  _value?: Array<Element>
}
export interface MedicationKnowledgeAdministrationGuidelines {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Dosage for the medication for the specific guidelines
   */
  dosage?: Array<MedicationKnowledgeAdministrationGuidelinesDosage>;
  /** 
   * Indication for use that apply to the specific administration guidelines
   */
  indicationCodeableConcept?: CodeableConcept;
  /** 
   * Indication for use that apply to the specific administration guidelines
   */
  indicationReference?: Reference;
  /** 
   * Characteristics of the patient that are relevant to the administration guidelines
   */
  patientCharacteristics?: Array<MedicationKnowledgeAdministrationGuidelinesPatientCharacteristics>;
}
export interface MedicationKnowledgeMedicineClassification {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The type of category for the medication (for example, therapeutic classification, therapeutic sub-classification)
   */
  type: CodeableConcept;
  /** 
   * Specific category assigned to the medication
   */
  classification?: Array<CodeableConcept>;
}
export interface MedicationKnowledgePackaging {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * A code that defines the specific type of packaging that the medication can be found in
   */
  type?: CodeableConcept;
  /** 
   * The number of product units the package would contain if fully loaded
   */
  quantity?: Quantity;
}
export interface MedicationKnowledgeDrugCharacteristic {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Code specifying the type of characteristic of medication
   */
  type?: CodeableConcept;
  /** 
   * Description of the characteristic
   */
  valueCodeableConcept?: CodeableConcept;
  /** 
   * Description of the characteristic
   */
  valueString?: string;
  /** 
   * Description of the characteristic
   */
  _valueString?: Element
  /** 
   * Description of the characteristic
   */
  valueQuantity?: Quantity;
  /** 
   * Description of the characteristic
   */
  valueBase64Binary?: base64Binary;
  /** 
   * Description of the characteristic
   */
  _valueBase64Binary?: Element
}
export interface MedicationKnowledgeRegulatorySubstitution {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Specifies the type of substitution allowed
   */
  type: CodeableConcept;
  /** 
   * Specifies if regulation allows for changes in the medication when dispensing
   */
  allowed: boolean;
  /** 
   * Specifies if regulation allows for changes in the medication when dispensing
   */
  _allowed?: Element
}
export interface MedicationKnowledgeRegulatorySchedule {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Specifies the specific drug schedule
   */
  schedule: CodeableConcept;
}
export interface MedicationKnowledgeRegulatoryMaxDispense {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The maximum number of units of the medication that can be dispensed
   */
  quantity: Quantity;
  /** 
   * The period that applies to the maximum number of units
   */
  period?: Duration;
}
export interface MedicationKnowledgeRegulatory {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Specifies the authority of the regulation
   */
  regulatoryAuthority: Reference;
  /** 
   * Specifies if changes are allowed when dispensing a medication from a regulatory perspective
   */
  substitution?: Array<MedicationKnowledgeRegulatorySubstitution>;
  /** 
   * Specifies the schedule of a medication in jurisdiction
   */
  schedule?: Array<MedicationKnowledgeRegulatorySchedule>;
  /** 
   * The maximum number of units of the medication that can be dispensed in a period
   */
  maxDispense?: MedicationKnowledgeRegulatoryMaxDispense;
}
export interface MedicationKnowledgeKinetics {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The drug concentration measured at certain discrete points in time
   */
  areaUnderCurve?: Array<Quantity>;
  /** 
   * The median lethal dose of a drug
   */
  lethalDose50?: Array<Quantity>;
  /** 
   * Time required for concentration in the body to decrease by half
   */
  halfLifePeriod?: Duration;
}
export interface MedicationKnowledge {
resourceType: "MedicationKnowledge"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Code that identifies this medication
   */
  code?: CodeableConcept;
  /** 
   * active | inactive | entered-in-error
   */
  status?: code;
  /** 
   * active | inactive | entered-in-error
   */
  _status?: Element
  /** 
   * Manufacturer of the item
   */
  manufacturer?: Reference;
  /** 
   * powder | tablets | capsule +
   */
  doseForm?: CodeableConcept;
  /** 
   * Amount of drug in package
   */
  amount?: Quantity;
  /** 
   * Additional names for a medication
   */
  synonym?: Array<string>;
  /** 
   * Additional names for a medication
   */
  _synonym?: Array<Element>
  /** 
   * Associated or related medication information
   */
  relatedMedicationKnowledge?: Array<MedicationKnowledgeRelatedMedicationKnowledge>;
  /** 
   * A medication resource that is associated with this medication
   */
  associatedMedication?: Array<Reference>;
  /** 
   * Category of the medication or product
   */
  productType?: Array<CodeableConcept>;
  /** 
   * Associated documentation about the medication
   */
  monograph?: Array<MedicationKnowledgeMonograph>;
  /** 
   * Active or inactive ingredient
   */
  ingredient?: Array<MedicationKnowledgeIngredient>;
  /** 
   * The instructions for preparing the medication
   */
  preparationInstruction?: markdown;
  /** 
   * The instructions for preparing the medication
   */
  _preparationInstruction?: Element
  /** 
   * The intended or approved route of administration
   */
  intendedRoute?: Array<CodeableConcept>;
  /** 
   * The pricing of the medication
   */
  cost?: Array<MedicationKnowledgeCost>;
  /** 
   * Program under which a medication is reviewed
   */
  monitoringProgram?: Array<MedicationKnowledgeMonitoringProgram>;
  /** 
   * Guidelines for administration of the medication
   */
  administrationGuidelines?: Array<MedicationKnowledgeAdministrationGuidelines>;
  /** 
   * Categorization of the medication within a formulary or classification system
   */
  medicineClassification?: Array<MedicationKnowledgeMedicineClassification>;
  /** 
   * Details about packaged medications
   */
  packaging?: MedicationKnowledgePackaging;
  /** 
   * Specifies descriptive properties of the medicine
   */
  drugCharacteristic?: Array<MedicationKnowledgeDrugCharacteristic>;
  /** 
   * Potential clinical issue with or between medication(s)
   */
  contraindication?: Array<Reference>;
  /** 
   * Regulatory information about a medication
   */
  regulatory?: Array<MedicationKnowledgeRegulatory>;
  /** 
   * The time course of drug absorption, distribution, metabolism and excretion of a medication from the body
   */
  kinetics?: Array<MedicationKnowledgeKinetics>;
}

export interface MedicationRequestDispenseRequestInitialFill {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * First fill quantity
   */
  quantity?: Quantity;
  /** 
   * First fill duration
   */
  duration?: Duration;
}
export interface MedicationRequestDispenseRequest {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * First fill details
   */
  initialFill?: MedicationRequestDispenseRequestInitialFill;
  /** 
   * Minimum period of time between dispenses
   */
  dispenseInterval?: Duration;
  /** 
   * Time period supply is authorized for
   */
  validityPeriod?: Period;
  /** 
   * Number of refills authorized
   */
  numberOfRepeatsAllowed?: unsignedInt;
  /** 
   * Number of refills authorized
   */
  _numberOfRepeatsAllowed?: Element
  /** 
   * Amount of medication to supply per dispense
   */
  quantity?: Quantity;
  /** 
   * Number of days supply per dispense
   */
  expectedSupplyDuration?: Duration;
  /** 
   * Intended dispenser
   */
  performer?: Reference;
}
export interface MedicationRequestSubstitution {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Whether substitution is allowed or not
   */
  allowedBoolean?: boolean;
  /** 
   * Whether substitution is allowed or not
   */
  _allowedBoolean?: Element
  /** 
   * Whether substitution is allowed or not
   */
  allowedCodeableConcept?: CodeableConcept;
  /** 
   * Why should (not) substitution be made
   */
  reason?: CodeableConcept;
}
export interface MedicationRequest {
resourceType: "MedicationRequest"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * External ids for this request
   */
  identifier?: Array<Identifier>;
  /** 
   * active | on-hold | cancelled | completed | entered-in-error | stopped | draft | unknown
   */
  status: code;
  /** 
   * active | on-hold | cancelled | completed | entered-in-error | stopped | draft | unknown
   */
  _status?: Element
  /** 
   * Reason for current status
   */
  statusReason?: CodeableConcept;
  /** 
   * proposal | plan | order | original-order | reflex-order | filler-order | instance-order | option
   */
  intent: code;
  /** 
   * proposal | plan | order | original-order | reflex-order | filler-order | instance-order | option
   */
  _intent?: Element
  /** 
   * Type of medication usage
   */
  category?: Array<CodeableConcept>;
  /** 
   * routine | urgent | asap | stat
   */
  priority?: code;
  /** 
   * routine | urgent | asap | stat
   */
  _priority?: Element
  /** 
   * True if request is prohibiting action
   */
  doNotPerform?: boolean;
  /** 
   * True if request is prohibiting action
   */
  _doNotPerform?: Element
  /** 
   * Reported rather than primary record
   */
  reportedBoolean?: boolean;
  /** 
   * Reported rather than primary record
   */
  _reportedBoolean?: Element
  /** 
   * Reported rather than primary record
   */
  reportedReference?: Reference;
  /** 
   * Medication to be taken
   */
  medicationCodeableConcept?: CodeableConcept;
  /** 
   * Medication to be taken
   */
  medicationReference?: Reference;
  /** 
   * Who or group medication request is for
   */
  subject: Reference;
  /** 
   * Encounter created as part of encounter/admission/stay
   */
  encounter?: Reference;
  /** 
   * Information to support ordering of the medication
   */
  supportingInformation?: Array<Reference>;
  /** 
   * When request was initially authored
   */
  authoredOn?: dateTime;
  /** 
   * When request was initially authored
   */
  _authoredOn?: Element
  /** 
   * Who/What requested the Request
   */
  requester?: Reference;
  /** 
   * Intended performer of administration
   */
  performer?: Reference;
  /** 
   * Desired kind of performer of the medication administration
   */
  performerType?: CodeableConcept;
  /** 
   * Person who entered the request
   */
  recorder?: Reference;
  /** 
   * Reason or indication for ordering or not ordering the medication
   */
  reasonCode?: Array<CodeableConcept>;
  /** 
   * Condition or observation that supports why the prescription is being written
   */
  reasonReference?: Array<Reference>;
  /** 
   * Instantiates FHIR protocol or definition
   */
  instantiatesCanonical?: Array<canonical>;
  /** 
   * Instantiates FHIR protocol or definition
   */
  _instantiatesCanonical?: Array<Element>
  /** 
   * Instantiates external protocol or definition
   */
  instantiatesUri?: Array<uri>;
  /** 
   * Instantiates external protocol or definition
   */
  _instantiatesUri?: Array<Element>
  /** 
   * What request fulfills
   */
  basedOn?: Array<Reference>;
  /** 
   * Composite request this is part of
   */
  groupIdentifier?: Identifier;
  /** 
   * Overall pattern of medication administration
   */
  courseOfTherapyType?: CodeableConcept;
  /** 
   * Associated insurance coverage
   */
  insurance?: Array<Reference>;
  /** 
   * Information about the prescription
   */
  note?: Array<Annotation>;
  /** 
   * How the medication should be taken
   */
  dosageInstruction?: Array<Dosage>;
  /** 
   * Medication supply authorization
   */
  dispenseRequest?: MedicationRequestDispenseRequest;
  /** 
   * Any restrictions on medication substitution
   */
  substitution?: MedicationRequestSubstitution;
  /** 
   * An order/prescription that is being replaced
   */
  priorPrescription?: Reference;
  /** 
   * Clinical Issue with action
   */
  detectedIssue?: Array<Reference>;
  /** 
   * A list of events of interest in the lifecycle
   */
  eventHistory?: Array<Reference>;
}

export interface MedicationStatement {
resourceType: "MedicationStatement"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * External identifier
   */
  identifier?: Array<Identifier>;
  /** 
   * Fulfils plan, proposal or order
   */
  basedOn?: Array<Reference>;
  /** 
   * Part of referenced event
   */
  partOf?: Array<Reference>;
  /** 
   * active | completed | entered-in-error | intended | stopped | on-hold | unknown | not-taken
   */
  status: code;
  /** 
   * active | completed | entered-in-error | intended | stopped | on-hold | unknown | not-taken
   */
  _status?: Element
  /** 
   * Reason for current status
   */
  statusReason?: Array<CodeableConcept>;
  /** 
   * Type of medication usage
   */
  category?: CodeableConcept;
  /** 
   * What medication was taken
   */
  medicationCodeableConcept?: CodeableConcept;
  /** 
   * What medication was taken
   */
  medicationReference?: Reference;
  /** 
   * Who is/was taking  the medication
   */
  subject: Reference;
  /** 
   * Encounter / Episode associated with MedicationStatement
   */
  context?: Reference;
  /** 
   * The date/time or interval when the medication is/was/will be taken
   */
  effectiveDateTime?: dateTime;
  /** 
   * The date/time or interval when the medication is/was/will be taken
   */
  _effectiveDateTime?: Element
  /** 
   * The date/time or interval when the medication is/was/will be taken
   */
  effectivePeriod?: Period;
  /** 
   * When the statement was asserted?
   */
  dateAsserted?: dateTime;
  /** 
   * When the statement was asserted?
   */
  _dateAsserted?: Element
  /** 
   * Person or organization that provided the information about the taking of this medication
   */
  informationSource?: Reference;
  /** 
   * Additional supporting information
   */
  derivedFrom?: Array<Reference>;
  /** 
   * Reason for why the medication is being/was taken
   */
  reasonCode?: Array<CodeableConcept>;
  /** 
   * Condition or observation that supports why the medication is being/was taken
   */
  reasonReference?: Array<Reference>;
  /** 
   * Further information about the statement
   */
  note?: Array<Annotation>;
  /** 
   * Details of how medication is/was taken or should be taken
   */
  dosage?: Array<Dosage>;
}

export interface MedicinalProductDefinitionContact {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Allows the contact to be classified, for example QPPV, Pharmacovigilance Enquiry Information
   */
  type?: CodeableConcept;
  /** 
   * A product specific contact, person (in a role), or an organization
   */
  contact: Reference;
}
export interface MedicinalProductDefinitionNameNamePart {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * A fragment of a product name
   */
  part: string;
  /** 
   * A fragment of a product name
   */
  _part?: Element
  /** 
   * Identifying type for this part of the name (e.g. strength part)
   */
  type: CodeableConcept;
}
export interface MedicinalProductDefinitionNameCountryLanguage {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Country code for where this name applies
   */
  country: CodeableConcept;
  /** 
   * Jurisdiction code for where this name applies
   */
  jurisdiction?: CodeableConcept;
  /** 
   * Language code for this name
   */
  language: CodeableConcept;
}
export interface MedicinalProductDefinitionName {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The full product name
   */
  productName: string;
  /** 
   * The full product name
   */
  _productName?: Element
  /** 
   * Type of product name, such as rINN, BAN, Proprietary, Non-Proprietary
   */
  type?: CodeableConcept;
  /** 
   * Coding words or phrases of the name
   */
  namePart?: Array<MedicinalProductDefinitionNameNamePart>;
  /** 
   * Country and jurisdiction where the name applies
   */
  countryLanguage?: Array<MedicinalProductDefinitionNameCountryLanguage>;
}
export interface MedicinalProductDefinitionCrossReference {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Reference to another product, e.g. for linking authorised to investigational product
   */
  product: CodeableReference;
  /** 
   * The type of relationship, for instance branded to generic or virtual to actual product
   */
  type?: CodeableConcept;
}
export interface MedicinalProductDefinitionOperation {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The type of manufacturing operation e.g. manufacturing itself, re-packaging
   */
  type?: CodeableReference;
  /** 
   * Date range of applicability
   */
  effectiveDate?: Period;
  /** 
   * The organization responsible for the particular process, e.g. the manufacturer or importer
   */
  organization?: Array<Reference>;
  /** 
   * Specifies whether this process is considered proprietary or confidential
   */
  confidentialityIndicator?: CodeableConcept;
}
export interface MedicinalProductDefinitionCharacteristic {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * A code expressing the type of characteristic
   */
  type: CodeableConcept;
  /** 
   * A value for the characteristic
   */
  valueCodeableConcept?: CodeableConcept;
  /** 
   * A value for the characteristic
   */
  valueQuantity?: Quantity;
  /** 
   * A value for the characteristic
   */
  valueDate?: date;
  /** 
   * A value for the characteristic
   */
  _valueDate?: Element
  /** 
   * A value for the characteristic
   */
  valueBoolean?: boolean;
  /** 
   * A value for the characteristic
   */
  _valueBoolean?: Element
  /** 
   * A value for the characteristic
   */
  valueAttachment?: Attachment;
}
export interface MedicinalProductDefinition {
resourceType: "MedicinalProductDefinition"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business identifier for this product. Could be an MPID
   */
  identifier?: Array<Identifier>;
  /** 
   * Regulatory type, e.g. Investigational or Authorized
   */
  type?: CodeableConcept;
  /** 
   * If this medicine applies to human or veterinary uses
   */
  domain?: CodeableConcept;
  /** 
   * A business identifier relating to a specific version of the product
   */
  version?: string;
  /** 
   * A business identifier relating to a specific version of the product
   */
  _version?: Element
  /** 
   * The status within the lifecycle of this product record
   */
  status?: CodeableConcept;
  /** 
   * The date at which the given status became applicable
   */
  statusDate?: dateTime;
  /** 
   * The date at which the given status became applicable
   */
  _statusDate?: Element
  /** 
   * General description of this product
   */
  description?: markdown;
  /** 
   * General description of this product
   */
  _description?: Element
  /** 
   * The dose form for a single part product, or combined form of a multiple part product
   */
  combinedPharmaceuticalDoseForm?: CodeableConcept;
  /** 
   * The path by which the product is taken into or makes contact with the body
   */
  route?: Array<CodeableConcept>;
  /** 
   * Description of indication(s) for this product, used when structured indications are not required
   */
  indication?: markdown;
  /** 
   * Description of indication(s) for this product, used when structured indications are not required
   */
  _indication?: Element
  /** 
   * The legal status of supply of the medicinal product as classified by the regulator
   */
  legalStatusOfSupply?: CodeableConcept;
  /** 
   * Whether the Medicinal Product is subject to additional monitoring for regulatory reasons
   */
  additionalMonitoringIndicator?: CodeableConcept;
  /** 
   * Whether the Medicinal Product is subject to special measures for regulatory reasons
   */
  specialMeasures?: Array<CodeableConcept>;
  /** 
   * If authorised for use in children
   */
  pediatricUseIndicator?: CodeableConcept;
  /** 
   * Allows the product to be classified by various systems
   */
  classification?: Array<CodeableConcept>;
  /** 
   * Marketing status of the medicinal product, in contrast to marketing authorization
   */
  marketingStatus?: Array<MarketingStatus>;
  /** 
   * Package type for the product
   */
  packagedMedicinalProduct?: Array<CodeableConcept>;
  /** 
   * The ingredients of this medicinal product - when not detailed in other resources
   */
  ingredient?: Array<CodeableConcept>;
  /** 
   * Any component of the drug product which is not the chemical entity defined as the drug substance, or an excipient in the drug product
   */
  impurity?: Array<CodeableReference>;
  /** 
   * Additional documentation about the medicinal product
   */
  attachedDocument?: Array<Reference>;
  /** 
   * A master file for the medicinal product (e.g. Pharmacovigilance System Master File)
   */
  masterFile?: Array<Reference>;
  /** 
   * A product specific contact, person (in a role), or an organization
   */
  contact?: Array<MedicinalProductDefinitionContact>;
  /** 
   * Clinical trials or studies that this product is involved in
   */
  clinicalTrial?: Array<Reference>;
  /** 
   * A code that this product is known by, within some formal terminology
   */
  code?: Array<Coding>;
  /** 
   * The product's name, including full name and possibly coded parts
   */
  name: Array<MedicinalProductDefinitionName>;
  /** 
   * Reference to another product, e.g. for linking authorised to investigational product
   */
  crossReference?: Array<MedicinalProductDefinitionCrossReference>;
  /** 
   * A manufacturing or administrative process for the medicinal product
   */
  operation?: Array<MedicinalProductDefinitionOperation>;
  /** 
   * Key product features such as "sugar free", "modified release"
   */
  characteristic?: Array<MedicinalProductDefinitionCharacteristic>;
}

export interface MessageDefinitionFocus {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Type of resource
   */
  code: code;
  /** 
   * Type of resource
   */
  _code?: Element
  /** 
   * Profile that must be adhered to by focus
   */
  profile?: canonical;
  /** 
   * Profile that must be adhered to by focus
   */
  _profile?: Element
  /** 
   * Minimum number of focuses of this type
   */
  min: unsignedInt;
  /** 
   * Minimum number of focuses of this type
   */
  _min?: Element
  /** 
   * Maximum number of focuses of this type
   */
  max?: string;
  /** 
   * Maximum number of focuses of this type
   */
  _max?: Element
}
export interface MessageDefinitionAllowedResponse {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Reference to allowed message definition response
   */
  message: canonical;
  /** 
   * Reference to allowed message definition response
   */
  _message?: Element
  /** 
   * When should this response be used
   */
  situation?: markdown;
  /** 
   * When should this response be used
   */
  _situation?: Element
}
export interface MessageDefinition {
resourceType: "MessageDefinition"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business Identifier for a given MessageDefinition
   */
  url?: uri;
  /** 
   * Business Identifier for a given MessageDefinition
   */
  _url?: Element
  /** 
   * Primary key for the message definition on a given server
   */
  identifier?: Array<Identifier>;
  /** 
   * Business version of the message definition
   */
  version?: string;
  /** 
   * Business version of the message definition
   */
  _version?: Element
  /** 
   * Name for this message definition (computer friendly)
   */
  name?: string;
  /** 
   * Name for this message definition (computer friendly)
   */
  _name?: Element
  /** 
   * Name for this message definition (human friendly)
   */
  title?: string;
  /** 
   * Name for this message definition (human friendly)
   */
  _title?: Element
  /** 
   * Takes the place of
   */
  replaces?: Array<canonical>;
  /** 
   * Takes the place of
   */
  _replaces?: Array<Element>
  /** 
   * draft | active | retired | unknown
   */
  status: code;
  /** 
   * draft | active | retired | unknown
   */
  _status?: Element
  /** 
   * For testing purposes, not real usage
   */
  experimental?: boolean;
  /** 
   * For testing purposes, not real usage
   */
  _experimental?: Element
  /** 
   * Date last changed
   */
  date: dateTime;
  /** 
   * Date last changed
   */
  _date?: Element
  /** 
   * Name of the publisher (organization or individual)
   */
  publisher?: string;
  /** 
   * Name of the publisher (organization or individual)
   */
  _publisher?: Element
  /** 
   * Contact details for the publisher
   */
  contact?: Array<ContactDetail>;
  /** 
   * Natural language description of the message definition
   */
  description?: markdown;
  /** 
   * Natural language description of the message definition
   */
  _description?: Element
  /** 
   * The context that the content is intended to support
   */
  useContext?: Array<UsageContext>;
  /** 
   * Intended jurisdiction for message definition (if applicable)
   */
  jurisdiction?: Array<CodeableConcept>;
  /** 
   * Why this message definition is defined
   */
  purpose?: markdown;
  /** 
   * Why this message definition is defined
   */
  _purpose?: Element
  /** 
   * Use and/or publishing restrictions
   */
  copyright?: markdown;
  /** 
   * Use and/or publishing restrictions
   */
  _copyright?: Element
  /** 
   * Definition this one is based on
   */
  base?: canonical;
  /** 
   * Definition this one is based on
   */
  _base?: Element
  /** 
   * Protocol/workflow this is part of
   */
  parent?: Array<canonical>;
  /** 
   * Protocol/workflow this is part of
   */
  _parent?: Array<Element>
  /** 
   * Event code  or link to the EventDefinition
   */
  eventCoding?: Coding;
  /** 
   * Event code  or link to the EventDefinition
   */
  eventUri?: uri;
  /** 
   * Event code  or link to the EventDefinition
   */
  _eventUri?: Element
  /** 
   * consequence | currency | notification
   */
  category?: code;
  /** 
   * consequence | currency | notification
   */
  _category?: Element
  /** 
   * Resource(s) that are the subject of the event
   */
  focus?: Array<MessageDefinitionFocus>;
  /** 
   * always | on-error | never | on-success
   */
  responseRequired?: code;
  /** 
   * always | on-error | never | on-success
   */
  _responseRequired?: Element
  /** 
   * Responses to this message
   */
  allowedResponse?: Array<MessageDefinitionAllowedResponse>;
  /** 
   * Canonical reference to a GraphDefinition
   */
  graph?: Array<canonical>;
  /** 
   * Canonical reference to a GraphDefinition
   */
  _graph?: Array<Element>
}

export interface MessageHeaderDestination {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Name of system
   */
  name?: string;
  /** 
   * Name of system
   */
  _name?: Element
  /** 
   * Particular delivery destination within the destination
   */
  target?: Reference;
  /** 
   * Actual destination address or id
   */
  endpoint: url;
  /** 
   * Actual destination address or id
   */
  _endpoint?: Element
  /** 
   * Intended "real-world" recipient for the data
   */
  receiver?: Reference;
}
export interface MessageHeaderSource {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Name of system
   */
  name?: string;
  /** 
   * Name of system
   */
  _name?: Element
  /** 
   * Name of software running the system
   */
  software?: string;
  /** 
   * Name of software running the system
   */
  _software?: Element
  /** 
   * Version of software running
   */
  version?: string;
  /** 
   * Version of software running
   */
  _version?: Element
  /** 
   * Human contact for problems
   */
  contact?: ContactPoint;
  /** 
   * Actual message source address or id
   */
  endpoint: url;
  /** 
   * Actual message source address or id
   */
  _endpoint?: Element
}
export interface MessageHeaderResponse {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Id of original message
   */
  identifier: id;
  /** 
   * Id of original message
   */
  _identifier?: Element
  /** 
   * ok | transient-error | fatal-error
   */
  code: code;
  /** 
   * ok | transient-error | fatal-error
   */
  _code?: Element
  /** 
   * Specific list of hints/warnings/errors
   */
  details?: Reference;
}
export interface MessageHeader {
resourceType: "MessageHeader"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Code for the event this message represents or link to event definition
   */
  eventCoding?: Coding;
  /** 
   * Code for the event this message represents or link to event definition
   */
  eventUri?: uri;
  /** 
   * Code for the event this message represents or link to event definition
   */
  _eventUri?: Element
  /** 
   * Message destination application(s)
   */
  destination?: Array<MessageHeaderDestination>;
  /** 
   * Real world sender of the message
   */
  sender?: Reference;
  /** 
   * The source of the data entry
   */
  enterer?: Reference;
  /** 
   * The source of the decision
   */
  author?: Reference;
  /** 
   * Message source application
   */
  source: MessageHeaderSource;
  /** 
   * Final responsibility for event
   */
  responsible?: Reference;
  /** 
   * Cause of event
   */
  reason?: CodeableConcept;
  /** 
   * If this is a reply to prior message
   */
  response?: MessageHeaderResponse;
  /** 
   * The actual content of the message
   */
  focus?: Array<Reference>;
  /** 
   * Link to the definition for this message
   */
  definition?: canonical;
  /** 
   * Link to the definition for this message
   */
  _definition?: Element
}

export interface MolecularSequenceReferenceSeq {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Chromosome containing genetic finding
   */
  chromosome?: CodeableConcept;
  /** 
   * The Genome Build used for reference, following GRCh build versions e.g. 'GRCh 37'
   */
  genomeBuild?: string;
  /** 
   * The Genome Build used for reference, following GRCh build versions e.g. 'GRCh 37'
   */
  _genomeBuild?: Element
  /** 
   * sense | antisense
   */
  orientation?: code;
  /** 
   * sense | antisense
   */
  _orientation?: Element
  /** 
   * Reference identifier
   */
  referenceSeqId?: CodeableConcept;
  /** 
   * A pointer to another MolecularSequence entity as reference sequence
   */
  referenceSeqPointer?: Reference;
  /** 
   * A string to represent reference sequence
   */
  referenceSeqString?: string;
  /** 
   * A string to represent reference sequence
   */
  _referenceSeqString?: Element
  /** 
   * watson | crick
   */
  strand?: code;
  /** 
   * watson | crick
   */
  _strand?: Element
  /** 
   * Start position of the window on the  reference sequence
   */
  windowStart?: integer;
  /** 
   * Start position of the window on the  reference sequence
   */
  _windowStart?: Element
  /** 
   * End position of the window on the reference sequence
   */
  windowEnd?: integer;
  /** 
   * End position of the window on the reference sequence
   */
  _windowEnd?: Element
}
export interface MolecularSequenceVariant {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Start position of the variant on the  reference sequence
   */
  start?: integer;
  /** 
   * Start position of the variant on the  reference sequence
   */
  _start?: Element
  /** 
   * End position of the variant on the reference sequence
   */
  end?: integer;
  /** 
   * End position of the variant on the reference sequence
   */
  _end?: Element
  /** 
   * Allele that was observed
   */
  observedAllele?: string;
  /** 
   * Allele that was observed
   */
  _observedAllele?: Element
  /** 
   * Allele in the reference sequence
   */
  referenceAllele?: string;
  /** 
   * Allele in the reference sequence
   */
  _referenceAllele?: Element
  /** 
   * Extended CIGAR string for aligning the sequence with reference bases
   */
  cigar?: string;
  /** 
   * Extended CIGAR string for aligning the sequence with reference bases
   */
  _cigar?: Element
  /** 
   * Pointer to observed variant information
   */
  variantPointer?: Reference;
}
export interface MolecularSequenceQualityRoc {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Genotype quality score
   */
  score?: Array<integer>;
  /** 
   * Genotype quality score
   */
  _score?: Array<Element>
  /** 
   * Roc score true positive numbers
   */
  numTP?: Array<integer>;
  /** 
   * Roc score true positive numbers
   */
  _numTP?: Array<Element>
  /** 
   * Roc score false positive numbers
   */
  numFP?: Array<integer>;
  /** 
   * Roc score false positive numbers
   */
  _numFP?: Array<Element>
  /** 
   * Roc score false negative numbers
   */
  numFN?: Array<integer>;
  /** 
   * Roc score false negative numbers
   */
  _numFN?: Array<Element>
  /** 
   * Precision of the GQ score
   */
  precision?: Array<decimal>;
  /** 
   * Precision of the GQ score
   */
  _precision?: Array<Element>
  /** 
   * Sensitivity of the GQ score
   */
  sensitivity?: Array<decimal>;
  /** 
   * Sensitivity of the GQ score
   */
  _sensitivity?: Array<Element>
  /** 
   * FScore of the GQ score
   */
  fMeasure?: Array<decimal>;
  /** 
   * FScore of the GQ score
   */
  _fMeasure?: Array<Element>
}
export interface MolecularSequenceQuality {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * indel | snp | unknown
   */
  type: code;
  /** 
   * indel | snp | unknown
   */
  _type?: Element
  /** 
   * Standard sequence for comparison
   */
  standardSequence?: CodeableConcept;
  /** 
   * Start position of the sequence
   */
  start?: integer;
  /** 
   * Start position of the sequence
   */
  _start?: Element
  /** 
   * End position of the sequence
   */
  end?: integer;
  /** 
   * End position of the sequence
   */
  _end?: Element
  /** 
   * Quality score for the comparison
   */
  score?: Quantity;
  /** 
   * Method to get quality
   */
  method?: CodeableConcept;
  /** 
   * True positives from the perspective of the truth data
   */
  truthTP?: decimal;
  /** 
   * True positives from the perspective of the truth data
   */
  _truthTP?: Element
  /** 
   * True positives from the perspective of the query data
   */
  queryTP?: decimal;
  /** 
   * True positives from the perspective of the query data
   */
  _queryTP?: Element
  /** 
   * False negatives
   */
  truthFN?: decimal;
  /** 
   * False negatives
   */
  _truthFN?: Element
  /** 
   * False positives
   */
  queryFP?: decimal;
  /** 
   * False positives
   */
  _queryFP?: Element
  /** 
   * False positives where the non-REF alleles in the Truth and Query Call Sets match
   */
  gtFP?: decimal;
  /** 
   * False positives where the non-REF alleles in the Truth and Query Call Sets match
   */
  _gtFP?: Element
  /** 
   * Precision of comparison
   */
  precision?: decimal;
  /** 
   * Precision of comparison
   */
  _precision?: Element
  /** 
   * Recall of comparison
   */
  recall?: decimal;
  /** 
   * Recall of comparison
   */
  _recall?: Element
  /** 
   * F-score
   */
  fScore?: decimal;
  /** 
   * F-score
   */
  _fScore?: Element
  /** 
   * Receiver Operator Characteristic (ROC) Curve
   */
  roc?: MolecularSequenceQualityRoc;
}
export interface MolecularSequenceRepository {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * directlink | openapi | login | oauth | other
   */
  type: code;
  /** 
   * directlink | openapi | login | oauth | other
   */
  _type?: Element
  /** 
   * URI of the repository
   */
  url?: uri;
  /** 
   * URI of the repository
   */
  _url?: Element
  /** 
   * Repository's name
   */
  name?: string;
  /** 
   * Repository's name
   */
  _name?: Element
  /** 
   * Id of the dataset that used to call for dataset in repository
   */
  datasetId?: string;
  /** 
   * Id of the dataset that used to call for dataset in repository
   */
  _datasetId?: Element
  /** 
   * Id of the variantset that used to call for variantset in repository
   */
  variantsetId?: string;
  /** 
   * Id of the variantset that used to call for variantset in repository
   */
  _variantsetId?: Element
  /** 
   * Id of the read
   */
  readsetId?: string;
  /** 
   * Id of the read
   */
  _readsetId?: Element
}
export interface MolecularSequenceStructureVariantOuter {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Structural variant outer start
   */
  start?: integer;
  /** 
   * Structural variant outer start
   */
  _start?: Element
  /** 
   * Structural variant outer end
   */
  end?: integer;
  /** 
   * Structural variant outer end
   */
  _end?: Element
}
export interface MolecularSequenceStructureVariantInner {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Structural variant inner start
   */
  start?: integer;
  /** 
   * Structural variant inner start
   */
  _start?: Element
  /** 
   * Structural variant inner end
   */
  end?: integer;
  /** 
   * Structural variant inner end
   */
  _end?: Element
}
export interface MolecularSequenceStructureVariant {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Structural variant change type
   */
  variantType?: CodeableConcept;
  /** 
   * Does the structural variant have base pair resolution breakpoints?
   */
  exact?: boolean;
  /** 
   * Does the structural variant have base pair resolution breakpoints?
   */
  _exact?: Element
  /** 
   * Structural variant length
   */
  length?: integer;
  /** 
   * Structural variant length
   */
  _length?: Element
  /** 
   * Structural variant outer
   */
  outer?: MolecularSequenceStructureVariantOuter;
  /** 
   * Structural variant inner
   */
  inner?: MolecularSequenceStructureVariantInner;
}
export interface MolecularSequence {
resourceType: "MolecularSequence"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Unique ID for this particular sequence. This is a FHIR-defined id
   */
  identifier?: Array<Identifier>;
  /** 
   * aa | dna | rna
   */
  type?: code;
  /** 
   * aa | dna | rna
   */
  _type?: Element
  /** 
   * Base number of coordinate system (0 for 0-based numbering or coordinates, inclusive start, exclusive end, 1 for 1-based numbering, inclusive start, inclusive end)
   */
  coordinateSystem: integer;
  /** 
   * Base number of coordinate system (0 for 0-based numbering or coordinates, inclusive start, exclusive end, 1 for 1-based numbering, inclusive start, inclusive end)
   */
  _coordinateSystem?: Element
  /** 
   * Who and/or what this is about
   */
  patient?: Reference;
  /** 
   * Specimen used for sequencing
   */
  specimen?: Reference;
  /** 
   * The method for sequencing
   */
  device?: Reference;
  /** 
   * Who should be responsible for test result
   */
  performer?: Reference;
  /** 
   * The number of copies of the sequence of interest.  (RNASeq)
   */
  quantity?: Quantity;
  /** 
   * A sequence used as reference
   */
  referenceSeq?: MolecularSequenceReferenceSeq;
  /** 
   * Variant in sequence
   */
  variant?: Array<MolecularSequenceVariant>;
  /** 
   * Sequence that was observed
   */
  observedSeq?: string;
  /** 
   * Sequence that was observed
   */
  _observedSeq?: Element
  /** 
   * An set of value as quality of sequence
   */
  quality?: Array<MolecularSequenceQuality>;
  /** 
   * Average number of reads representing a given nucleotide in the reconstructed sequence
   */
  readCoverage?: integer;
  /** 
   * Average number of reads representing a given nucleotide in the reconstructed sequence
   */
  _readCoverage?: Element
  /** 
   * External repository which contains detailed report related with observedSeq in this resource
   */
  repository?: Array<MolecularSequenceRepository>;
  /** 
   * Pointer to next atomic sequence
   */
  pointer?: Array<Reference>;
  /** 
   * Structural variant
   */
  structureVariant?: Array<MolecularSequenceStructureVariant>;
}

export interface NamingSystemUniqueId {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * oid | uuid | uri | other
   */
  type: code;
  /** 
   * oid | uuid | uri | other
   */
  _type?: Element
  /** 
   * The unique identifier
   */
  value: string;
  /** 
   * The unique identifier
   */
  _value?: Element
  /** 
   * Is this the id that should be used for this type
   */
  preferred?: boolean;
  /** 
   * Is this the id that should be used for this type
   */
  _preferred?: Element
  /** 
   * Notes about identifier usage
   */
  comment?: string;
  /** 
   * Notes about identifier usage
   */
  _comment?: Element
  /** 
   * When is identifier valid?
   */
  period?: Period;
}
export interface NamingSystem {
resourceType: "NamingSystem"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Name for this naming system (computer friendly)
   */
  name: string;
  /** 
   * Name for this naming system (computer friendly)
   */
  _name?: Element
  /** 
   * draft | active | retired | unknown
   */
  status: code;
  /** 
   * draft | active | retired | unknown
   */
  _status?: Element
  /** 
   * codesystem | identifier | root
   */
  kind: code;
  /** 
   * codesystem | identifier | root
   */
  _kind?: Element
  /** 
   * Date last changed
   */
  date: dateTime;
  /** 
   * Date last changed
   */
  _date?: Element
  /** 
   * Name of the publisher (organization or individual)
   */
  publisher?: string;
  /** 
   * Name of the publisher (organization or individual)
   */
  _publisher?: Element
  /** 
   * Contact details for the publisher
   */
  contact?: Array<ContactDetail>;
  /** 
   * Who maintains system namespace?
   */
  responsible?: string;
  /** 
   * Who maintains system namespace?
   */
  _responsible?: Element
  /** 
   * e.g. driver,  provider,  patient, bank etc.
   */
  type?: CodeableConcept;
  /** 
   * Natural language description of the naming system
   */
  description?: markdown;
  /** 
   * Natural language description of the naming system
   */
  _description?: Element
  /** 
   * The context that the content is intended to support
   */
  useContext?: Array<UsageContext>;
  /** 
   * Intended jurisdiction for naming system (if applicable)
   */
  jurisdiction?: Array<CodeableConcept>;
  /** 
   * How/where is it used
   */
  usage?: string;
  /** 
   * How/where is it used
   */
  _usage?: Element
  /** 
   * Unique identifiers used for system
   */
  uniqueId: Array<NamingSystemUniqueId>;
}

export interface NutritionOrderOralDietNutrient {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Type of nutrient that is being modified
   */
  modifier?: CodeableConcept;
  /** 
   * Quantity of the specified nutrient
   */
  amount?: Quantity;
}
export interface NutritionOrderOralDietTexture {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Code to indicate how to alter the texture of the foods, e.g. pureed
   */
  modifier?: CodeableConcept;
  /** 
   * Concepts that are used to identify an entity that is ingested for nutritional purposes
   */
  foodType?: CodeableConcept;
}
export interface NutritionOrderOralDiet {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Type of oral diet or diet restrictions that describe what can be consumed orally
   */
  type?: Array<CodeableConcept>;
  /** 
   * Scheduled frequency of diet
   */
  schedule?: Array<Timing>;
  /** 
   * Required  nutrient modifications
   */
  nutrient?: Array<NutritionOrderOralDietNutrient>;
  /** 
   * Required  texture modifications
   */
  texture?: Array<NutritionOrderOralDietTexture>;
  /** 
   * The required consistency of fluids and liquids provided to the patient
   */
  fluidConsistencyType?: Array<CodeableConcept>;
  /** 
   * Instructions or additional information about the oral diet
   */
  instruction?: string;
  /** 
   * Instructions or additional information about the oral diet
   */
  _instruction?: Element
}
export interface NutritionOrderSupplement {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Type of supplement product requested
   */
  type?: CodeableConcept;
  /** 
   * Product or brand name of the nutritional supplement
   */
  productName?: string;
  /** 
   * Product or brand name of the nutritional supplement
   */
  _productName?: Element
  /** 
   * Scheduled frequency of supplement
   */
  schedule?: Array<Timing>;
  /** 
   * Amount of the nutritional supplement
   */
  quantity?: Quantity;
  /** 
   * Instructions or additional information about the oral supplement
   */
  instruction?: string;
  /** 
   * Instructions or additional information about the oral supplement
   */
  _instruction?: Element
}
export interface NutritionOrderEnteralFormulaAdministration {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Scheduled frequency of enteral feeding
   */
  schedule?: Timing;
  /** 
   * The volume of formula to provide
   */
  quantity?: Quantity;
  /** 
   * Speed with which the formula is provided per period of time
   */
  rateQuantity?: Quantity;
  /** 
   * Speed with which the formula is provided per period of time
   */
  rateRatio?: Ratio;
}
export interface NutritionOrderEnteralFormula {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Type of enteral or infant formula
   */
  baseFormulaType?: CodeableConcept;
  /** 
   * Product or brand name of the enteral or infant formula
   */
  baseFormulaProductName?: string;
  /** 
   * Product or brand name of the enteral or infant formula
   */
  _baseFormulaProductName?: Element
  /** 
   * Type of modular component to add to the feeding
   */
  additiveType?: CodeableConcept;
  /** 
   * Product or brand name of the modular additive
   */
  additiveProductName?: string;
  /** 
   * Product or brand name of the modular additive
   */
  _additiveProductName?: Element
  /** 
   * Amount of energy per specified volume that is required
   */
  caloricDensity?: Quantity;
  /** 
   * How the formula should enter the patient's gastrointestinal tract
   */
  routeofAdministration?: CodeableConcept;
  /** 
   * Formula feeding instruction as structured data
   */
  administration?: Array<NutritionOrderEnteralFormulaAdministration>;
  /** 
   * Upper limit on formula volume per unit of time
   */
  maxVolumeToDeliver?: Quantity;
  /** 
   * Formula feeding instructions expressed as text
   */
  administrationInstruction?: string;
  /** 
   * Formula feeding instructions expressed as text
   */
  _administrationInstruction?: Element
}
export interface NutritionOrder {
resourceType: "NutritionOrder"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Identifiers assigned to this order
   */
  identifier?: Array<Identifier>;
  /** 
   * Instantiates FHIR protocol or definition
   */
  instantiatesCanonical?: Array<canonical>;
  /** 
   * Instantiates FHIR protocol or definition
   */
  _instantiatesCanonical?: Array<Element>
  /** 
   * Instantiates external protocol or definition
   */
  instantiatesUri?: Array<uri>;
  /** 
   * Instantiates external protocol or definition
   */
  _instantiatesUri?: Array<Element>
  /** 
   * Instantiates protocol or definition
   */
  instantiates?: Array<uri>;
  /** 
   * Instantiates protocol or definition
   */
  _instantiates?: Array<Element>
  /** 
   * draft | active | on-hold | revoked | completed | entered-in-error | unknown
   */
  status: code;
  /** 
   * draft | active | on-hold | revoked | completed | entered-in-error | unknown
   */
  _status?: Element
  /** 
   * proposal | plan | directive | order | original-order | reflex-order | filler-order | instance-order | option
   */
  intent: code;
  /** 
   * proposal | plan | directive | order | original-order | reflex-order | filler-order | instance-order | option
   */
  _intent?: Element
  /** 
   * The person who requires the diet, formula or nutritional supplement
   */
  patient: Reference;
  /** 
   * The encounter associated with this nutrition order
   */
  encounter?: Reference;
  /** 
   * Date and time the nutrition order was requested
   */
  dateTime: dateTime;
  /** 
   * Date and time the nutrition order was requested
   */
  _dateTime?: Element
  /** 
   * Who ordered the diet, formula or nutritional supplement
   */
  orderer?: Reference;
  /** 
   * List of the patient's food and nutrition-related allergies and intolerances
   */
  allergyIntolerance?: Array<Reference>;
  /** 
   * Order-specific modifier about the type of food that should be given
   */
  foodPreferenceModifier?: Array<CodeableConcept>;
  /** 
   * Order-specific modifier about the type of food that should not be given
   */
  excludeFoodModifier?: Array<CodeableConcept>;
  /** 
   * Oral diet components
   */
  oralDiet?: NutritionOrderOralDiet;
  /** 
   * Supplement components
   */
  supplement?: Array<NutritionOrderSupplement>;
  /** 
   * Enteral formula components
   */
  enteralFormula?: NutritionOrderEnteralFormula;
  /** 
   * Comments
   */
  note?: Array<Annotation>;
}

export interface NutritionProductNutrient {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The (relevant) nutrients in the product
   */
  item?: CodeableReference;
  /** 
   * The amount of nutrient expressed in one or more units: X per pack / per serving / per dose
   */
  amount?: Array<Ratio>;
}
export interface NutritionProductIngredient {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The ingredient contained in the product
   */
  item: CodeableReference;
  /** 
   * The amount of ingredient that is in the product
   */
  amount?: Array<Ratio>;
}
export interface NutritionProductProductCharacteristic {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Code specifying the type of characteristic
   */
  type: CodeableConcept;
  /** 
   * The value of the characteristic
   */
  valueCodeableConcept?: CodeableConcept;
  /** 
   * The value of the characteristic
   */
  valueString?: string;
  /** 
   * The value of the characteristic
   */
  _valueString?: Element
  /** 
   * The value of the characteristic
   */
  valueQuantity?: Quantity;
  /** 
   * The value of the characteristic
   */
  valueBase64Binary?: base64Binary;
  /** 
   * The value of the characteristic
   */
  _valueBase64Binary?: Element
  /** 
   * The value of the characteristic
   */
  valueAttachment?: Attachment;
  /** 
   * The value of the characteristic
   */
  valueBoolean?: boolean;
  /** 
   * The value of the characteristic
   */
  _valueBoolean?: Element
}
export interface NutritionProductInstance {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The amount of items or instances
   */
  quantity?: Quantity;
  /** 
   * The identifier for the physical instance, typically a serial number
   */
  identifier?: Array<Identifier>;
  /** 
   * The identification of the batch or lot of the product
   */
  lotNumber?: string;
  /** 
   * The identification of the batch or lot of the product
   */
  _lotNumber?: Element
  /** 
   * The expiry date or date and time for the product
   */
  expiry?: dateTime;
  /** 
   * The expiry date or date and time for the product
   */
  _expiry?: Element
  /** 
   * The date until which the product is expected to be good for consumption
   */
  useBy?: dateTime;
  /** 
   * The date until which the product is expected to be good for consumption
   */
  _useBy?: Element
}
export interface NutritionProduct {
resourceType: "NutritionProduct"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * active | inactive | entered-in-error
   */
  status: code;
  /** 
   * active | inactive | entered-in-error
   */
  _status?: Element
  /** 
   * A category or class of the nutrition product (halal, kosher, gluten free, vegan, etc)
   */
  category?: Array<CodeableConcept>;
  /** 
   * A code designating a specific type of nutritional product
   */
  code?: CodeableConcept;
  /** 
   * Manufacturer, representative or officially responsible for the product
   */
  manufacturer?: Array<Reference>;
  /** 
   * The product's nutritional information expressed by the nutrients
   */
  nutrient?: Array<NutritionProductNutrient>;
  /** 
   * Ingredients contained in this product
   */
  ingredient?: Array<NutritionProductIngredient>;
  /** 
   * Known or suspected allergens that are a part of this product
   */
  knownAllergen?: Array<CodeableReference>;
  /** 
   * Specifies descriptive properties of the nutrition product
   */
  productCharacteristic?: Array<NutritionProductProductCharacteristic>;
  /** 
   * One or several physical instances or occurrences of the nutrition product
   */
  instance?: NutritionProductInstance;
  /** 
   * Comments made about the product
   */
  note?: Array<Annotation>;
}

export interface ObservationReferenceRange {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Low Range, if relevant
   */
  low?: Quantity;
  /** 
   * High Range, if relevant
   */
  high?: Quantity;
  /** 
   * Reference range qualifier
   */
  type?: CodeableConcept;
  /** 
   * Reference range population
   */
  appliesTo?: Array<CodeableConcept>;
  /** 
   * Applicable age range, if relevant
   */
  age?: Range;
  /** 
   * Text based reference range in an observation
   */
  text?: string;
  /** 
   * Text based reference range in an observation
   */
  _text?: Element
}
export interface ObservationComponent {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Type of component observation (code / type)
   */
  code: CodeableConcept;
  /** 
   * Actual component result
   */
  valueQuantity?: Quantity;
  /** 
   * Actual component result
   */
  valueCodeableConcept?: CodeableConcept;
  /** 
   * Actual component result
   */
  valueString?: string;
  /** 
   * Actual component result
   */
  _valueString?: Element
  /** 
   * Actual component result
   */
  valueBoolean?: boolean;
  /** 
   * Actual component result
   */
  _valueBoolean?: Element
  /** 
   * Actual component result
   */
  valueInteger?: integer;
  /** 
   * Actual component result
   */
  _valueInteger?: Element
  /** 
   * Actual component result
   */
  valueRange?: Range;
  /** 
   * Actual component result
   */
  valueRatio?: Ratio;
  /** 
   * Actual component result
   */
  valueSampledData?: SampledData;
  /** 
   * Actual component result
   */
  valueTime?: time;
  /** 
   * Actual component result
   */
  _valueTime?: Element
  /** 
   * Actual component result
   */
  valueDateTime?: dateTime;
  /** 
   * Actual component result
   */
  _valueDateTime?: Element
  /** 
   * Actual component result
   */
  valuePeriod?: Period;
  /** 
   * Why the component result is missing
   */
  dataAbsentReason?: CodeableConcept;
  /** 
   * High, low, normal, etc.
   */
  interpretation?: Array<CodeableConcept>;
  /** 
   * Provides guide for interpretation of component result
   */
  referenceRange?: Array<ObservationReferenceRange>;
}
export interface Observation {
resourceType: "Observation"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business Identifier for observation
   */
  identifier?: Array<Identifier>;
  /** 
   * Fulfills plan, proposal or order
   */
  basedOn?: Array<Reference>;
  /** 
   * Part of referenced event
   */
  partOf?: Array<Reference>;
  /** 
   * registered | preliminary | final | amended +
   */
  status: code;
  /** 
   * registered | preliminary | final | amended +
   */
  _status?: Element
  /** 
   * Classification of  type of observation
   */
  category?: Array<CodeableConcept>;
  /** 
   * Type of observation (code / type)
   */
  code: CodeableConcept;
  /** 
   * Who and/or what the observation is about
   */
  subject?: Reference;
  /** 
   * What the observation is about, when it is not about the subject of record
   */
  focus?: Array<Reference>;
  /** 
   * Healthcare event during which this observation is made
   */
  encounter?: Reference;
  /** 
   * Clinically relevant time/time-period for observation
   */
  effectiveDateTime?: dateTime;
  /** 
   * Clinically relevant time/time-period for observation
   */
  _effectiveDateTime?: Element
  /** 
   * Clinically relevant time/time-period for observation
   */
  effectivePeriod?: Period;
  /** 
   * Clinically relevant time/time-period for observation
   */
  effectiveTiming?: Timing;
  /** 
   * Clinically relevant time/time-period for observation
   */
  effectiveInstant?: instant;
  /** 
   * Clinically relevant time/time-period for observation
   */
  _effectiveInstant?: Element
  /** 
   * Date/Time this version was made available
   */
  issued?: instant;
  /** 
   * Date/Time this version was made available
   */
  _issued?: Element
  /** 
   * Who is responsible for the observation
   */
  performer?: Array<Reference>;
  /** 
   * Actual result
   */
  valueQuantity?: Quantity;
  /** 
   * Actual result
   */
  valueCodeableConcept?: CodeableConcept;
  /** 
   * Actual result
   */
  valueString?: string;
  /** 
   * Actual result
   */
  _valueString?: Element
  /** 
   * Actual result
   */
  valueBoolean?: boolean;
  /** 
   * Actual result
   */
  _valueBoolean?: Element
  /** 
   * Actual result
   */
  valueInteger?: integer;
  /** 
   * Actual result
   */
  _valueInteger?: Element
  /** 
   * Actual result
   */
  valueRange?: Range;
  /** 
   * Actual result
   */
  valueRatio?: Ratio;
  /** 
   * Actual result
   */
  valueSampledData?: SampledData;
  /** 
   * Actual result
   */
  valueTime?: time;
  /** 
   * Actual result
   */
  _valueTime?: Element
  /** 
   * Actual result
   */
  valueDateTime?: dateTime;
  /** 
   * Actual result
   */
  _valueDateTime?: Element
  /** 
   * Actual result
   */
  valuePeriod?: Period;
  /** 
   * Why the result is missing
   */
  dataAbsentReason?: CodeableConcept;
  /** 
   * High, low, normal, etc.
   */
  interpretation?: Array<CodeableConcept>;
  /** 
   * Comments about the observation
   */
  note?: Array<Annotation>;
  /** 
   * Observed body part
   */
  bodySite?: CodeableConcept;
  /** 
   * How it was done
   */
  method?: CodeableConcept;
  /** 
   * Specimen used for this observation
   */
  specimen?: Reference;
  /** 
   * (Measurement) Device
   */
  device?: Reference;
  /** 
   * Provides guide for interpretation
   */
  referenceRange?: Array<ObservationReferenceRange>;
  /** 
   * Related resource that belongs to the Observation group
   */
  hasMember?: Array<Reference>;
  /** 
   * Related measurements the observation is made from
   */
  derivedFrom?: Array<Reference>;
  /** 
   * Component results
   */
  component?: Array<ObservationComponent>;
}

export interface ObservationDefinitionQuantitativeDetails {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Customary unit for quantitative results
   */
  customaryUnit?: CodeableConcept;
  /** 
   * SI unit for quantitative results
   */
  unit?: CodeableConcept;
  /** 
   * SI to Customary unit conversion factor
   */
  conversionFactor?: decimal;
  /** 
   * SI to Customary unit conversion factor
   */
  _conversionFactor?: Element
  /** 
   * Decimal precision of observation quantitative results
   */
  decimalPrecision?: integer;
  /** 
   * Decimal precision of observation quantitative results
   */
  _decimalPrecision?: Element
}
export interface ObservationDefinitionQualifiedInterval {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * reference | critical | absolute
   */
  category?: code;
  /** 
   * reference | critical | absolute
   */
  _category?: Element
  /** 
   * The interval itself, for continuous or ordinal observations
   */
  range?: Range;
  /** 
   * Range context qualifier
   */
  context?: CodeableConcept;
  /** 
   * Targetted population of the range
   */
  appliesTo?: Array<CodeableConcept>;
  /** 
   * male | female | other | unknown
   */
  gender?: code;
  /** 
   * male | female | other | unknown
   */
  _gender?: Element
  /** 
   * Applicable age range, if relevant
   */
  age?: Range;
  /** 
   * Applicable gestational age range, if relevant
   */
  gestationalAge?: Range;
  /** 
   * Condition associated with the reference range
   */
  condition?: string;
  /** 
   * Condition associated with the reference range
   */
  _condition?: Element
}
export interface ObservationDefinition {
resourceType: "ObservationDefinition"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Category of observation
   */
  category?: Array<CodeableConcept>;
  /** 
   * Type of observation (code / type)
   */
  code: CodeableConcept;
  /** 
   * Business identifier for this ObservationDefinition instance
   */
  identifier?: Array<Identifier>;
  /** 
   * Quantity | CodeableConcept | string | boolean | integer | Range | Ratio | SampledData | time | dateTime | Period
   */
  permittedDataType?: Array<code>;
  /** 
   * Quantity | CodeableConcept | string | boolean | integer | Range | Ratio | SampledData | time | dateTime | Period
   */
  _permittedDataType?: Array<Element>
  /** 
   * Multiple results allowed
   */
  multipleResultsAllowed?: boolean;
  /** 
   * Multiple results allowed
   */
  _multipleResultsAllowed?: Element
  /** 
   * Method used to produce the observation
   */
  method?: CodeableConcept;
  /** 
   * Preferred report name
   */
  preferredReportName?: string;
  /** 
   * Preferred report name
   */
  _preferredReportName?: Element
  /** 
   * Characteristics of quantitative results
   */
  quantitativeDetails?: ObservationDefinitionQuantitativeDetails;
  /** 
   * Qualified range for continuous and ordinal observation results
   */
  qualifiedInterval?: Array<ObservationDefinitionQualifiedInterval>;
  /** 
   * Value set of valid coded values for the observations conforming to this ObservationDefinition
   */
  validCodedValueSet?: Reference;
  /** 
   * Value set of normal coded values for the observations conforming to this ObservationDefinition
   */
  normalCodedValueSet?: Reference;
  /** 
   * Value set of abnormal coded values for the observations conforming to this ObservationDefinition
   */
  abnormalCodedValueSet?: Reference;
  /** 
   * Value set of critical coded values for the observations conforming to this ObservationDefinition
   */
  criticalCodedValueSet?: Reference;
}

export interface OperationDefinitionParameterBinding {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * required | extensible | preferred | example
   */
  strength: code;
  /** 
   * required | extensible | preferred | example
   */
  _strength?: Element
  /** 
   * Source of value set
   */
  valueSet: canonical;
  /** 
   * Source of value set
   */
  _valueSet?: Element
}
export interface OperationDefinitionParameterReferencedFrom {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Referencing parameter
   */
  source: string;
  /** 
   * Referencing parameter
   */
  _source?: Element
  /** 
   * Element id of reference
   */
  sourceId?: string;
  /** 
   * Element id of reference
   */
  _sourceId?: Element
}
export interface OperationDefinitionParameter {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Name in Parameters.parameter.name or in URL
   */
  name: code;
  /** 
   * Name in Parameters.parameter.name or in URL
   */
  _name?: Element
  /** 
   * in | out
   */
  use: code;
  /** 
   * in | out
   */
  _use?: Element
  /** 
   * Minimum Cardinality
   */
  min: integer;
  /** 
   * Minimum Cardinality
   */
  _min?: Element
  /** 
   * Maximum Cardinality (a number or *)
   */
  max: string;
  /** 
   * Maximum Cardinality (a number or *)
   */
  _max?: Element
  /** 
   * Description of meaning/use
   */
  documentation?: string;
  /** 
   * Description of meaning/use
   */
  _documentation?: Element
  /** 
   * What type this parameter has
   */
  type?: code;
  /** 
   * What type this parameter has
   */
  _type?: Element
  /** 
   * If type is Reference | canonical, allowed targets
   */
  targetProfile?: Array<canonical>;
  /** 
   * If type is Reference | canonical, allowed targets
   */
  _targetProfile?: Array<Element>
  /** 
   * number | date | string | token | reference | composite | quantity | uri | special
   */
  searchType?: code;
  /** 
   * number | date | string | token | reference | composite | quantity | uri | special
   */
  _searchType?: Element
  /** 
   * ValueSet details if this is coded
   */
  binding?: OperationDefinitionParameterBinding;
  /** 
   * References to this parameter
   */
  referencedFrom?: Array<OperationDefinitionParameterReferencedFrom>;
  /** 
   * Parts of a nested Parameter
   */
  part?: Array<OperationDefinitionParameter>;
}
export interface OperationDefinitionOverload {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Name of parameter to include in overload
   */
  parameterName?: Array<string>;
  /** 
   * Name of parameter to include in overload
   */
  _parameterName?: Array<Element>
  /** 
   * Comments to go on overload
   */
  comment?: string;
  /** 
   * Comments to go on overload
   */
  _comment?: Element
}
export interface OperationDefinition {
resourceType: "OperationDefinition"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Canonical identifier for this operation definition, represented as a URI (globally unique)
   */
  url?: uri;
  /** 
   * Canonical identifier for this operation definition, represented as a URI (globally unique)
   */
  _url?: Element
  /** 
   * Business version of the operation definition
   */
  version?: string;
  /** 
   * Business version of the operation definition
   */
  _version?: Element
  /** 
   * Name for this operation definition (computer friendly)
   */
  name: string;
  /** 
   * Name for this operation definition (computer friendly)
   */
  _name?: Element
  /** 
   * Name for this operation definition (human friendly)
   */
  title?: string;
  /** 
   * Name for this operation definition (human friendly)
   */
  _title?: Element
  /** 
   * draft | active | retired | unknown
   */
  status: code;
  /** 
   * draft | active | retired | unknown
   */
  _status?: Element
  /** 
   * operation | query
   */
  kind: code;
  /** 
   * operation | query
   */
  _kind?: Element
  /** 
   * For testing purposes, not real usage
   */
  experimental?: boolean;
  /** 
   * For testing purposes, not real usage
   */
  _experimental?: Element
  /** 
   * Date last changed
   */
  date?: dateTime;
  /** 
   * Date last changed
   */
  _date?: Element
  /** 
   * Name of the publisher (organization or individual)
   */
  publisher?: string;
  /** 
   * Name of the publisher (organization or individual)
   */
  _publisher?: Element
  /** 
   * Contact details for the publisher
   */
  contact?: Array<ContactDetail>;
  /** 
   * Natural language description of the operation definition
   */
  description?: markdown;
  /** 
   * Natural language description of the operation definition
   */
  _description?: Element
  /** 
   * The context that the content is intended to support
   */
  useContext?: Array<UsageContext>;
  /** 
   * Intended jurisdiction for operation definition (if applicable)
   */
  jurisdiction?: Array<CodeableConcept>;
  /** 
   * Why this operation definition is defined
   */
  purpose?: markdown;
  /** 
   * Why this operation definition is defined
   */
  _purpose?: Element
  /** 
   * Whether content is changed by the operation
   */
  affectsState?: boolean;
  /** 
   * Whether content is changed by the operation
   */
  _affectsState?: Element
  /** 
   * Name used to invoke the operation
   */
  code: code;
  /** 
   * Name used to invoke the operation
   */
  _code?: Element
  /** 
   * Additional information about use
   */
  comment?: markdown;
  /** 
   * Additional information about use
   */
  _comment?: Element
  /** 
   * Marks this as a profile of the base
   */
  base?: canonical;
  /** 
   * Marks this as a profile of the base
   */
  _base?: Element
  /** 
   * Types this operation applies to
   */
  resource?: Array<code>;
  /** 
   * Types this operation applies to
   */
  _resource?: Array<Element>
  /** 
   * Invoke at the system level?
   */
  system: boolean;
  /** 
   * Invoke at the system level?
   */
  _system?: Element
  /** 
   * Invoke at the type level?
   */
  type: boolean;
  /** 
   * Invoke at the type level?
   */
  _type?: Element
  /** 
   * Invoke on an instance?
   */
  instance: boolean;
  /** 
   * Invoke on an instance?
   */
  _instance?: Element
  /** 
   * Validation information for in parameters
   */
  inputProfile?: canonical;
  /** 
   * Validation information for in parameters
   */
  _inputProfile?: Element
  /** 
   * Validation information for out parameters
   */
  outputProfile?: canonical;
  /** 
   * Validation information for out parameters
   */
  _outputProfile?: Element
  /** 
   * Parameters for the operation/query
   */
  parameter?: Array<OperationDefinitionParameter>;
  /** 
   * Define overloaded variants for when  generating code
   */
  overload?: Array<OperationDefinitionOverload>;
}

export interface OperationOutcomeIssue {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * fatal | error | warning | information
   */
  severity: code;
  /** 
   * fatal | error | warning | information
   */
  _severity?: Element
  /** 
   * Error or warning code
   */
  code: code;
  /** 
   * Error or warning code
   */
  _code?: Element
  /** 
   * Additional details about the error
   */
  details?: CodeableConcept;
  /** 
   * Additional diagnostic information about the issue
   */
  diagnostics?: string;
  /** 
   * Additional diagnostic information about the issue
   */
  _diagnostics?: Element
  /** 
   * Deprecated: Path of element(s) related to issue
   */
  location?: Array<string>;
  /** 
   * Deprecated: Path of element(s) related to issue
   */
  _location?: Array<Element>
  /** 
   * FHIRPath of element(s) related to issue
   */
  expression?: Array<string>;
  /** 
   * FHIRPath of element(s) related to issue
   */
  _expression?: Array<Element>
}
export interface OperationOutcome {
resourceType: "OperationOutcome"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * A single issue associated with the action
   */
  issue: Array<OperationOutcomeIssue>;
}

export interface OrganizationContact {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The type of contact
   */
  purpose?: CodeableConcept;
  /** 
   * A name associated with the contact
   */
  name?: HumanName;
  /** 
   * Contact details (telephone, email, etc.)  for a contact
   */
  telecom?: Array<ContactPoint>;
  /** 
   * Visiting or postal addresses for the contact
   */
  address?: Address;
}
export interface Organization {
resourceType: "Organization"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Identifies this organization  across multiple systems
   */
  identifier?: Array<Identifier>;
  /** 
   * Whether the organization's record is still in active use
   */
  active?: boolean;
  /** 
   * Whether the organization's record is still in active use
   */
  _active?: Element
  /** 
   * Kind of organization
   */
  type?: Array<CodeableConcept>;
  /** 
   * Name used for the organization
   */
  name?: string;
  /** 
   * Name used for the organization
   */
  _name?: Element
  /** 
   * A list of alternate names that the organization is known as, or was known as in the past
   */
  alias?: Array<string>;
  /** 
   * A list of alternate names that the organization is known as, or was known as in the past
   */
  _alias?: Array<Element>
  /** 
   * A contact detail for the organization
   */
  telecom?: Array<ContactPoint>;
  /** 
   * An address for the organization
   */
  address?: Array<Address>;
  /** 
   * The organization of which this organization forms a part
   */
  partOf?: Reference;
  /** 
   * Contact for the organization for a certain purpose
   */
  contact?: Array<OrganizationContact>;
  /** 
   * Technical endpoints providing access to services operated for the organization
   */
  endpoint?: Array<Reference>;
}

export interface OrganizationAffiliation {
resourceType: "OrganizationAffiliation"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business identifiers that are specific to this role
   */
  identifier?: Array<Identifier>;
  /** 
   * Whether this organization affiliation record is in active use
   */
  active?: boolean;
  /** 
   * Whether this organization affiliation record is in active use
   */
  _active?: Element
  /** 
   * The period during which the participatingOrganization is affiliated with the primary organization
   */
  period?: Period;
  /** 
   * Organization where the role is available
   */
  organization?: Reference;
  /** 
   * Organization that provides/performs the role (e.g. providing services or is a member of)
   */
  participatingOrganization?: Reference;
  /** 
   * Health insurance provider network in which the participatingOrganization provides the role's services (if defined) at the indicated locations (if defined)
   */
  network?: Array<Reference>;
  /** 
   * Definition of the role the participatingOrganization plays
   */
  code?: Array<CodeableConcept>;
  /** 
   * Specific specialty of the participatingOrganization in the context of the role
   */
  specialty?: Array<CodeableConcept>;
  /** 
   * The location(s) at which the role occurs
   */
  location?: Array<Reference>;
  /** 
   * Healthcare services provided through the role
   */
  healthcareService?: Array<Reference>;
  /** 
   * Contact details at the participatingOrganization relevant to this Affiliation
   */
  telecom?: Array<ContactPoint>;
  /** 
   * Technical endpoints providing access to services operated for this role
   */
  endpoint?: Array<Reference>;
}

export interface PackagedProductDefinitionLegalStatusOfSupply {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The actual status of supply. In what situation this package type may be supplied for use
   */
  code?: CodeableConcept;
  /** 
   * The place where the legal status of supply applies
   */
  jurisdiction?: CodeableConcept;
}
export interface PackagedProductDefinitionPackageShelfLifeStorage {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * This describes the shelf life, taking into account various scenarios such as shelf life of the packaged Medicinal Product itself, shelf life after transformation where necessary and shelf life after the first opening of a bottle, etc. The shelf life type shall be specified using an appropriate controlled vocabulary The controlled term and the controlled term identifier shall be specified
   */
  type?: CodeableConcept;
  /** 
   * The shelf life time period can be specified using a numerical value for the period of time and its unit of time measurement The unit of measurement shall be specified in accordance with ISO 11240 and the resulting terminology The symbol and the symbol identifier shall be used
   */
  periodDuration?: Duration;
  /** 
   * The shelf life time period can be specified using a numerical value for the period of time and its unit of time measurement The unit of measurement shall be specified in accordance with ISO 11240 and the resulting terminology The symbol and the symbol identifier shall be used
   */
  periodString?: string;
  /** 
   * The shelf life time period can be specified using a numerical value for the period of time and its unit of time measurement The unit of measurement shall be specified in accordance with ISO 11240 and the resulting terminology The symbol and the symbol identifier shall be used
   */
  _periodString?: Element
  /** 
   * Special precautions for storage, if any, can be specified using an appropriate controlled vocabulary. The controlled term and the controlled term identifier shall be specified
   */
  specialPrecautionsForStorage?: Array<CodeableConcept>;
}
export interface PackagedProductDefinitionPackageProperty {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * A code expressing the type of characteristic
   */
  type: CodeableConcept;
  /** 
   * A value for the characteristic
   */
  valueCodeableConcept?: CodeableConcept;
  /** 
   * A value for the characteristic
   */
  valueQuantity?: Quantity;
  /** 
   * A value for the characteristic
   */
  valueDate?: date;
  /** 
   * A value for the characteristic
   */
  _valueDate?: Element
  /** 
   * A value for the characteristic
   */
  valueBoolean?: boolean;
  /** 
   * A value for the characteristic
   */
  _valueBoolean?: Element
  /** 
   * A value for the characteristic
   */
  valueAttachment?: Attachment;
}
export interface PackagedProductDefinitionPackageContainedItem {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The actual item(s) of medication, as manufactured, or a device, or other medically related item (food, biologicals, raw materials, medical fluids, gases etc.), as contained in the package
   */
  item: CodeableReference;
  /** 
   * The number of this type of item within this packaging
   */
  amount?: Quantity;
}
export interface PackagedProductDefinitionPackage {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * An identifier that is specific to this particular part of the packaging. Including possibly a Data Carrier Identifier
   */
  identifier?: Array<Identifier>;
  /** 
   * The physical type of the container of the items
   */
  type?: CodeableConcept;
  /** 
   * The quantity of this level of packaging in the package that contains it (with the outermost level being 1)
   */
  quantity?: integer;
  /** 
   * The quantity of this level of packaging in the package that contains it (with the outermost level being 1)
   */
  _quantity?: Element
  /** 
   * Material type of the package item
   */
  material?: Array<CodeableConcept>;
  /** 
   * A possible alternate material for this part of the packaging, that is allowed to be used instead of the usual material
   */
  alternateMaterial?: Array<CodeableConcept>;
  /** 
   * Shelf Life and storage information
   */
  shelfLifeStorage?: Array<PackagedProductDefinitionPackageShelfLifeStorage>;
  /** 
   * Manufacturer of this package Item (multiple means these are all possible manufacturers)
   */
  manufacturer?: Array<Reference>;
  /** 
   * General characteristics of this item
   */
  property?: Array<PackagedProductDefinitionPackageProperty>;
  /** 
   * The item(s) within the packaging
   */
  containedItem?: Array<PackagedProductDefinitionPackageContainedItem>;
  /** 
   * Allows containers (and parts of containers) within containers, still a single packaged product
   */
  package?: Array<PackagedProductDefinitionPackage>;
}
export interface PackagedProductDefinition {
resourceType: "PackagedProductDefinition"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * A unique identifier for this package as whole
   */
  identifier?: Array<Identifier>;
  /** 
   * A name for this package. Typically as listed in a drug formulary, catalogue, inventory etc
   */
  name?: string;
  /** 
   * A name for this package. Typically as listed in a drug formulary, catalogue, inventory etc
   */
  _name?: Element
  /** 
   * A high level category e.g. medicinal product, raw material, shipping container etc
   */
  type?: CodeableConcept;
  /** 
   * The product that this is a pack for
   */
  packageFor?: Array<Reference>;
  /** 
   * The status within the lifecycle of this item. High level - not intended to duplicate details elsewhere e.g. legal status, or authorization/marketing status
   */
  status?: CodeableConcept;
  /** 
   * The date at which the given status became applicable
   */
  statusDate?: dateTime;
  /** 
   * The date at which the given status became applicable
   */
  _statusDate?: Element
  /** 
   * A total of the complete count of contained items of a particular type/form, independent of sub-packaging or organization. This can be considered as the pack size
   */
  containedItemQuantity?: Array<Quantity>;
  /** 
   * Textual description. Note that this is not the name of the package or product
   */
  description?: markdown;
  /** 
   * Textual description. Note that this is not the name of the package or product
   */
  _description?: Element
  /** 
   * The legal status of supply of the packaged item as classified by the regulator
   */
  legalStatusOfSupply?: Array<PackagedProductDefinitionLegalStatusOfSupply>;
  /** 
   * Allows specifying that an item is on the market for sale, or that it is not available, and the dates and locations associated
   */
  marketingStatus?: Array<MarketingStatus>;
  /** 
   * Allows the key features to be recorded, such as "hospital pack", "nurse prescribable"
   */
  characteristic?: Array<CodeableConcept>;
  /** 
   * If the drug product is supplied with another item such as a diluent or adjuvant
   */
  copackagedIndicator?: boolean;
  /** 
   * If the drug product is supplied with another item such as a diluent or adjuvant
   */
  _copackagedIndicator?: Element
  /** 
   * Manufacturer of this package type (multiple means these are all possible manufacturers)
   */
  manufacturer?: Array<Reference>;
  /** 
   * A packaging item, as a container for medically related items, possibly with other packaging items within, or a packaging component, such as bottle cap
   */
  package?: PackagedProductDefinitionPackage;
}

export interface ParametersParameter {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Name from the definition
   */
  name: string;
  /** 
   * Name from the definition
   */
  _name?: Element
  /** 
   * If parameter is a data type
   */
  valueBase64Binary?: base64Binary;
  /** 
   * If parameter is a data type
   */
  _valueBase64Binary?: Element
  /** 
   * If parameter is a data type
   */
  valueBoolean?: boolean;
  /** 
   * If parameter is a data type
   */
  _valueBoolean?: Element
  /** 
   * If parameter is a data type
   */
  valueCanonical?: canonical;
  /** 
   * If parameter is a data type
   */
  _valueCanonical?: Element
  /** 
   * If parameter is a data type
   */
  valueCode?: code;
  /** 
   * If parameter is a data type
   */
  _valueCode?: Element
  /** 
   * If parameter is a data type
   */
  valueDate?: date;
  /** 
   * If parameter is a data type
   */
  _valueDate?: Element
  /** 
   * If parameter is a data type
   */
  valueDateTime?: dateTime;
  /** 
   * If parameter is a data type
   */
  _valueDateTime?: Element
  /** 
   * If parameter is a data type
   */
  valueDecimal?: decimal;
  /** 
   * If parameter is a data type
   */
  _valueDecimal?: Element
  /** 
   * If parameter is a data type
   */
  valueId?: id;
  /** 
   * If parameter is a data type
   */
  _valueId?: Element
  /** 
   * If parameter is a data type
   */
  valueInstant?: instant;
  /** 
   * If parameter is a data type
   */
  _valueInstant?: Element
  /** 
   * If parameter is a data type
   */
  valueInteger?: integer;
  /** 
   * If parameter is a data type
   */
  _valueInteger?: Element
  /** 
   * If parameter is a data type
   */
  valueMarkdown?: markdown;
  /** 
   * If parameter is a data type
   */
  _valueMarkdown?: Element
  /** 
   * If parameter is a data type
   */
  valueOid?: oid;
  /** 
   * If parameter is a data type
   */
  _valueOid?: Element
  /** 
   * If parameter is a data type
   */
  valuePositiveInt?: positiveInt;
  /** 
   * If parameter is a data type
   */
  _valuePositiveInt?: Element
  /** 
   * If parameter is a data type
   */
  valueString?: string;
  /** 
   * If parameter is a data type
   */
  _valueString?: Element
  /** 
   * If parameter is a data type
   */
  valueTime?: time;
  /** 
   * If parameter is a data type
   */
  _valueTime?: Element
  /** 
   * If parameter is a data type
   */
  valueUnsignedInt?: unsignedInt;
  /** 
   * If parameter is a data type
   */
  _valueUnsignedInt?: Element
  /** 
   * If parameter is a data type
   */
  valueUri?: uri;
  /** 
   * If parameter is a data type
   */
  _valueUri?: Element
  /** 
   * If parameter is a data type
   */
  valueUrl?: url;
  /** 
   * If parameter is a data type
   */
  _valueUrl?: Element
  /** 
   * If parameter is a data type
   */
  valueUuid?: uuid;
  /** 
   * If parameter is a data type
   */
  _valueUuid?: Element
  /** 
   * If parameter is a data type
   */
  valueAddress?: Address;
  /** 
   * If parameter is a data type
   */
  valueAge?: Age;
  /** 
   * If parameter is a data type
   */
  valueAnnotation?: Annotation;
  /** 
   * If parameter is a data type
   */
  valueAttachment?: Attachment;
  /** 
   * If parameter is a data type
   */
  valueCodeableConcept?: CodeableConcept;
  /** 
   * If parameter is a data type
   */
  valueCoding?: Coding;
  /** 
   * If parameter is a data type
   */
  valueContactPoint?: ContactPoint;
  /** 
   * If parameter is a data type
   */
  valueCount?: Count;
  /** 
   * If parameter is a data type
   */
  valueDistance?: Distance;
  /** 
   * If parameter is a data type
   */
  valueDuration?: Duration;
  /** 
   * If parameter is a data type
   */
  valueHumanName?: HumanName;
  /** 
   * If parameter is a data type
   */
  valueIdentifier?: Identifier;
  /** 
   * If parameter is a data type
   */
  valueMoney?: Money;
  /** 
   * If parameter is a data type
   */
  valuePeriod?: Period;
  /** 
   * If parameter is a data type
   */
  valueQuantity?: Quantity;
  /** 
   * If parameter is a data type
   */
  valueRange?: Range;
  /** 
   * If parameter is a data type
   */
  valueRatio?: Ratio;
  /** 
   * If parameter is a data type
   */
  valueReference?: Reference;
  /** 
   * If parameter is a data type
   */
  valueSampledData?: SampledData;
  /** 
   * If parameter is a data type
   */
  valueSignature?: Signature;
  /** 
   * If parameter is a data type
   */
  valueTiming?: Timing;
  /** 
   * If parameter is a data type
   */
  valueContactDetail?: ContactDetail;
  /** 
   * If parameter is a data type
   */
  valueContributor?: Contributor;
  /** 
   * If parameter is a data type
   */
  valueDataRequirement?: DataRequirement;
  /** 
   * If parameter is a data type
   */
  valueExpression?: Expression;
  /** 
   * If parameter is a data type
   */
  valueParameterDefinition?: ParameterDefinition;
  /** 
   * If parameter is a data type
   */
  valueRelatedArtifact?: RelatedArtifact;
  /** 
   * If parameter is a data type
   */
  valueTriggerDefinition?: TriggerDefinition;
  /** 
   * If parameter is a data type
   */
  valueUsageContext?: UsageContext;
  /** 
   * If parameter is a data type
   */
  valueDosage?: Dosage;
  /** 
   * If parameter is a data type
   */
  valueMeta?: Meta;
  /** 
   * If parameter is a whole resource
   */
  resource?: Resource;
  /** 
   * Named part of a multi-part parameter
   */
  part?: Array<ParametersParameter>;
}
export interface Parameters {
resourceType: "Parameters"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Operation Parameter
   */
  parameter?: Array<ParametersParameter>;
}

export interface PatientContact {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The kind of relationship
   */
  relationship?: Array<CodeableConcept>;
  /** 
   * A name associated with the contact person
   */
  name?: HumanName;
  /** 
   * A contact detail for the person
   */
  telecom?: Array<ContactPoint>;
  /** 
   * Address for the contact person
   */
  address?: Address;
  /** 
   * male | female | other | unknown
   */
  gender?: code;
  /** 
   * male | female | other | unknown
   */
  _gender?: Element
  /** 
   * Organization that is associated with the contact
   */
  organization?: Reference;
  /** 
   * The period during which this contact person or organization is valid to be contacted relating to this patient
   */
  period?: Period;
}
export interface PatientCommunication {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The language which can be used to communicate with the patient about his or her health
   */
  language: CodeableConcept;
  /** 
   * Language preference indicator
   */
  preferred?: boolean;
  /** 
   * Language preference indicator
   */
  _preferred?: Element
}
export interface PatientLink {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The other patient or related person resource that the link refers to
   */
  other: Reference;
  /** 
   * replaced-by | replaces | refer | seealso
   */
  type: code;
  /** 
   * replaced-by | replaces | refer | seealso
   */
  _type?: Element
}
export interface Patient {
resourceType: "Patient"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * An identifier for this patient
   */
  identifier?: Array<Identifier>;
  /** 
   * Whether this patient's record is in active use
   */
  active?: boolean;
  /** 
   * Whether this patient's record is in active use
   */
  _active?: Element
  /** 
   * A name associated with the patient
   */
  name?: Array<HumanName>;
  /** 
   * A contact detail for the individual
   */
  telecom?: Array<ContactPoint>;
  /** 
   * male | female | other | unknown
   */
  gender?: code;
  /** 
   * male | female | other | unknown
   */
  _gender?: Element
  /** 
   * The date of birth for the individual
   */
  birthDate?: date;
  /** 
   * The date of birth for the individual
   */
  _birthDate?: Element
  /** 
   * Indicates if the individual is deceased or not
   */
  deceasedBoolean?: boolean;
  /** 
   * Indicates if the individual is deceased or not
   */
  _deceasedBoolean?: Element
  /** 
   * Indicates if the individual is deceased or not
   */
  deceasedDateTime?: dateTime;
  /** 
   * Indicates if the individual is deceased or not
   */
  _deceasedDateTime?: Element
  /** 
   * An address for the individual
   */
  address?: Array<Address>;
  /** 
   * Marital (civil) status of a patient
   */
  maritalStatus?: CodeableConcept;
  /** 
   * Whether patient is part of a multiple birth
   */
  multipleBirthBoolean?: boolean;
  /** 
   * Whether patient is part of a multiple birth
   */
  _multipleBirthBoolean?: Element
  /** 
   * Whether patient is part of a multiple birth
   */
  multipleBirthInteger?: integer;
  /** 
   * Whether patient is part of a multiple birth
   */
  _multipleBirthInteger?: Element
  /** 
   * Image of the patient
   */
  photo?: Array<Attachment>;
  /** 
   * A contact party (e.g. guardian, partner, friend) for the patient
   */
  contact?: Array<PatientContact>;
  /** 
   * A language which may be used to communicate with the patient about his or her health
   */
  communication?: Array<PatientCommunication>;
  /** 
   * Patient's nominated primary care provider
   */
  generalPractitioner?: Array<Reference>;
  /** 
   * Organization that is the custodian of the patient record
   */
  managingOrganization?: Reference;
  /** 
   * Link to another patient resource that concerns the same actual person
   */
  link?: Array<PatientLink>;
}

export interface PaymentNotice {
resourceType: "PaymentNotice"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business Identifier for the payment noctice
   */
  identifier?: Array<Identifier>;
  /** 
   * active | cancelled | draft | entered-in-error
   */
  status: code;
  /** 
   * active | cancelled | draft | entered-in-error
   */
  _status?: Element
  /** 
   * Request reference
   */
  request?: Reference;
  /** 
   * Response reference
   */
  response?: Reference;
  /** 
   * Creation date
   */
  created: dateTime;
  /** 
   * Creation date
   */
  _created?: Element
  /** 
   * Responsible practitioner
   */
  provider?: Reference;
  /** 
   * Payment reference
   */
  payment: Reference;
  /** 
   * Payment or clearing date
   */
  paymentDate?: date;
  /** 
   * Payment or clearing date
   */
  _paymentDate?: Element
  /** 
   * Party being paid
   */
  payee?: Reference;
  /** 
   * Party being notified
   */
  recipient: Reference;
  /** 
   * Monetary amount of the payment
   */
  amount: Money;
  /** 
   * Issued or cleared Status of the payment
   */
  paymentStatus?: CodeableConcept;
}

export interface PaymentReconciliationDetail {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business identifier of the payment detail
   */
  identifier?: Identifier;
  /** 
   * Business identifier of the prior payment detail
   */
  predecessor?: Identifier;
  /** 
   * Category of payment
   */
  type: CodeableConcept;
  /** 
   * Request giving rise to the payment
   */
  request?: Reference;
  /** 
   * Submitter of the request
   */
  submitter?: Reference;
  /** 
   * Response committing to a payment
   */
  response?: Reference;
  /** 
   * Date of commitment to pay
   */
  date?: date;
  /** 
   * Date of commitment to pay
   */
  _date?: Element
  /** 
   * Contact for the response
   */
  responsible?: Reference;
  /** 
   * Recipient of the payment
   */
  payee?: Reference;
  /** 
   * Amount allocated to this payable
   */
  amount?: Money;
}
export interface PaymentReconciliationProcessNote {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * display | print | printoper
   */
  type?: code;
  /** 
   * display | print | printoper
   */
  _type?: Element
  /** 
   * Note explanatory text
   */
  text?: string;
  /** 
   * Note explanatory text
   */
  _text?: Element
}
export interface PaymentReconciliation {
resourceType: "PaymentReconciliation"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business Identifier for a payment reconciliation
   */
  identifier?: Array<Identifier>;
  /** 
   * active | cancelled | draft | entered-in-error
   */
  status: code;
  /** 
   * active | cancelled | draft | entered-in-error
   */
  _status?: Element
  /** 
   * Period covered
   */
  period?: Period;
  /** 
   * Creation date
   */
  created: dateTime;
  /** 
   * Creation date
   */
  _created?: Element
  /** 
   * Party generating payment
   */
  paymentIssuer?: Reference;
  /** 
   * Reference to requesting resource
   */
  request?: Reference;
  /** 
   * Responsible practitioner
   */
  requestor?: Reference;
  /** 
   * queued | complete | error | partial
   */
  outcome?: code;
  /** 
   * queued | complete | error | partial
   */
  _outcome?: Element
  /** 
   * Disposition message
   */
  disposition?: string;
  /** 
   * Disposition message
   */
  _disposition?: Element
  /** 
   * When payment issued
   */
  paymentDate: date;
  /** 
   * When payment issued
   */
  _paymentDate?: Element
  /** 
   * Total amount of Payment
   */
  paymentAmount: Money;
  /** 
   * Business identifier for the payment
   */
  paymentIdentifier?: Identifier;
  /** 
   * Settlement particulars
   */
  detail?: Array<PaymentReconciliationDetail>;
  /** 
   * Printed form identifier
   */
  formCode?: CodeableConcept;
  /** 
   * Note concerning processing
   */
  processNote?: Array<PaymentReconciliationProcessNote>;
}

export interface PersonLink {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The resource to which this actual person is associated
   */
  target: Reference;
  /** 
   * level1 | level2 | level3 | level4
   */
  assurance?: code;
  /** 
   * level1 | level2 | level3 | level4
   */
  _assurance?: Element
}
export interface Person {
resourceType: "Person"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * A human identifier for this person
   */
  identifier?: Array<Identifier>;
  /** 
   * A name associated with the person
   */
  name?: Array<HumanName>;
  /** 
   * A contact detail for the person
   */
  telecom?: Array<ContactPoint>;
  /** 
   * male | female | other | unknown
   */
  gender?: code;
  /** 
   * male | female | other | unknown
   */
  _gender?: Element
  /** 
   * The date on which the person was born
   */
  birthDate?: date;
  /** 
   * The date on which the person was born
   */
  _birthDate?: Element
  /** 
   * One or more addresses for the person
   */
  address?: Array<Address>;
  /** 
   * Image of the person
   */
  photo?: Attachment;
  /** 
   * The organization that is the custodian of the person record
   */
  managingOrganization?: Reference;
  /** 
   * This person's record is in active use
   */
  active?: boolean;
  /** 
   * This person's record is in active use
   */
  _active?: Element
  /** 
   * Link to a resource that concerns the same actual person
   */
  link?: Array<PersonLink>;
}

export interface PlanDefinitionGoalTarget {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The parameter whose value is to be tracked
   */
  measure?: CodeableConcept;
  /** 
   * The target value to be achieved
   */
  detailQuantity?: Quantity;
  /** 
   * The target value to be achieved
   */
  detailRange?: Range;
  /** 
   * The target value to be achieved
   */
  detailCodeableConcept?: CodeableConcept;
  /** 
   * Reach goal within
   */
  due?: Duration;
}
export interface PlanDefinitionGoal {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * E.g. Treatment, dietary, behavioral
   */
  category?: CodeableConcept;
  /** 
   * Code or text describing the goal
   */
  description: CodeableConcept;
  /** 
   * high-priority | medium-priority | low-priority
   */
  priority?: CodeableConcept;
  /** 
   * When goal pursuit begins
   */
  start?: CodeableConcept;
  /** 
   * What does the goal address
   */
  addresses?: Array<CodeableConcept>;
  /** 
   * Supporting documentation for the goal
   */
  documentation?: Array<RelatedArtifact>;
  /** 
   * Target outcome for the goal
   */
  target?: Array<PlanDefinitionGoalTarget>;
}
export interface PlanDefinitionActionCondition {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * applicability | start | stop
   */
  kind: code;
  /** 
   * applicability | start | stop
   */
  _kind?: Element
  /** 
   * Boolean-valued expression
   */
  expression?: Expression;
}
export interface PlanDefinitionActionRelatedAction {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * What action is this related to
   */
  actionId: id;
  /** 
   * What action is this related to
   */
  _actionId?: Element
  /** 
   * before-start | before | before-end | concurrent-with-start | concurrent | concurrent-with-end | after-start | after | after-end
   */
  relationship: code;
  /** 
   * before-start | before | before-end | concurrent-with-start | concurrent | concurrent-with-end | after-start | after | after-end
   */
  _relationship?: Element
  /** 
   * Time offset for the relationship
   */
  offsetDuration?: Duration;
  /** 
   * Time offset for the relationship
   */
  offsetRange?: Range;
}
export interface PlanDefinitionActionParticipant {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * patient | practitioner | related-person | device
   */
  type: code;
  /** 
   * patient | practitioner | related-person | device
   */
  _type?: Element
  /** 
   * E.g. Nurse, Surgeon, Parent
   */
  role?: CodeableConcept;
}
export interface PlanDefinitionActionDynamicValue {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The path to the element to be set dynamically
   */
  path?: string;
  /** 
   * The path to the element to be set dynamically
   */
  _path?: Element
  /** 
   * An expression that provides the dynamic value for the customization
   */
  expression?: Expression;
}
export interface PlanDefinitionAction {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * User-visible prefix for the action (e.g. 1. or A.)
   */
  prefix?: string;
  /** 
   * User-visible prefix for the action (e.g. 1. or A.)
   */
  _prefix?: Element
  /** 
   * User-visible title
   */
  title?: string;
  /** 
   * User-visible title
   */
  _title?: Element
  /** 
   * Brief description of the action
   */
  description?: string;
  /** 
   * Brief description of the action
   */
  _description?: Element
  /** 
   * Static text equivalent of the action, used if the dynamic aspects cannot be interpreted by the receiving system
   */
  textEquivalent?: string;
  /** 
   * Static text equivalent of the action, used if the dynamic aspects cannot be interpreted by the receiving system
   */
  _textEquivalent?: Element
  /** 
   * routine | urgent | asap | stat
   */
  priority?: code;
  /** 
   * routine | urgent | asap | stat
   */
  _priority?: Element
  /** 
   * Code representing the meaning of the action or sub-actions
   */
  code?: Array<CodeableConcept>;
  /** 
   * Why the action should be performed
   */
  reason?: Array<CodeableConcept>;
  /** 
   * Supporting documentation for the intended performer of the action
   */
  documentation?: Array<RelatedArtifact>;
  /** 
   * What goals this action supports
   */
  goalId?: Array<id>;
  /** 
   * What goals this action supports
   */
  _goalId?: Array<Element>
  /** 
   * Type of individual the action is focused on
   */
  subjectCodeableConcept?: CodeableConcept;
  /** 
   * Type of individual the action is focused on
   */
  subjectReference?: Reference;
  /** 
   * Type of individual the action is focused on
   */
  subjectCanonical?: canonical;
  /** 
   * Type of individual the action is focused on
   */
  _subjectCanonical?: Element
  /** 
   * When the action should be triggered
   */
  trigger?: Array<TriggerDefinition>;
  /** 
   * Whether or not the action is applicable
   */
  condition?: Array<PlanDefinitionActionCondition>;
  /** 
   * Input data requirements
   */
  input?: Array<DataRequirement>;
  /** 
   * Output data definition
   */
  output?: Array<DataRequirement>;
  /** 
   * Relationship to another action
   */
  relatedAction?: Array<PlanDefinitionActionRelatedAction>;
  /** 
   * When the action should take place
   */
  timingDateTime?: dateTime;
  /** 
   * When the action should take place
   */
  _timingDateTime?: Element
  /** 
   * When the action should take place
   */
  timingAge?: Age;
  /** 
   * When the action should take place
   */
  timingPeriod?: Period;
  /** 
   * When the action should take place
   */
  timingDuration?: Duration;
  /** 
   * When the action should take place
   */
  timingRange?: Range;
  /** 
   * When the action should take place
   */
  timingTiming?: Timing;
  /** 
   * Who should participate in the action
   */
  participant?: Array<PlanDefinitionActionParticipant>;
  /** 
   * create | update | remove | fire-event
   */
  type?: CodeableConcept;
  /** 
   * visual-group | logical-group | sentence-group
   */
  groupingBehavior?: code;
  /** 
   * visual-group | logical-group | sentence-group
   */
  _groupingBehavior?: Element
  /** 
   * any | all | all-or-none | exactly-one | at-most-one | one-or-more
   */
  selectionBehavior?: code;
  /** 
   * any | all | all-or-none | exactly-one | at-most-one | one-or-more
   */
  _selectionBehavior?: Element
  /** 
   * must | could | must-unless-documented
   */
  requiredBehavior?: code;
  /** 
   * must | could | must-unless-documented
   */
  _requiredBehavior?: Element
  /** 
   * yes | no
   */
  precheckBehavior?: code;
  /** 
   * yes | no
   */
  _precheckBehavior?: Element
  /** 
   * single | multiple
   */
  cardinalityBehavior?: code;
  /** 
   * single | multiple
   */
  _cardinalityBehavior?: Element
  /** 
   * Description of the activity to be performed
   */
  definitionCanonical?: canonical;
  /** 
   * Description of the activity to be performed
   */
  _definitionCanonical?: Element
  /** 
   * Description of the activity to be performed
   */
  definitionUri?: uri;
  /** 
   * Description of the activity to be performed
   */
  _definitionUri?: Element
  /** 
   * Transform to apply the template
   */
  transform?: canonical;
  /** 
   * Transform to apply the template
   */
  _transform?: Element
  /** 
   * Dynamic aspects of the definition
   */
  dynamicValue?: Array<PlanDefinitionActionDynamicValue>;
  /** 
   * A sub-action
   */
  action?: Array<PlanDefinitionAction>;
}
export interface PlanDefinition {
resourceType: "PlanDefinition"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Canonical identifier for this plan definition, represented as a URI (globally unique)
   */
  url?: uri;
  /** 
   * Canonical identifier for this plan definition, represented as a URI (globally unique)
   */
  _url?: Element
  /** 
   * Additional identifier for the plan definition
   */
  identifier?: Array<Identifier>;
  /** 
   * Business version of the plan definition
   */
  version?: string;
  /** 
   * Business version of the plan definition
   */
  _version?: Element
  /** 
   * Name for this plan definition (computer friendly)
   */
  name?: string;
  /** 
   * Name for this plan definition (computer friendly)
   */
  _name?: Element
  /** 
   * Name for this plan definition (human friendly)
   */
  title?: string;
  /** 
   * Name for this plan definition (human friendly)
   */
  _title?: Element
  /** 
   * Subordinate title of the plan definition
   */
  subtitle?: string;
  /** 
   * Subordinate title of the plan definition
   */
  _subtitle?: Element
  /** 
   * order-set | clinical-protocol | eca-rule | workflow-definition
   */
  type?: CodeableConcept;
  /** 
   * draft | active | retired | unknown
   */
  status: code;
  /** 
   * draft | active | retired | unknown
   */
  _status?: Element
  /** 
   * For testing purposes, not real usage
   */
  experimental?: boolean;
  /** 
   * For testing purposes, not real usage
   */
  _experimental?: Element
  /** 
   * Type of individual the plan definition is focused on
   */
  subjectCodeableConcept?: CodeableConcept;
  /** 
   * Type of individual the plan definition is focused on
   */
  subjectReference?: Reference;
  /** 
   * Type of individual the plan definition is focused on
   */
  subjectCanonical?: canonical;
  /** 
   * Type of individual the plan definition is focused on
   */
  _subjectCanonical?: Element
  /** 
   * Date last changed
   */
  date?: dateTime;
  /** 
   * Date last changed
   */
  _date?: Element
  /** 
   * Name of the publisher (organization or individual)
   */
  publisher?: string;
  /** 
   * Name of the publisher (organization or individual)
   */
  _publisher?: Element
  /** 
   * Contact details for the publisher
   */
  contact?: Array<ContactDetail>;
  /** 
   * Natural language description of the plan definition
   */
  description?: markdown;
  /** 
   * Natural language description of the plan definition
   */
  _description?: Element
  /** 
   * The context that the content is intended to support
   */
  useContext?: Array<UsageContext>;
  /** 
   * Intended jurisdiction for plan definition (if applicable)
   */
  jurisdiction?: Array<CodeableConcept>;
  /** 
   * Why this plan definition is defined
   */
  purpose?: markdown;
  /** 
   * Why this plan definition is defined
   */
  _purpose?: Element
  /** 
   * Describes the clinical usage of the plan
   */
  usage?: string;
  /** 
   * Describes the clinical usage of the plan
   */
  _usage?: Element
  /** 
   * Use and/or publishing restrictions
   */
  copyright?: markdown;
  /** 
   * Use and/or publishing restrictions
   */
  _copyright?: Element
  /** 
   * When the plan definition was approved by publisher
   */
  approvalDate?: date;
  /** 
   * When the plan definition was approved by publisher
   */
  _approvalDate?: Element
  /** 
   * When the plan definition was last reviewed
   */
  lastReviewDate?: date;
  /** 
   * When the plan definition was last reviewed
   */
  _lastReviewDate?: Element
  /** 
   * When the plan definition is expected to be used
   */
  effectivePeriod?: Period;
  /** 
   * E.g. Education, Treatment, Assessment
   */
  topic?: Array<CodeableConcept>;
  /** 
   * Who authored the content
   */
  author?: Array<ContactDetail>;
  /** 
   * Who edited the content
   */
  editor?: Array<ContactDetail>;
  /** 
   * Who reviewed the content
   */
  reviewer?: Array<ContactDetail>;
  /** 
   * Who endorsed the content
   */
  endorser?: Array<ContactDetail>;
  /** 
   * Additional documentation, citations
   */
  relatedArtifact?: Array<RelatedArtifact>;
  /** 
   * Logic used by the plan definition
   */
  library?: Array<canonical>;
  /** 
   * Logic used by the plan definition
   */
  _library?: Array<Element>
  /** 
   * What the plan is trying to accomplish
   */
  goal?: Array<PlanDefinitionGoal>;
  /** 
   * Action defined by the plan
   */
  action?: Array<PlanDefinitionAction>;
}

export interface PractitionerQualification {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * An identifier for this qualification for the practitioner
   */
  identifier?: Array<Identifier>;
  /** 
   * Coded representation of the qualification
   */
  code: CodeableConcept;
  /** 
   * Period during which the qualification is valid
   */
  period?: Period;
  /** 
   * Organization that regulates and issues the qualification
   */
  issuer?: Reference;
}
export interface Practitioner {
resourceType: "Practitioner"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * An identifier for the person as this agent
   */
  identifier?: Array<Identifier>;
  /** 
   * Whether this practitioner's record is in active use
   */
  active?: boolean;
  /** 
   * Whether this practitioner's record is in active use
   */
  _active?: Element
  /** 
   * The name(s) associated with the practitioner
   */
  name?: Array<HumanName>;
  /** 
   * A contact detail for the practitioner (that apply to all roles)
   */
  telecom?: Array<ContactPoint>;
  /** 
   * Address(es) of the practitioner that are not role specific (typically home address)
   */
  address?: Array<Address>;
  /** 
   * male | female | other | unknown
   */
  gender?: code;
  /** 
   * male | female | other | unknown
   */
  _gender?: Element
  /** 
   * The date  on which the practitioner was born
   */
  birthDate?: date;
  /** 
   * The date  on which the practitioner was born
   */
  _birthDate?: Element
  /** 
   * Image of the person
   */
  photo?: Array<Attachment>;
  /** 
   * Certification, licenses, or training pertaining to the provision of care
   */
  qualification?: Array<PractitionerQualification>;
  /** 
   * A language the practitioner can use in patient communication
   */
  communication?: Array<CodeableConcept>;
}

export interface PractitionerRoleAvailableTime {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * mon | tue | wed | thu | fri | sat | sun
   */
  daysOfWeek?: Array<code>;
  /** 
   * mon | tue | wed | thu | fri | sat | sun
   */
  _daysOfWeek?: Array<Element>
  /** 
   * Always available? e.g. 24 hour service
   */
  allDay?: boolean;
  /** 
   * Always available? e.g. 24 hour service
   */
  _allDay?: Element
  /** 
   * Opening time of day (ignored if allDay = true)
   */
  availableStartTime?: time;
  /** 
   * Opening time of day (ignored if allDay = true)
   */
  _availableStartTime?: Element
  /** 
   * Closing time of day (ignored if allDay = true)
   */
  availableEndTime?: time;
  /** 
   * Closing time of day (ignored if allDay = true)
   */
  _availableEndTime?: Element
}
export interface PractitionerRoleNotAvailable {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Reason presented to the user explaining why time not available
   */
  description: string;
  /** 
   * Reason presented to the user explaining why time not available
   */
  _description?: Element
  /** 
   * Service not available from this date
   */
  during?: Period;
}
export interface PractitionerRole {
resourceType: "PractitionerRole"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business Identifiers that are specific to a role/location
   */
  identifier?: Array<Identifier>;
  /** 
   * Whether this practitioner role record is in active use
   */
  active?: boolean;
  /** 
   * Whether this practitioner role record is in active use
   */
  _active?: Element
  /** 
   * The period during which the practitioner is authorized to perform in these role(s)
   */
  period?: Period;
  /** 
   * Practitioner that is able to provide the defined services for the organization
   */
  practitioner?: Reference;
  /** 
   * Organization where the roles are available
   */
  organization?: Reference;
  /** 
   * Roles which this practitioner may perform
   */
  code?: Array<CodeableConcept>;
  /** 
   * Specific specialty of the practitioner
   */
  specialty?: Array<CodeableConcept>;
  /** 
   * The location(s) at which this practitioner provides care
   */
  location?: Array<Reference>;
  /** 
   * The list of healthcare services that this worker provides for this role's Organization/Location(s)
   */
  healthcareService?: Array<Reference>;
  /** 
   * Contact details that are specific to the role/location/service
   */
  telecom?: Array<ContactPoint>;
  /** 
   * Times the Service Site is available
   */
  availableTime?: Array<PractitionerRoleAvailableTime>;
  /** 
   * Not available during this time due to provided reason
   */
  notAvailable?: Array<PractitionerRoleNotAvailable>;
  /** 
   * Description of availability exceptions
   */
  availabilityExceptions?: string;
  /** 
   * Description of availability exceptions
   */
  _availabilityExceptions?: Element
  /** 
   * Technical endpoints providing access to services operated for the practitioner with this role
   */
  endpoint?: Array<Reference>;
}

export interface ProcedurePerformer {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Type of performance
   */
  function?: CodeableConcept;
  /** 
   * The reference to the practitioner
   */
  actor: Reference;
  /** 
   * Organization the device or practitioner was acting for
   */
  onBehalfOf?: Reference;
}
export interface ProcedureFocalDevice {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Kind of change to device
   */
  action?: CodeableConcept;
  /** 
   * Device that was changed
   */
  manipulated: Reference;
}
export interface Procedure {
resourceType: "Procedure"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * External Identifiers for this procedure
   */
  identifier?: Array<Identifier>;
  /** 
   * Instantiates FHIR protocol or definition
   */
  instantiatesCanonical?: Array<canonical>;
  /** 
   * Instantiates FHIR protocol or definition
   */
  _instantiatesCanonical?: Array<Element>
  /** 
   * Instantiates external protocol or definition
   */
  instantiatesUri?: Array<uri>;
  /** 
   * Instantiates external protocol or definition
   */
  _instantiatesUri?: Array<Element>
  /** 
   * A request for this procedure
   */
  basedOn?: Array<Reference>;
  /** 
   * Part of referenced event
   */
  partOf?: Array<Reference>;
  /** 
   * preparation | in-progress | not-done | on-hold | stopped | completed | entered-in-error | unknown
   */
  status: code;
  /** 
   * preparation | in-progress | not-done | on-hold | stopped | completed | entered-in-error | unknown
   */
  _status?: Element
  /** 
   * Reason for current status
   */
  statusReason?: CodeableConcept;
  /** 
   * Classification of the procedure
   */
  category?: CodeableConcept;
  /** 
   * Identification of the procedure
   */
  code?: CodeableConcept;
  /** 
   * Who the procedure was performed on
   */
  subject: Reference;
  /** 
   * Encounter created as part of
   */
  encounter?: Reference;
  /** 
   * When the procedure was performed
   */
  performedDateTime?: dateTime;
  /** 
   * When the procedure was performed
   */
  _performedDateTime?: Element
  /** 
   * When the procedure was performed
   */
  performedPeriod?: Period;
  /** 
   * When the procedure was performed
   */
  performedString?: string;
  /** 
   * When the procedure was performed
   */
  _performedString?: Element
  /** 
   * When the procedure was performed
   */
  performedAge?: Age;
  /** 
   * When the procedure was performed
   */
  performedRange?: Range;
  /** 
   * Who recorded the procedure
   */
  recorder?: Reference;
  /** 
   * Person who asserts this procedure
   */
  asserter?: Reference;
  /** 
   * The people who performed the procedure
   */
  performer?: Array<ProcedurePerformer>;
  /** 
   * Where the procedure happened
   */
  location?: Reference;
  /** 
   * Coded reason procedure performed
   */
  reasonCode?: Array<CodeableConcept>;
  /** 
   * The justification that the procedure was performed
   */
  reasonReference?: Array<Reference>;
  /** 
   * Target body sites
   */
  bodySite?: Array<CodeableConcept>;
  /** 
   * The result of procedure
   */
  outcome?: CodeableConcept;
  /** 
   * Any report resulting from the procedure
   */
  report?: Array<Reference>;
  /** 
   * Complication following the procedure
   */
  complication?: Array<CodeableConcept>;
  /** 
   * A condition that is a result of the procedure
   */
  complicationDetail?: Array<Reference>;
  /** 
   * Instructions for follow up
   */
  followUp?: Array<CodeableConcept>;
  /** 
   * Additional information about the procedure
   */
  note?: Array<Annotation>;
  /** 
   * Manipulated, implanted, or removed device
   */
  focalDevice?: Array<ProcedureFocalDevice>;
  /** 
   * Items used during procedure
   */
  usedReference?: Array<Reference>;
  /** 
   * Coded items used during the procedure
   */
  usedCode?: Array<CodeableConcept>;
}

export interface ProvenanceAgent {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * How the agent participated
   */
  type?: CodeableConcept;
  /** 
   * What the agents role was
   */
  role?: Array<CodeableConcept>;
  /** 
   * Who participated
   */
  who: Reference;
  /** 
   * Who the agent is representing
   */
  onBehalfOf?: Reference;
}
export interface ProvenanceEntity {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * derivation | revision | quotation | source | removal
   */
  role: code;
  /** 
   * derivation | revision | quotation | source | removal
   */
  _role?: Element
  /** 
   * Identity of entity
   */
  what: Reference;
  /** 
   * Entity is attributed to this agent
   */
  agent?: Array<ProvenanceAgent>;
}
export interface Provenance {
resourceType: "Provenance"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Target Reference(s) (usually version specific)
   */
  target: Array<Reference>;
  /** 
   * When the activity occurred
   */
  occurredPeriod?: Period;
  /** 
   * When the activity occurred
   */
  occurredDateTime?: dateTime;
  /** 
   * When the activity occurred
   */
  _occurredDateTime?: Element
  /** 
   * When the activity was recorded / updated
   */
  recorded: instant;
  /** 
   * When the activity was recorded / updated
   */
  _recorded?: Element
  /** 
   * Policy or plan the activity was defined by
   */
  policy?: Array<uri>;
  /** 
   * Policy or plan the activity was defined by
   */
  _policy?: Array<Element>
  /** 
   * Where the activity occurred, if relevant
   */
  location?: Reference;
  /** 
   * Reason the activity is occurring
   */
  reason?: Array<CodeableConcept>;
  /** 
   * Activity that occurred
   */
  activity?: CodeableConcept;
  /** 
   * Actor involved
   */
  agent: Array<ProvenanceAgent>;
  /** 
   * An entity used in this activity
   */
  entity?: Array<ProvenanceEntity>;
  /** 
   * Signature on target
   */
  signature?: Array<Signature>;
}

export interface QuestionnaireItemEnableWhen {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Question that determines whether item is enabled
   */
  question: string;
  /** 
   * Question that determines whether item is enabled
   */
  _question?: Element
  /** 
   * exists | = | != | > | < | >= | <=
   */
  operator: code;
  /** 
   * exists | = | != | > | < | >= | <=
   */
  _operator?: Element
  /** 
   * Value for question comparison based on operator
   */
  answerBoolean?: boolean;
  /** 
   * Value for question comparison based on operator
   */
  _answerBoolean?: Element
  /** 
   * Value for question comparison based on operator
   */
  answerDecimal?: decimal;
  /** 
   * Value for question comparison based on operator
   */
  _answerDecimal?: Element
  /** 
   * Value for question comparison based on operator
   */
  answerInteger?: integer;
  /** 
   * Value for question comparison based on operator
   */
  _answerInteger?: Element
  /** 
   * Value for question comparison based on operator
   */
  answerDate?: date;
  /** 
   * Value for question comparison based on operator
   */
  _answerDate?: Element
  /** 
   * Value for question comparison based on operator
   */
  answerDateTime?: dateTime;
  /** 
   * Value for question comparison based on operator
   */
  _answerDateTime?: Element
  /** 
   * Value for question comparison based on operator
   */
  answerTime?: time;
  /** 
   * Value for question comparison based on operator
   */
  _answerTime?: Element
  /** 
   * Value for question comparison based on operator
   */
  answerString?: string;
  /** 
   * Value for question comparison based on operator
   */
  _answerString?: Element
  /** 
   * Value for question comparison based on operator
   */
  answerCoding?: Coding;
  /** 
   * Value for question comparison based on operator
   */
  answerQuantity?: Quantity;
  /** 
   * Value for question comparison based on operator
   */
  answerReference?: Reference;
}
export interface QuestionnaireItemAnswerOption {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Answer value
   */
  valueInteger?: integer;
  /** 
   * Answer value
   */
  _valueInteger?: Element
  /** 
   * Answer value
   */
  valueDate?: date;
  /** 
   * Answer value
   */
  _valueDate?: Element
  /** 
   * Answer value
   */
  valueTime?: time;
  /** 
   * Answer value
   */
  _valueTime?: Element
  /** 
   * Answer value
   */
  valueString?: string;
  /** 
   * Answer value
   */
  _valueString?: Element
  /** 
   * Answer value
   */
  valueCoding?: Coding;
  /** 
   * Answer value
   */
  valueReference?: Reference;
  /** 
   * Whether option is selected by default
   */
  initialSelected?: boolean;
  /** 
   * Whether option is selected by default
   */
  _initialSelected?: Element
}
export interface QuestionnaireItemInitial {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Actual value for initializing the question
   */
  valueBoolean?: boolean;
  /** 
   * Actual value for initializing the question
   */
  _valueBoolean?: Element
  /** 
   * Actual value for initializing the question
   */
  valueDecimal?: decimal;
  /** 
   * Actual value for initializing the question
   */
  _valueDecimal?: Element
  /** 
   * Actual value for initializing the question
   */
  valueInteger?: integer;
  /** 
   * Actual value for initializing the question
   */
  _valueInteger?: Element
  /** 
   * Actual value for initializing the question
   */
  valueDate?: date;
  /** 
   * Actual value for initializing the question
   */
  _valueDate?: Element
  /** 
   * Actual value for initializing the question
   */
  valueDateTime?: dateTime;
  /** 
   * Actual value for initializing the question
   */
  _valueDateTime?: Element
  /** 
   * Actual value for initializing the question
   */
  valueTime?: time;
  /** 
   * Actual value for initializing the question
   */
  _valueTime?: Element
  /** 
   * Actual value for initializing the question
   */
  valueString?: string;
  /** 
   * Actual value for initializing the question
   */
  _valueString?: Element
  /** 
   * Actual value for initializing the question
   */
  valueUri?: uri;
  /** 
   * Actual value for initializing the question
   */
  _valueUri?: Element
  /** 
   * Actual value for initializing the question
   */
  valueAttachment?: Attachment;
  /** 
   * Actual value for initializing the question
   */
  valueCoding?: Coding;
  /** 
   * Actual value for initializing the question
   */
  valueQuantity?: Quantity;
  /** 
   * Actual value for initializing the question
   */
  valueReference?: Reference;
}
export interface QuestionnaireItem {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Unique id for item in questionnaire
   */
  linkId: string;
  /** 
   * Unique id for item in questionnaire
   */
  _linkId?: Element
  /** 
   * ElementDefinition - details for the item
   */
  definition?: uri;
  /** 
   * ElementDefinition - details for the item
   */
  _definition?: Element
  /** 
   * Corresponding concept for this item in a terminology
   */
  code?: Array<Coding>;
  /** 
   * E.g. "1(a)", "2.5.3"
   */
  prefix?: string;
  /** 
   * E.g. "1(a)", "2.5.3"
   */
  _prefix?: Element
  /** 
   * Primary text for the item
   */
  text?: string;
  /** 
   * Primary text for the item
   */
  _text?: Element
  /** 
   * group | display | boolean | decimal | integer | date | dateTime +
   */
  type: code;
  /** 
   * group | display | boolean | decimal | integer | date | dateTime +
   */
  _type?: Element
  /** 
   * Only allow data when
   */
  enableWhen?: Array<QuestionnaireItemEnableWhen>;
  /** 
   * all | any
   */
  enableBehavior?: code;
  /** 
   * all | any
   */
  _enableBehavior?: Element
  /** 
   * Whether the item must be included in data results
   */
  required?: boolean;
  /** 
   * Whether the item must be included in data results
   */
  _required?: Element
  /** 
   * Whether the item may repeat
   */
  repeats?: boolean;
  /** 
   * Whether the item may repeat
   */
  _repeats?: Element
  /** 
   * Don't allow human editing
   */
  readOnly?: boolean;
  /** 
   * Don't allow human editing
   */
  _readOnly?: Element
  /** 
   * No more than this many characters
   */
  maxLength?: integer;
  /** 
   * No more than this many characters
   */
  _maxLength?: Element
  /** 
   * Valueset containing permitted answers
   */
  answerValueSet?: canonical;
  /** 
   * Valueset containing permitted answers
   */
  _answerValueSet?: Element
  /** 
   * Permitted answer
   */
  answerOption?: Array<QuestionnaireItemAnswerOption>;
  /** 
   * Initial value(s) when item is first rendered
   */
  initial?: Array<QuestionnaireItemInitial>;
  /** 
   * Nested questionnaire items
   */
  item?: Array<QuestionnaireItem>;
}
export interface Questionnaire {
resourceType: "Questionnaire"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Canonical identifier for this questionnaire, represented as a URI (globally unique)
   */
  url?: uri;
  /** 
   * Canonical identifier for this questionnaire, represented as a URI (globally unique)
   */
  _url?: Element
  /** 
   * Additional identifier for the questionnaire
   */
  identifier?: Array<Identifier>;
  /** 
   * Business version of the questionnaire
   */
  version?: string;
  /** 
   * Business version of the questionnaire
   */
  _version?: Element
  /** 
   * Name for this questionnaire (computer friendly)
   */
  name?: string;
  /** 
   * Name for this questionnaire (computer friendly)
   */
  _name?: Element
  /** 
   * Name for this questionnaire (human friendly)
   */
  title?: string;
  /** 
   * Name for this questionnaire (human friendly)
   */
  _title?: Element
  /** 
   * Instantiates protocol or definition
   */
  derivedFrom?: Array<canonical>;
  /** 
   * Instantiates protocol or definition
   */
  _derivedFrom?: Array<Element>
  /** 
   * draft | active | retired | unknown
   */
  status: code;
  /** 
   * draft | active | retired | unknown
   */
  _status?: Element
  /** 
   * For testing purposes, not real usage
   */
  experimental?: boolean;
  /** 
   * For testing purposes, not real usage
   */
  _experimental?: Element
  /** 
   * Resource that can be subject of QuestionnaireResponse
   */
  subjectType?: Array<code>;
  /** 
   * Resource that can be subject of QuestionnaireResponse
   */
  _subjectType?: Array<Element>
  /** 
   * Date last changed
   */
  date?: dateTime;
  /** 
   * Date last changed
   */
  _date?: Element
  /** 
   * Name of the publisher (organization or individual)
   */
  publisher?: string;
  /** 
   * Name of the publisher (organization or individual)
   */
  _publisher?: Element
  /** 
   * Contact details for the publisher
   */
  contact?: Array<ContactDetail>;
  /** 
   * Natural language description of the questionnaire
   */
  description?: markdown;
  /** 
   * Natural language description of the questionnaire
   */
  _description?: Element
  /** 
   * The context that the content is intended to support
   */
  useContext?: Array<UsageContext>;
  /** 
   * Intended jurisdiction for questionnaire (if applicable)
   */
  jurisdiction?: Array<CodeableConcept>;
  /** 
   * Why this questionnaire is defined
   */
  purpose?: markdown;
  /** 
   * Why this questionnaire is defined
   */
  _purpose?: Element
  /** 
   * Use and/or publishing restrictions
   */
  copyright?: markdown;
  /** 
   * Use and/or publishing restrictions
   */
  _copyright?: Element
  /** 
   * When the questionnaire was approved by publisher
   */
  approvalDate?: date;
  /** 
   * When the questionnaire was approved by publisher
   */
  _approvalDate?: Element
  /** 
   * When the questionnaire was last reviewed
   */
  lastReviewDate?: date;
  /** 
   * When the questionnaire was last reviewed
   */
  _lastReviewDate?: Element
  /** 
   * When the questionnaire is expected to be used
   */
  effectivePeriod?: Period;
  /** 
   * Concept that represents the overall questionnaire
   */
  code?: Array<Coding>;
  /** 
   * Questions and sections within the Questionnaire
   */
  item?: Array<QuestionnaireItem>;
}

export interface QuestionnaireResponseItemAnswer {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Single-valued answer to the question
   */
  valueBoolean?: boolean;
  /** 
   * Single-valued answer to the question
   */
  _valueBoolean?: Element
  /** 
   * Single-valued answer to the question
   */
  valueDecimal?: decimal;
  /** 
   * Single-valued answer to the question
   */
  _valueDecimal?: Element
  /** 
   * Single-valued answer to the question
   */
  valueInteger?: integer;
  /** 
   * Single-valued answer to the question
   */
  _valueInteger?: Element
  /** 
   * Single-valued answer to the question
   */
  valueDate?: date;
  /** 
   * Single-valued answer to the question
   */
  _valueDate?: Element
  /** 
   * Single-valued answer to the question
   */
  valueDateTime?: dateTime;
  /** 
   * Single-valued answer to the question
   */
  _valueDateTime?: Element
  /** 
   * Single-valued answer to the question
   */
  valueTime?: time;
  /** 
   * Single-valued answer to the question
   */
  _valueTime?: Element
  /** 
   * Single-valued answer to the question
   */
  valueString?: string;
  /** 
   * Single-valued answer to the question
   */
  _valueString?: Element
  /** 
   * Single-valued answer to the question
   */
  valueUri?: uri;
  /** 
   * Single-valued answer to the question
   */
  _valueUri?: Element
  /** 
   * Single-valued answer to the question
   */
  valueAttachment?: Attachment;
  /** 
   * Single-valued answer to the question
   */
  valueCoding?: Coding;
  /** 
   * Single-valued answer to the question
   */
  valueQuantity?: Quantity;
  /** 
   * Single-valued answer to the question
   */
  valueReference?: Reference;
  /** 
   * Nested groups and questions
   */
  item?: Array<QuestionnaireResponseItem>;
}
export interface QuestionnaireResponseItem {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Pointer to specific item from Questionnaire
   */
  linkId: string;
  /** 
   * Pointer to specific item from Questionnaire
   */
  _linkId?: Element
  /** 
   * ElementDefinition - details for the item
   */
  definition?: uri;
  /** 
   * ElementDefinition - details for the item
   */
  _definition?: Element
  /** 
   * Name for group or question text
   */
  text?: string;
  /** 
   * Name for group or question text
   */
  _text?: Element
  /** 
   * The response(s) to the question
   */
  answer?: Array<QuestionnaireResponseItemAnswer>;
  /** 
   * Nested questionnaire response items
   */
  item?: Array<QuestionnaireResponseItem>;
}
export interface QuestionnaireResponse {
resourceType: "QuestionnaireResponse"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Unique id for this set of answers
   */
  identifier?: Identifier;
  /** 
   * Request fulfilled by this QuestionnaireResponse
   */
  basedOn?: Array<Reference>;
  /** 
   * Part of this action
   */
  partOf?: Array<Reference>;
  /** 
   * Form being answered
   */
  questionnaire?: canonical;
  /** 
   * Form being answered
   */
  _questionnaire?: Element
  /** 
   * in-progress | completed | amended | entered-in-error | stopped
   */
  status: code;
  /** 
   * in-progress | completed | amended | entered-in-error | stopped
   */
  _status?: Element
  /** 
   * The subject of the questions
   */
  subject?: Reference;
  /** 
   * Encounter created as part of
   */
  encounter?: Reference;
  /** 
   * Date the answers were gathered
   */
  authored?: dateTime;
  /** 
   * Date the answers were gathered
   */
  _authored?: Element
  /** 
   * Person who received and recorded the answers
   */
  author?: Reference;
  /** 
   * The person who answered the questions
   */
  source?: Reference;
  /** 
   * Groups and questions
   */
  item?: Array<QuestionnaireResponseItem>;
}

export interface RegulatedAuthorizationCase {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Identifier by which this case can be referenced
   */
  identifier?: Identifier;
  /** 
   * The defining type of case
   */
  type?: CodeableConcept;
  /** 
   * The status associated with the case
   */
  status?: CodeableConcept;
  /** 
   * Relevant date for this case
   */
  datePeriod?: Period;
  /** 
   * Relevant date for this case
   */
  dateDateTime?: dateTime;
  /** 
   * Relevant date for this case
   */
  _dateDateTime?: Element
  /** 
   * Applications submitted to obtain a regulated authorization. Steps within the longer running case or procedure
   */
  application?: Array<RegulatedAuthorizationCase>;
}
export interface RegulatedAuthorization {
resourceType: "RegulatedAuthorization"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business identifier for the authorization, typically assigned by the authorizing body
   */
  identifier?: Array<Identifier>;
  /** 
   * The product type, treatment, facility or activity that is being authorized
   */
  subject?: Array<Reference>;
  /** 
   * Overall type of this authorization, for example drug marketing approval, orphan drug designation
   */
  type?: CodeableConcept;
  /** 
   * General textual supporting information
   */
  description?: markdown;
  /** 
   * General textual supporting information
   */
  _description?: Element
  /** 
   * The territory in which the authorization has been granted
   */
  region?: Array<CodeableConcept>;
  /** 
   * The status that is authorised e.g. approved. Intermediate states can be tracked with cases and applications
   */
  status?: CodeableConcept;
  /** 
   * The date at which the current status was assigned
   */
  statusDate?: dateTime;
  /** 
   * The date at which the current status was assigned
   */
  _statusDate?: Element
  /** 
   * The time period in which the regulatory approval etc. is in effect, e.g. a Marketing Authorization includes the date of authorization and/or expiration date
   */
  validityPeriod?: Period;
  /** 
   * Condition for which the use of the regulated product applies
   */
  indication?: CodeableReference;
  /** 
   * The intended use of the product, e.g. prevention, treatment
   */
  intendedUse?: CodeableConcept;
  /** 
   * The legal/regulatory framework or reasons under which this authorization is granted
   */
  basis?: Array<CodeableConcept>;
  /** 
   * The organization that has been granted this authorization, by the regulator
   */
  holder?: Reference;
  /** 
   * The regulatory authority or authorizing body granting the authorization
   */
  regulator?: Reference;
  /** 
   * The case or regulatory procedure for granting or amending a regulated authorization. Note: This area is subject to ongoing review and the workgroup is seeking implementer feedback on its use (see link at bottom of page)
   */
  case?: RegulatedAuthorizationCase;
}

export interface RelatedPersonCommunication {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The language which can be used to communicate with the patient about his or her health
   */
  language: CodeableConcept;
  /** 
   * Language preference indicator
   */
  preferred?: boolean;
  /** 
   * Language preference indicator
   */
  _preferred?: Element
}
export interface RelatedPerson {
resourceType: "RelatedPerson"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * A human identifier for this person
   */
  identifier?: Array<Identifier>;
  /** 
   * Whether this related person's record is in active use
   */
  active?: boolean;
  /** 
   * Whether this related person's record is in active use
   */
  _active?: Element
  /** 
   * The patient this person is related to
   */
  patient: Reference;
  /** 
   * The nature of the relationship
   */
  relationship?: Array<CodeableConcept>;
  /** 
   * A name associated with the person
   */
  name?: Array<HumanName>;
  /** 
   * A contact detail for the person
   */
  telecom?: Array<ContactPoint>;
  /** 
   * male | female | other | unknown
   */
  gender?: code;
  /** 
   * male | female | other | unknown
   */
  _gender?: Element
  /** 
   * The date on which the related person was born
   */
  birthDate?: date;
  /** 
   * The date on which the related person was born
   */
  _birthDate?: Element
  /** 
   * Address where the related person can be contacted or visited
   */
  address?: Array<Address>;
  /** 
   * Image of the person
   */
  photo?: Array<Attachment>;
  /** 
   * Period of time that this relationship is considered valid
   */
  period?: Period;
  /** 
   * A language which may be used to communicate with about the patient's health
   */
  communication?: Array<RelatedPersonCommunication>;
}

export interface RequestGroupActionCondition {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * applicability | start | stop
   */
  kind: code;
  /** 
   * applicability | start | stop
   */
  _kind?: Element
  /** 
   * Boolean-valued expression
   */
  expression?: Expression;
}
export interface RequestGroupActionRelatedAction {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * What action this is related to
   */
  actionId: id;
  /** 
   * What action this is related to
   */
  _actionId?: Element
  /** 
   * before-start | before | before-end | concurrent-with-start | concurrent | concurrent-with-end | after-start | after | after-end
   */
  relationship: code;
  /** 
   * before-start | before | before-end | concurrent-with-start | concurrent | concurrent-with-end | after-start | after | after-end
   */
  _relationship?: Element
  /** 
   * Time offset for the relationship
   */
  offsetDuration?: Duration;
  /** 
   * Time offset for the relationship
   */
  offsetRange?: Range;
}
export interface RequestGroupAction {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * User-visible prefix for the action (e.g. 1. or A.)
   */
  prefix?: string;
  /** 
   * User-visible prefix for the action (e.g. 1. or A.)
   */
  _prefix?: Element
  /** 
   * User-visible title
   */
  title?: string;
  /** 
   * User-visible title
   */
  _title?: Element
  /** 
   * Short description of the action
   */
  description?: string;
  /** 
   * Short description of the action
   */
  _description?: Element
  /** 
   * Static text equivalent of the action, used if the dynamic aspects cannot be interpreted by the receiving system
   */
  textEquivalent?: string;
  /** 
   * Static text equivalent of the action, used if the dynamic aspects cannot be interpreted by the receiving system
   */
  _textEquivalent?: Element
  /** 
   * routine | urgent | asap | stat
   */
  priority?: code;
  /** 
   * routine | urgent | asap | stat
   */
  _priority?: Element
  /** 
   * Code representing the meaning of the action or sub-actions
   */
  code?: Array<CodeableConcept>;
  /** 
   * Supporting documentation for the intended performer of the action
   */
  documentation?: Array<RelatedArtifact>;
  /** 
   * Whether or not the action is applicable
   */
  condition?: Array<RequestGroupActionCondition>;
  /** 
   * Relationship to another action
   */
  relatedAction?: Array<RequestGroupActionRelatedAction>;
  /** 
   * When the action should take place
   */
  timingDateTime?: dateTime;
  /** 
   * When the action should take place
   */
  _timingDateTime?: Element
  /** 
   * When the action should take place
   */
  timingAge?: Age;
  /** 
   * When the action should take place
   */
  timingPeriod?: Period;
  /** 
   * When the action should take place
   */
  timingDuration?: Duration;
  /** 
   * When the action should take place
   */
  timingRange?: Range;
  /** 
   * When the action should take place
   */
  timingTiming?: Timing;
  /** 
   * Who should perform the action
   */
  participant?: Array<Reference>;
  /** 
   * create | update | remove | fire-event
   */
  type?: CodeableConcept;
  /** 
   * visual-group | logical-group | sentence-group
   */
  groupingBehavior?: code;
  /** 
   * visual-group | logical-group | sentence-group
   */
  _groupingBehavior?: Element
  /** 
   * any | all | all-or-none | exactly-one | at-most-one | one-or-more
   */
  selectionBehavior?: code;
  /** 
   * any | all | all-or-none | exactly-one | at-most-one | one-or-more
   */
  _selectionBehavior?: Element
  /** 
   * must | could | must-unless-documented
   */
  requiredBehavior?: code;
  /** 
   * must | could | must-unless-documented
   */
  _requiredBehavior?: Element
  /** 
   * yes | no
   */
  precheckBehavior?: code;
  /** 
   * yes | no
   */
  _precheckBehavior?: Element
  /** 
   * single | multiple
   */
  cardinalityBehavior?: code;
  /** 
   * single | multiple
   */
  _cardinalityBehavior?: Element
  /** 
   * The target of the action
   */
  resource?: Reference;
  /** 
   * Sub action
   */
  action?: Array<RequestGroupAction>;
}
export interface RequestGroup {
resourceType: "RequestGroup"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business identifier
   */
  identifier?: Array<Identifier>;
  /** 
   * Instantiates FHIR protocol or definition
   */
  instantiatesCanonical?: Array<canonical>;
  /** 
   * Instantiates FHIR protocol or definition
   */
  _instantiatesCanonical?: Array<Element>
  /** 
   * Instantiates external protocol or definition
   */
  instantiatesUri?: Array<uri>;
  /** 
   * Instantiates external protocol or definition
   */
  _instantiatesUri?: Array<Element>
  /** 
   * Fulfills plan, proposal, or order
   */
  basedOn?: Array<Reference>;
  /** 
   * Request(s) replaced by this request
   */
  replaces?: Array<Reference>;
  /** 
   * Composite request this is part of
   */
  groupIdentifier?: Identifier;
  /** 
   * draft | active | on-hold | revoked | completed | entered-in-error | unknown
   */
  status: code;
  /** 
   * draft | active | on-hold | revoked | completed | entered-in-error | unknown
   */
  _status?: Element
  /** 
   * proposal | plan | directive | order | original-order | reflex-order | filler-order | instance-order | option
   */
  intent: code;
  /** 
   * proposal | plan | directive | order | original-order | reflex-order | filler-order | instance-order | option
   */
  _intent?: Element
  /** 
   * routine | urgent | asap | stat
   */
  priority?: code;
  /** 
   * routine | urgent | asap | stat
   */
  _priority?: Element
  /** 
   * What's being requested/ordered
   */
  code?: CodeableConcept;
  /** 
   * Who the request group is about
   */
  subject?: Reference;
  /** 
   * Created as part of
   */
  encounter?: Reference;
  /** 
   * When the request group was authored
   */
  authoredOn?: dateTime;
  /** 
   * When the request group was authored
   */
  _authoredOn?: Element
  /** 
   * Device or practitioner that authored the request group
   */
  author?: Reference;
  /** 
   * Why the request group is needed
   */
  reasonCode?: Array<CodeableConcept>;
  /** 
   * Why the request group is needed
   */
  reasonReference?: Array<Reference>;
  /** 
   * Additional notes about the response
   */
  note?: Array<Annotation>;
  /** 
   * Proposed actions, if any
   */
  action?: Array<RequestGroupAction>;
}

export interface ResearchDefinition {
resourceType: "ResearchDefinition"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Canonical identifier for this research definition, represented as a URI (globally unique)
   */
  url?: uri;
  /** 
   * Canonical identifier for this research definition, represented as a URI (globally unique)
   */
  _url?: Element
  /** 
   * Additional identifier for the research definition
   */
  identifier?: Array<Identifier>;
  /** 
   * Business version of the research definition
   */
  version?: string;
  /** 
   * Business version of the research definition
   */
  _version?: Element
  /** 
   * Name for this research definition (computer friendly)
   */
  name?: string;
  /** 
   * Name for this research definition (computer friendly)
   */
  _name?: Element
  /** 
   * Name for this research definition (human friendly)
   */
  title?: string;
  /** 
   * Name for this research definition (human friendly)
   */
  _title?: Element
  /** 
   * Title for use in informal contexts
   */
  shortTitle?: string;
  /** 
   * Title for use in informal contexts
   */
  _shortTitle?: Element
  /** 
   * Subordinate title of the ResearchDefinition
   */
  subtitle?: string;
  /** 
   * Subordinate title of the ResearchDefinition
   */
  _subtitle?: Element
  /** 
   * draft | active | retired | unknown
   */
  status: code;
  /** 
   * draft | active | retired | unknown
   */
  _status?: Element
  /** 
   * For testing purposes, not real usage
   */
  experimental?: boolean;
  /** 
   * For testing purposes, not real usage
   */
  _experimental?: Element
  /** 
   * E.g. Patient, Practitioner, RelatedPerson, Organization, Location, Device
   */
  subjectCodeableConcept?: CodeableConcept;
  /** 
   * E.g. Patient, Practitioner, RelatedPerson, Organization, Location, Device
   */
  subjectReference?: Reference;
  /** 
   * Date last changed
   */
  date?: dateTime;
  /** 
   * Date last changed
   */
  _date?: Element
  /** 
   * Name of the publisher (organization or individual)
   */
  publisher?: string;
  /** 
   * Name of the publisher (organization or individual)
   */
  _publisher?: Element
  /** 
   * Contact details for the publisher
   */
  contact?: Array<ContactDetail>;
  /** 
   * Natural language description of the research definition
   */
  description?: markdown;
  /** 
   * Natural language description of the research definition
   */
  _description?: Element
  /** 
   * Used for footnotes or explanatory notes
   */
  comment?: Array<string>;
  /** 
   * Used for footnotes or explanatory notes
   */
  _comment?: Array<Element>
  /** 
   * The context that the content is intended to support
   */
  useContext?: Array<UsageContext>;
  /** 
   * Intended jurisdiction for research definition (if applicable)
   */
  jurisdiction?: Array<CodeableConcept>;
  /** 
   * Why this research definition is defined
   */
  purpose?: markdown;
  /** 
   * Why this research definition is defined
   */
  _purpose?: Element
  /** 
   * Describes the clinical usage of the ResearchDefinition
   */
  usage?: string;
  /** 
   * Describes the clinical usage of the ResearchDefinition
   */
  _usage?: Element
  /** 
   * Use and/or publishing restrictions
   */
  copyright?: markdown;
  /** 
   * Use and/or publishing restrictions
   */
  _copyright?: Element
  /** 
   * When the research definition was approved by publisher
   */
  approvalDate?: date;
  /** 
   * When the research definition was approved by publisher
   */
  _approvalDate?: Element
  /** 
   * When the research definition was last reviewed
   */
  lastReviewDate?: date;
  /** 
   * When the research definition was last reviewed
   */
  _lastReviewDate?: Element
  /** 
   * When the research definition is expected to be used
   */
  effectivePeriod?: Period;
  /** 
   * The category of the ResearchDefinition, such as Education, Treatment, Assessment, etc.
   */
  topic?: Array<CodeableConcept>;
  /** 
   * Who authored the content
   */
  author?: Array<ContactDetail>;
  /** 
   * Who edited the content
   */
  editor?: Array<ContactDetail>;
  /** 
   * Who reviewed the content
   */
  reviewer?: Array<ContactDetail>;
  /** 
   * Who endorsed the content
   */
  endorser?: Array<ContactDetail>;
  /** 
   * Additional documentation, citations, etc.
   */
  relatedArtifact?: Array<RelatedArtifact>;
  /** 
   * Logic used by the ResearchDefinition
   */
  library?: Array<canonical>;
  /** 
   * Logic used by the ResearchDefinition
   */
  _library?: Array<Element>
  /** 
   * What population?
   */
  population: Reference;
  /** 
   * What exposure?
   */
  exposure?: Reference;
  /** 
   * What alternative exposure state?
   */
  exposureAlternative?: Reference;
  /** 
   * What outcome?
   */
  outcome?: Reference;
}

export interface ResearchElementDefinitionCharacteristic {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * What code or expression defines members?
   */
  definitionCodeableConcept?: CodeableConcept;
  /** 
   * What code or expression defines members?
   */
  definitionCanonical?: canonical;
  /** 
   * What code or expression defines members?
   */
  _definitionCanonical?: Element
  /** 
   * What code or expression defines members?
   */
  definitionExpression?: Expression;
  /** 
   * What code or expression defines members?
   */
  definitionDataRequirement?: DataRequirement;
  /** 
   * What code/value pairs define members?
   */
  usageContext?: Array<UsageContext>;
  /** 
   * Whether the characteristic includes or excludes members
   */
  exclude?: boolean;
  /** 
   * Whether the characteristic includes or excludes members
   */
  _exclude?: Element
  /** 
   * What unit is the outcome described in?
   */
  unitOfMeasure?: CodeableConcept;
  /** 
   * What time period does the study cover
   */
  studyEffectiveDescription?: string;
  /** 
   * What time period does the study cover
   */
  _studyEffectiveDescription?: Element
  /** 
   * What time period does the study cover
   */
  studyEffectiveDateTime?: dateTime;
  /** 
   * What time period does the study cover
   */
  _studyEffectiveDateTime?: Element
  /** 
   * What time period does the study cover
   */
  studyEffectivePeriod?: Period;
  /** 
   * What time period does the study cover
   */
  studyEffectiveDuration?: Duration;
  /** 
   * What time period does the study cover
   */
  studyEffectiveTiming?: Timing;
  /** 
   * Observation time from study start
   */
  studyEffectiveTimeFromStart?: Duration;
  /** 
   * mean | median | mean-of-mean | mean-of-median | median-of-mean | median-of-median
   */
  studyEffectiveGroupMeasure?: code;
  /** 
   * mean | median | mean-of-mean | mean-of-median | median-of-mean | median-of-median
   */
  _studyEffectiveGroupMeasure?: Element
  /** 
   * What time period do participants cover
   */
  participantEffectiveDescription?: string;
  /** 
   * What time period do participants cover
   */
  _participantEffectiveDescription?: Element
  /** 
   * What time period do participants cover
   */
  participantEffectiveDateTime?: dateTime;
  /** 
   * What time period do participants cover
   */
  _participantEffectiveDateTime?: Element
  /** 
   * What time period do participants cover
   */
  participantEffectivePeriod?: Period;
  /** 
   * What time period do participants cover
   */
  participantEffectiveDuration?: Duration;
  /** 
   * What time period do participants cover
   */
  participantEffectiveTiming?: Timing;
  /** 
   * Observation time from study start
   */
  participantEffectiveTimeFromStart?: Duration;
  /** 
   * mean | median | mean-of-mean | mean-of-median | median-of-mean | median-of-median
   */
  participantEffectiveGroupMeasure?: code;
  /** 
   * mean | median | mean-of-mean | mean-of-median | median-of-mean | median-of-median
   */
  _participantEffectiveGroupMeasure?: Element
}
export interface ResearchElementDefinition {
resourceType: "ResearchElementDefinition"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Canonical identifier for this research element definition, represented as a URI (globally unique)
   */
  url?: uri;
  /** 
   * Canonical identifier for this research element definition, represented as a URI (globally unique)
   */
  _url?: Element
  /** 
   * Additional identifier for the research element definition
   */
  identifier?: Array<Identifier>;
  /** 
   * Business version of the research element definition
   */
  version?: string;
  /** 
   * Business version of the research element definition
   */
  _version?: Element
  /** 
   * Name for this research element definition (computer friendly)
   */
  name?: string;
  /** 
   * Name for this research element definition (computer friendly)
   */
  _name?: Element
  /** 
   * Name for this research element definition (human friendly)
   */
  title?: string;
  /** 
   * Name for this research element definition (human friendly)
   */
  _title?: Element
  /** 
   * Title for use in informal contexts
   */
  shortTitle?: string;
  /** 
   * Title for use in informal contexts
   */
  _shortTitle?: Element
  /** 
   * Subordinate title of the ResearchElementDefinition
   */
  subtitle?: string;
  /** 
   * Subordinate title of the ResearchElementDefinition
   */
  _subtitle?: Element
  /** 
   * draft | active | retired | unknown
   */
  status: code;
  /** 
   * draft | active | retired | unknown
   */
  _status?: Element
  /** 
   * For testing purposes, not real usage
   */
  experimental?: boolean;
  /** 
   * For testing purposes, not real usage
   */
  _experimental?: Element
  /** 
   * E.g. Patient, Practitioner, RelatedPerson, Organization, Location, Device
   */
  subjectCodeableConcept?: CodeableConcept;
  /** 
   * E.g. Patient, Practitioner, RelatedPerson, Organization, Location, Device
   */
  subjectReference?: Reference;
  /** 
   * Date last changed
   */
  date?: dateTime;
  /** 
   * Date last changed
   */
  _date?: Element
  /** 
   * Name of the publisher (organization or individual)
   */
  publisher?: string;
  /** 
   * Name of the publisher (organization or individual)
   */
  _publisher?: Element
  /** 
   * Contact details for the publisher
   */
  contact?: Array<ContactDetail>;
  /** 
   * Natural language description of the research element definition
   */
  description?: markdown;
  /** 
   * Natural language description of the research element definition
   */
  _description?: Element
  /** 
   * Used for footnotes or explanatory notes
   */
  comment?: Array<string>;
  /** 
   * Used for footnotes or explanatory notes
   */
  _comment?: Array<Element>
  /** 
   * The context that the content is intended to support
   */
  useContext?: Array<UsageContext>;
  /** 
   * Intended jurisdiction for research element definition (if applicable)
   */
  jurisdiction?: Array<CodeableConcept>;
  /** 
   * Why this research element definition is defined
   */
  purpose?: markdown;
  /** 
   * Why this research element definition is defined
   */
  _purpose?: Element
  /** 
   * Describes the clinical usage of the ResearchElementDefinition
   */
  usage?: string;
  /** 
   * Describes the clinical usage of the ResearchElementDefinition
   */
  _usage?: Element
  /** 
   * Use and/or publishing restrictions
   */
  copyright?: markdown;
  /** 
   * Use and/or publishing restrictions
   */
  _copyright?: Element
  /** 
   * When the research element definition was approved by publisher
   */
  approvalDate?: date;
  /** 
   * When the research element definition was approved by publisher
   */
  _approvalDate?: Element
  /** 
   * When the research element definition was last reviewed
   */
  lastReviewDate?: date;
  /** 
   * When the research element definition was last reviewed
   */
  _lastReviewDate?: Element
  /** 
   * When the research element definition is expected to be used
   */
  effectivePeriod?: Period;
  /** 
   * The category of the ResearchElementDefinition, such as Education, Treatment, Assessment, etc.
   */
  topic?: Array<CodeableConcept>;
  /** 
   * Who authored the content
   */
  author?: Array<ContactDetail>;
  /** 
   * Who edited the content
   */
  editor?: Array<ContactDetail>;
  /** 
   * Who reviewed the content
   */
  reviewer?: Array<ContactDetail>;
  /** 
   * Who endorsed the content
   */
  endorser?: Array<ContactDetail>;
  /** 
   * Additional documentation, citations, etc.
   */
  relatedArtifact?: Array<RelatedArtifact>;
  /** 
   * Logic used by the ResearchElementDefinition
   */
  library?: Array<canonical>;
  /** 
   * Logic used by the ResearchElementDefinition
   */
  _library?: Array<Element>
  /** 
   * population | exposure | outcome
   */
  type: code;
  /** 
   * population | exposure | outcome
   */
  _type?: Element
  /** 
   * dichotomous | continuous | descriptive
   */
  variableType?: code;
  /** 
   * dichotomous | continuous | descriptive
   */
  _variableType?: Element
  /** 
   * What defines the members of the research element
   */
  characteristic: Array<ResearchElementDefinitionCharacteristic>;
}

export interface ResearchStudyArm {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Label for study arm
   */
  name: string;
  /** 
   * Label for study arm
   */
  _name?: Element
  /** 
   * Categorization of study arm
   */
  type?: CodeableConcept;
  /** 
   * Short explanation of study path
   */
  description?: string;
  /** 
   * Short explanation of study path
   */
  _description?: Element
}
export interface ResearchStudyObjective {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Label for the objective
   */
  name?: string;
  /** 
   * Label for the objective
   */
  _name?: Element
  /** 
   * primary | secondary | exploratory
   */
  type?: CodeableConcept;
}
export interface ResearchStudy {
resourceType: "ResearchStudy"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business Identifier for study
   */
  identifier?: Array<Identifier>;
  /** 
   * Name for this study
   */
  title?: string;
  /** 
   * Name for this study
   */
  _title?: Element
  /** 
   * Steps followed in executing study
   */
  protocol?: Array<Reference>;
  /** 
   * Part of larger study
   */
  partOf?: Array<Reference>;
  /** 
   * active | administratively-completed | approved | closed-to-accrual | closed-to-accrual-and-intervention | completed | disapproved | in-review | temporarily-closed-to-accrual | temporarily-closed-to-accrual-and-intervention | withdrawn
   */
  status: code;
  /** 
   * active | administratively-completed | approved | closed-to-accrual | closed-to-accrual-and-intervention | completed | disapproved | in-review | temporarily-closed-to-accrual | temporarily-closed-to-accrual-and-intervention | withdrawn
   */
  _status?: Element
  /** 
   * treatment | prevention | diagnostic | supportive-care | screening | health-services-research | basic-science | device-feasibility
   */
  primaryPurposeType?: CodeableConcept;
  /** 
   * n-a | early-phase-1 | phase-1 | phase-1-phase-2 | phase-2 | phase-2-phase-3 | phase-3 | phase-4
   */
  phase?: CodeableConcept;
  /** 
   * Classifications for the study
   */
  category?: Array<CodeableConcept>;
  /** 
   * Drugs, devices, etc. under study
   */
  focus?: Array<CodeableConcept>;
  /** 
   * Condition being studied
   */
  condition?: Array<CodeableConcept>;
  /** 
   * Contact details for the study
   */
  contact?: Array<ContactDetail>;
  /** 
   * References and dependencies
   */
  relatedArtifact?: Array<RelatedArtifact>;
  /** 
   * Used to search for the study
   */
  keyword?: Array<CodeableConcept>;
  /** 
   * Geographic region(s) for study
   */
  location?: Array<CodeableConcept>;
  /** 
   * What this is study doing
   */
  description?: markdown;
  /** 
   * What this is study doing
   */
  _description?: Element
  /** 
   * Inclusion & exclusion criteria
   */
  enrollment?: Array<Reference>;
  /** 
   * When the study began and ended
   */
  period?: Period;
  /** 
   * Organization that initiates and is legally responsible for the study
   */
  sponsor?: Reference;
  /** 
   * Researcher who oversees multiple aspects of the study
   */
  principalInvestigator?: Reference;
  /** 
   * Facility where study activities are conducted
   */
  site?: Array<Reference>;
  /** 
   * accrual-goal-met | closed-due-to-toxicity | closed-due-to-lack-of-study-progress | temporarily-closed-per-study-design
   */
  reasonStopped?: CodeableConcept;
  /** 
   * Comments made about the study
   */
  note?: Array<Annotation>;
  /** 
   * Defined path through the study for a subject
   */
  arm?: Array<ResearchStudyArm>;
  /** 
   * A goal for the study
   */
  objective?: Array<ResearchStudyObjective>;
}

export interface ResearchSubject {
resourceType: "ResearchSubject"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business Identifier for research subject in a study
   */
  identifier?: Array<Identifier>;
  /** 
   * candidate | eligible | follow-up | ineligible | not-registered | off-study | on-study | on-study-intervention | on-study-observation | pending-on-study | potential-candidate | screening | withdrawn
   */
  status: code;
  /** 
   * candidate | eligible | follow-up | ineligible | not-registered | off-study | on-study | on-study-intervention | on-study-observation | pending-on-study | potential-candidate | screening | withdrawn
   */
  _status?: Element
  /** 
   * Start and end of participation
   */
  period?: Period;
  /** 
   * Study subject is part of
   */
  study: Reference;
  /** 
   * Who is part of study
   */
  individual: Reference;
  /** 
   * What path should be followed
   */
  assignedArm?: string;
  /** 
   * What path should be followed
   */
  _assignedArm?: Element
  /** 
   * What path was followed
   */
  actualArm?: string;
  /** 
   * What path was followed
   */
  _actualArm?: Element
  /** 
   * Agreement to participate in study
   */
  consent?: Reference;
}

export interface RiskAssessmentPrediction {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Possible outcome for the subject
   */
  outcome?: CodeableConcept;
  /** 
   * Likelihood of specified outcome
   */
  probabilityDecimal?: decimal;
  /** 
   * Likelihood of specified outcome
   */
  _probabilityDecimal?: Element
  /** 
   * Likelihood of specified outcome
   */
  probabilityRange?: Range;
  /** 
   * Likelihood of specified outcome as a qualitative value
   */
  qualitativeRisk?: CodeableConcept;
  /** 
   * Relative likelihood
   */
  relativeRisk?: decimal;
  /** 
   * Relative likelihood
   */
  _relativeRisk?: Element
  /** 
   * Timeframe or age range
   */
  whenPeriod?: Period;
  /** 
   * Timeframe or age range
   */
  whenRange?: Range;
  /** 
   * Explanation of prediction
   */
  rationale?: string;
  /** 
   * Explanation of prediction
   */
  _rationale?: Element
}
export interface RiskAssessment {
resourceType: "RiskAssessment"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Unique identifier for the assessment
   */
  identifier?: Array<Identifier>;
  /** 
   * Request fulfilled by this assessment
   */
  basedOn?: Reference;
  /** 
   * Part of this occurrence
   */
  parent?: Reference;
  /** 
   * registered | preliminary | final | amended +
   */
  status: code;
  /** 
   * registered | preliminary | final | amended +
   */
  _status?: Element
  /** 
   * Evaluation mechanism
   */
  method?: CodeableConcept;
  /** 
   * Type of assessment
   */
  code?: CodeableConcept;
  /** 
   * Who/what does assessment apply to?
   */
  subject: Reference;
  /** 
   * Where was assessment performed?
   */
  encounter?: Reference;
  /** 
   * When was assessment made?
   */
  occurrenceDateTime?: dateTime;
  /** 
   * When was assessment made?
   */
  _occurrenceDateTime?: Element
  /** 
   * When was assessment made?
   */
  occurrencePeriod?: Period;
  /** 
   * Condition assessed
   */
  condition?: Reference;
  /** 
   * Who did assessment?
   */
  performer?: Reference;
  /** 
   * Why the assessment was necessary?
   */
  reasonCode?: Array<CodeableConcept>;
  /** 
   * Why the assessment was necessary?
   */
  reasonReference?: Array<Reference>;
  /** 
   * Information used in assessment
   */
  basis?: Array<Reference>;
  /** 
   * Outcome predicted
   */
  prediction?: Array<RiskAssessmentPrediction>;
  /** 
   * How to reduce risk
   */
  mitigation?: string;
  /** 
   * How to reduce risk
   */
  _mitigation?: Element
  /** 
   * Comments on the risk assessment
   */
  note?: Array<Annotation>;
}

export interface Schedule {
resourceType: "Schedule"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * External Ids for this item
   */
  identifier?: Array<Identifier>;
  /** 
   * Whether this schedule is in active use
   */
  active?: boolean;
  /** 
   * Whether this schedule is in active use
   */
  _active?: Element
  /** 
   * High-level category
   */
  serviceCategory?: Array<CodeableConcept>;
  /** 
   * Specific service
   */
  serviceType?: Array<CodeableConcept>;
  /** 
   * Type of specialty needed
   */
  specialty?: Array<CodeableConcept>;
  /** 
   * Resource(s) that availability information is being provided for
   */
  actor: Array<Reference>;
  /** 
   * Period of time covered by schedule
   */
  planningHorizon?: Period;
  /** 
   * Comments on availability
   */
  comment?: string;
  /** 
   * Comments on availability
   */
  _comment?: Element
}

export interface SearchParameterComponent {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Defines how the part works
   */
  definition: canonical;
  /** 
   * Defines how the part works
   */
  _definition?: Element
  /** 
   * Subexpression relative to main expression
   */
  expression: string;
  /** 
   * Subexpression relative to main expression
   */
  _expression?: Element
}
export interface SearchParameter {
resourceType: "SearchParameter"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Canonical identifier for this search parameter, represented as a URI (globally unique)
   */
  url: uri;
  /** 
   * Canonical identifier for this search parameter, represented as a URI (globally unique)
   */
  _url?: Element
  /** 
   * Business version of the search parameter
   */
  version?: string;
  /** 
   * Business version of the search parameter
   */
  _version?: Element
  /** 
   * Name for this search parameter (computer friendly)
   */
  name: string;
  /** 
   * Name for this search parameter (computer friendly)
   */
  _name?: Element
  /** 
   * Original definition for the search parameter
   */
  derivedFrom?: canonical;
  /** 
   * Original definition for the search parameter
   */
  _derivedFrom?: Element
  /** 
   * draft | active | retired | unknown
   */
  status: code;
  /** 
   * draft | active | retired | unknown
   */
  _status?: Element
  /** 
   * For testing purposes, not real usage
   */
  experimental?: boolean;
  /** 
   * For testing purposes, not real usage
   */
  _experimental?: Element
  /** 
   * Date last changed
   */
  date?: dateTime;
  /** 
   * Date last changed
   */
  _date?: Element
  /** 
   * Name of the publisher (organization or individual)
   */
  publisher?: string;
  /** 
   * Name of the publisher (organization or individual)
   */
  _publisher?: Element
  /** 
   * Contact details for the publisher
   */
  contact?: Array<ContactDetail>;
  /** 
   * Natural language description of the search parameter
   */
  description: markdown;
  /** 
   * Natural language description of the search parameter
   */
  _description?: Element
  /** 
   * The context that the content is intended to support
   */
  useContext?: Array<UsageContext>;
  /** 
   * Intended jurisdiction for search parameter (if applicable)
   */
  jurisdiction?: Array<CodeableConcept>;
  /** 
   * Why this search parameter is defined
   */
  purpose?: markdown;
  /** 
   * Why this search parameter is defined
   */
  _purpose?: Element
  /** 
   * Code used in URL
   */
  code: code;
  /** 
   * Code used in URL
   */
  _code?: Element
  /** 
   * The resource type(s) this search parameter applies to
   */
  base: Array<code>;
  /** 
   * The resource type(s) this search parameter applies to
   */
  _base?: Array<Element>
  /** 
   * number | date | string | token | reference | composite | quantity | uri | special
   */
  type: code;
  /** 
   * number | date | string | token | reference | composite | quantity | uri | special
   */
  _type?: Element
  /** 
   * FHIRPath expression that extracts the values
   */
  expression?: string;
  /** 
   * FHIRPath expression that extracts the values
   */
  _expression?: Element
  /** 
   * XPath that extracts the values
   */
  xpath?: string;
  /** 
   * XPath that extracts the values
   */
  _xpath?: Element
  /** 
   * normal | phonetic | nearby | distance | other
   */
  xpathUsage?: code;
  /** 
   * normal | phonetic | nearby | distance | other
   */
  _xpathUsage?: Element
  /** 
   * Types of resource (if a resource reference)
   */
  target?: Array<code>;
  /** 
   * Types of resource (if a resource reference)
   */
  _target?: Array<Element>
  /** 
   * Allow multiple values per parameter (or)
   */
  multipleOr?: boolean;
  /** 
   * Allow multiple values per parameter (or)
   */
  _multipleOr?: Element
  /** 
   * Allow multiple parameters (and)
   */
  multipleAnd?: boolean;
  /** 
   * Allow multiple parameters (and)
   */
  _multipleAnd?: Element
  /** 
   * eq | ne | gt | lt | ge | le | sa | eb | ap
   */
  comparator?: Array<code>;
  /** 
   * eq | ne | gt | lt | ge | le | sa | eb | ap
   */
  _comparator?: Array<Element>
  /** 
   * missing | exact | contains | not | text | in | not-in | below | above | type | identifier | ofType
   */
  modifier?: Array<code>;
  /** 
   * missing | exact | contains | not | text | in | not-in | below | above | type | identifier | ofType
   */
  _modifier?: Array<Element>
  /** 
   * Chained names supported
   */
  chain?: Array<string>;
  /** 
   * Chained names supported
   */
  _chain?: Array<Element>
  /** 
   * For Composite resources to define the parts
   */
  component?: Array<SearchParameterComponent>;
}

export interface ServiceRequest {
resourceType: "ServiceRequest"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Identifiers assigned to this order
   */
  identifier?: Array<Identifier>;
  /** 
   * Instantiates FHIR protocol or definition
   */
  instantiatesCanonical?: Array<canonical>;
  /** 
   * Instantiates FHIR protocol or definition
   */
  _instantiatesCanonical?: Array<Element>
  /** 
   * Instantiates external protocol or definition
   */
  instantiatesUri?: Array<uri>;
  /** 
   * Instantiates external protocol or definition
   */
  _instantiatesUri?: Array<Element>
  /** 
   * What request fulfills
   */
  basedOn?: Array<Reference>;
  /** 
   * What request replaces
   */
  replaces?: Array<Reference>;
  /** 
   * Composite Request ID
   */
  requisition?: Identifier;
  /** 
   * draft | active | on-hold | revoked | completed | entered-in-error | unknown
   */
  status: code;
  /** 
   * draft | active | on-hold | revoked | completed | entered-in-error | unknown
   */
  _status?: Element
  /** 
   * proposal | plan | directive | order | original-order | reflex-order | filler-order | instance-order | option
   */
  intent: code;
  /** 
   * proposal | plan | directive | order | original-order | reflex-order | filler-order | instance-order | option
   */
  _intent?: Element
  /** 
   * Classification of service
   */
  category?: Array<CodeableConcept>;
  /** 
   * routine | urgent | asap | stat
   */
  priority?: code;
  /** 
   * routine | urgent | asap | stat
   */
  _priority?: Element
  /** 
   * True if service/procedure should not be performed
   */
  doNotPerform?: boolean;
  /** 
   * True if service/procedure should not be performed
   */
  _doNotPerform?: Element
  /** 
   * What is being requested/ordered
   */
  code?: CodeableConcept;
  /** 
   * Additional order information
   */
  orderDetail?: Array<CodeableConcept>;
  /** 
   * Service amount
   */
  quantityQuantity?: Quantity;
  /** 
   * Service amount
   */
  quantityRatio?: Ratio;
  /** 
   * Service amount
   */
  quantityRange?: Range;
  /** 
   * Individual or Entity the service is ordered for
   */
  subject: Reference;
  /** 
   * Encounter in which the request was created
   */
  encounter?: Reference;
  /** 
   * When service should occur
   */
  occurrenceDateTime?: dateTime;
  /** 
   * When service should occur
   */
  _occurrenceDateTime?: Element
  /** 
   * When service should occur
   */
  occurrencePeriod?: Period;
  /** 
   * When service should occur
   */
  occurrenceTiming?: Timing;
  /** 
   * Preconditions for service
   */
  asNeededBoolean?: boolean;
  /** 
   * Preconditions for service
   */
  _asNeededBoolean?: Element
  /** 
   * Preconditions for service
   */
  asNeededCodeableConcept?: CodeableConcept;
  /** 
   * Date request signed
   */
  authoredOn?: dateTime;
  /** 
   * Date request signed
   */
  _authoredOn?: Element
  /** 
   * Who/what is requesting service
   */
  requester?: Reference;
  /** 
   * Performer role
   */
  performerType?: CodeableConcept;
  /** 
   * Requested performer
   */
  performer?: Array<Reference>;
  /** 
   * Requested location
   */
  locationCode?: Array<CodeableConcept>;
  /** 
   * Requested location
   */
  locationReference?: Array<Reference>;
  /** 
   * Explanation/Justification for procedure or service
   */
  reasonCode?: Array<CodeableConcept>;
  /** 
   * Explanation/Justification for service or service
   */
  reasonReference?: Array<Reference>;
  /** 
   * Associated insurance coverage
   */
  insurance?: Array<Reference>;
  /** 
   * Additional clinical information
   */
  supportingInfo?: Array<Reference>;
  /** 
   * Procedure Samples
   */
  specimen?: Array<Reference>;
  /** 
   * Location on Body
   */
  bodySite?: Array<CodeableConcept>;
  /** 
   * Comments
   */
  note?: Array<Annotation>;
  /** 
   * Patient or consumer-oriented instructions
   */
  patientInstruction?: string;
  /** 
   * Patient or consumer-oriented instructions
   */
  _patientInstruction?: Element
  /** 
   * Request provenance
   */
  relevantHistory?: Array<Reference>;
}

export interface Slot {
resourceType: "Slot"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * External Ids for this item
   */
  identifier?: Array<Identifier>;
  /** 
   * A broad categorization of the service that is to be performed during this appointment
   */
  serviceCategory?: Array<CodeableConcept>;
  /** 
   * The type of appointments that can be booked into this slot (ideally this would be an identifiable service - which is at a location, rather than the location itself). If provided then this overrides the value provided on the availability resource
   */
  serviceType?: Array<CodeableConcept>;
  /** 
   * The specialty of a practitioner that would be required to perform the service requested in this appointment
   */
  specialty?: Array<CodeableConcept>;
  /** 
   * The style of appointment or patient that may be booked in the slot (not service type)
   */
  appointmentType?: CodeableConcept;
  /** 
   * The schedule resource that this slot defines an interval of status information
   */
  schedule: Reference;
  /** 
   * busy | free | busy-unavailable | busy-tentative | entered-in-error
   */
  status: code;
  /** 
   * busy | free | busy-unavailable | busy-tentative | entered-in-error
   */
  _status?: Element
  /** 
   * Date/Time that the slot is to begin
   */
  start: instant;
  /** 
   * Date/Time that the slot is to begin
   */
  _start?: Element
  /** 
   * Date/Time that the slot is to conclude
   */
  end: instant;
  /** 
   * Date/Time that the slot is to conclude
   */
  _end?: Element
  /** 
   * This slot has already been overbooked, appointments are unlikely to be accepted for this time
   */
  overbooked?: boolean;
  /** 
   * This slot has already been overbooked, appointments are unlikely to be accepted for this time
   */
  _overbooked?: Element
  /** 
   * Comments on the slot to describe any extended information. Such as custom constraints on the slot
   */
  comment?: string;
  /** 
   * Comments on the slot to describe any extended information. Such as custom constraints on the slot
   */
  _comment?: Element
}

export interface SpecimenCollection {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Who collected the specimen
   */
  collector?: Reference;
  /** 
   * Collection time
   */
  collectedDateTime?: dateTime;
  /** 
   * Collection time
   */
  _collectedDateTime?: Element
  /** 
   * Collection time
   */
  collectedPeriod?: Period;
  /** 
   * How long it took to collect specimen
   */
  duration?: Duration;
  /** 
   * The quantity of specimen collected
   */
  quantity?: Quantity;
  /** 
   * Technique used to perform collection
   */
  method?: CodeableConcept;
  /** 
   * Anatomical collection site
   */
  bodySite?: CodeableConcept;
  /** 
   * Whether or how long patient abstained from food and/or drink
   */
  fastingStatusCodeableConcept?: CodeableConcept;
  /** 
   * Whether or how long patient abstained from food and/or drink
   */
  fastingStatusDuration?: Duration;
}
export interface SpecimenProcessing {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Textual description of procedure
   */
  description?: string;
  /** 
   * Textual description of procedure
   */
  _description?: Element
  /** 
   * Indicates the treatment step  applied to the specimen
   */
  procedure?: CodeableConcept;
  /** 
   * Material used in the processing step
   */
  additive?: Array<Reference>;
  /** 
   * Date and time of specimen processing
   */
  timeDateTime?: dateTime;
  /** 
   * Date and time of specimen processing
   */
  _timeDateTime?: Element
  /** 
   * Date and time of specimen processing
   */
  timePeriod?: Period;
}
export interface SpecimenContainer {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Id for the container
   */
  identifier?: Array<Identifier>;
  /** 
   * Textual description of the container
   */
  description?: string;
  /** 
   * Textual description of the container
   */
  _description?: Element
  /** 
   * Kind of container directly associated with specimen
   */
  type?: CodeableConcept;
  /** 
   * Container volume or size
   */
  capacity?: Quantity;
  /** 
   * Quantity of specimen within container
   */
  specimenQuantity?: Quantity;
  /** 
   * Additive associated with container
   */
  additiveCodeableConcept?: CodeableConcept;
  /** 
   * Additive associated with container
   */
  additiveReference?: Reference;
}
export interface Specimen {
resourceType: "Specimen"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * External Identifier
   */
  identifier?: Array<Identifier>;
  /** 
   * Identifier assigned by the lab
   */
  accessionIdentifier?: Identifier;
  /** 
   * available | unavailable | unsatisfactory | entered-in-error
   */
  status?: code;
  /** 
   * available | unavailable | unsatisfactory | entered-in-error
   */
  _status?: Element
  /** 
   * Kind of material that forms the specimen
   */
  type?: CodeableConcept;
  /** 
   * Where the specimen came from. This may be from patient(s), from a location (e.g., the source of an environmental sample), or a sampling of a substance or a device
   */
  subject?: Reference;
  /** 
   * The time when specimen was received for processing
   */
  receivedTime?: dateTime;
  /** 
   * The time when specimen was received for processing
   */
  _receivedTime?: Element
  /** 
   * Specimen from which this specimen originated
   */
  parent?: Array<Reference>;
  /** 
   * Why the specimen was collected
   */
  request?: Array<Reference>;
  /** 
   * Collection details
   */
  collection?: SpecimenCollection;
  /** 
   * Processing and processing step details
   */
  processing?: Array<SpecimenProcessing>;
  /** 
   * Direct container of specimen (tube/slide, etc.)
   */
  container?: Array<SpecimenContainer>;
  /** 
   * State of the specimen
   */
  condition?: Array<CodeableConcept>;
  /** 
   * Comments
   */
  note?: Array<Annotation>;
}

export interface SpecimenDefinitionTypeTestedContainerAdditive {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Additive associated with container
   */
  additiveCodeableConcept?: CodeableConcept;
  /** 
   * Additive associated with container
   */
  additiveReference?: Reference;
}
export interface SpecimenDefinitionTypeTestedContainer {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Container material
   */
  material?: CodeableConcept;
  /** 
   * Kind of container associated with the kind of specimen
   */
  type?: CodeableConcept;
  /** 
   * Color of container cap
   */
  cap?: CodeableConcept;
  /** 
   * Container description
   */
  description?: string;
  /** 
   * Container description
   */
  _description?: Element
  /** 
   * Container capacity
   */
  capacity?: Quantity;
  /** 
   * Minimum volume
   */
  minimumVolumeQuantity?: Quantity;
  /** 
   * Minimum volume
   */
  minimumVolumeString?: string;
  /** 
   * Minimum volume
   */
  _minimumVolumeString?: Element
  /** 
   * Additive associated with container
   */
  additive?: Array<SpecimenDefinitionTypeTestedContainerAdditive>;
  /** 
   * Specimen container preparation
   */
  preparation?: string;
  /** 
   * Specimen container preparation
   */
  _preparation?: Element
}
export interface SpecimenDefinitionTypeTestedHandling {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Temperature qualifier
   */
  temperatureQualifier?: CodeableConcept;
  /** 
   * Temperature range
   */
  temperatureRange?: Range;
  /** 
   * Maximum preservation time
   */
  maxDuration?: Duration;
  /** 
   * Preservation instruction
   */
  instruction?: string;
  /** 
   * Preservation instruction
   */
  _instruction?: Element
}
export interface SpecimenDefinitionTypeTested {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Primary or secondary specimen
   */
  isDerived?: boolean;
  /** 
   * Primary or secondary specimen
   */
  _isDerived?: Element
  /** 
   * Type of intended specimen
   */
  type?: CodeableConcept;
  /** 
   * preferred | alternate
   */
  preference: code;
  /** 
   * preferred | alternate
   */
  _preference?: Element
  /** 
   * The specimen's container
   */
  container?: SpecimenDefinitionTypeTestedContainer;
  /** 
   * Specimen requirements
   */
  requirement?: string;
  /** 
   * Specimen requirements
   */
  _requirement?: Element
  /** 
   * Specimen retention time
   */
  retentionTime?: Duration;
  /** 
   * Rejection criterion
   */
  rejectionCriterion?: Array<CodeableConcept>;
  /** 
   * Specimen handling before testing
   */
  handling?: Array<SpecimenDefinitionTypeTestedHandling>;
}
export interface SpecimenDefinition {
resourceType: "SpecimenDefinition"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business identifier of a kind of specimen
   */
  identifier?: Identifier;
  /** 
   * Kind of material to collect
   */
  typeCollected?: CodeableConcept;
  /** 
   * Patient preparation for collection
   */
  patientPreparation?: Array<CodeableConcept>;
  /** 
   * Time aspect for collection
   */
  timeAspect?: string;
  /** 
   * Time aspect for collection
   */
  _timeAspect?: Element
  /** 
   * Specimen collection procedure
   */
  collection?: Array<CodeableConcept>;
  /** 
   * Specimen in container intended for testing by lab
   */
  typeTested?: Array<SpecimenDefinitionTypeTested>;
}

export interface StructureDefinitionMapping {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Internal id when this mapping is used
   */
  identity: id;
  /** 
   * Internal id when this mapping is used
   */
  _identity?: Element
  /** 
   * Identifies what this mapping refers to
   */
  uri?: uri;
  /** 
   * Identifies what this mapping refers to
   */
  _uri?: Element
  /** 
   * Names what this mapping refers to
   */
  name?: string;
  /** 
   * Names what this mapping refers to
   */
  _name?: Element
  /** 
   * Versions, Issues, Scope limitations etc.
   */
  comment?: string;
  /** 
   * Versions, Issues, Scope limitations etc.
   */
  _comment?: Element
}
export interface StructureDefinitionContext {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * fhirpath | element | extension
   */
  type: code;
  /** 
   * fhirpath | element | extension
   */
  _type?: Element
  /** 
   * Where the extension can be used in instances
   */
  expression: string;
  /** 
   * Where the extension can be used in instances
   */
  _expression?: Element
}
export interface StructureDefinitionSnapshot {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Definition of elements in the resource (if no StructureDefinition)
   */
  element: Array<ElementDefinition>;
}
export interface StructureDefinitionDifferential {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Definition of elements in the resource (if no StructureDefinition)
   */
  element: Array<ElementDefinition>;
}
export interface StructureDefinition {
resourceType: "StructureDefinition"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Canonical identifier for this structure definition, represented as a URI (globally unique)
   */
  url: uri;
  /** 
   * Canonical identifier for this structure definition, represented as a URI (globally unique)
   */
  _url?: Element
  /** 
   * Additional identifier for the structure definition
   */
  identifier?: Array<Identifier>;
  /** 
   * Business version of the structure definition
   */
  version?: string;
  /** 
   * Business version of the structure definition
   */
  _version?: Element
  /** 
   * Name for this structure definition (computer friendly)
   */
  name: string;
  /** 
   * Name for this structure definition (computer friendly)
   */
  _name?: Element
  /** 
   * Name for this structure definition (human friendly)
   */
  title?: string;
  /** 
   * Name for this structure definition (human friendly)
   */
  _title?: Element
  /** 
   * draft | active | retired | unknown
   */
  status: code;
  /** 
   * draft | active | retired | unknown
   */
  _status?: Element
  /** 
   * For testing purposes, not real usage
   */
  experimental?: boolean;
  /** 
   * For testing purposes, not real usage
   */
  _experimental?: Element
  /** 
   * Date last changed
   */
  date?: dateTime;
  /** 
   * Date last changed
   */
  _date?: Element
  /** 
   * Name of the publisher (organization or individual)
   */
  publisher?: string;
  /** 
   * Name of the publisher (organization or individual)
   */
  _publisher?: Element
  /** 
   * Contact details for the publisher
   */
  contact?: Array<ContactDetail>;
  /** 
   * Natural language description of the structure definition
   */
  description?: markdown;
  /** 
   * Natural language description of the structure definition
   */
  _description?: Element
  /** 
   * The context that the content is intended to support
   */
  useContext?: Array<UsageContext>;
  /** 
   * Intended jurisdiction for structure definition (if applicable)
   */
  jurisdiction?: Array<CodeableConcept>;
  /** 
   * Why this structure definition is defined
   */
  purpose?: markdown;
  /** 
   * Why this structure definition is defined
   */
  _purpose?: Element
  /** 
   * Use and/or publishing restrictions
   */
  copyright?: markdown;
  /** 
   * Use and/or publishing restrictions
   */
  _copyright?: Element
  /** 
   * Assist with indexing and finding
   */
  keyword?: Array<Coding>;
  /** 
   * FHIR Version this StructureDefinition targets
   */
  fhirVersion?: code;
  /** 
   * FHIR Version this StructureDefinition targets
   */
  _fhirVersion?: Element
  /** 
   * External specification that the content is mapped to
   */
  mapping?: Array<StructureDefinitionMapping>;
  /** 
   * primitive-type | complex-type | resource | logical
   */
  kind: code;
  /** 
   * primitive-type | complex-type | resource | logical
   */
  _kind?: Element
  /** 
   * Whether the structure is abstract
   */
  abstract: boolean;
  /** 
   * Whether the structure is abstract
   */
  _abstract?: Element
  /** 
   * If an extension, where it can be used in instances
   */
  context?: Array<StructureDefinitionContext>;
  /** 
   * FHIRPath invariants - when the extension can be used
   */
  contextInvariant?: Array<string>;
  /** 
   * FHIRPath invariants - when the extension can be used
   */
  _contextInvariant?: Array<Element>
  /** 
   * Type defined or constrained by this structure
   */
  type: uri;
  /** 
   * Type defined or constrained by this structure
   */
  _type?: Element
  /** 
   * Definition that this type is constrained/specialized from
   */
  baseDefinition?: canonical;
  /** 
   * Definition that this type is constrained/specialized from
   */
  _baseDefinition?: Element
  /** 
   * specialization | constraint - How relates to base definition
   */
  derivation?: code;
  /** 
   * specialization | constraint - How relates to base definition
   */
  _derivation?: Element
  /** 
   * Snapshot view of the structure
   */
  snapshot?: StructureDefinitionSnapshot;
  /** 
   * Differential view of the structure
   */
  differential?: StructureDefinitionDifferential;
}

export interface StructureMapStructure {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Canonical reference to structure definition
   */
  url: canonical;
  /** 
   * Canonical reference to structure definition
   */
  _url?: Element
  /** 
   * source | queried | target | produced
   */
  mode: code;
  /** 
   * source | queried | target | produced
   */
  _mode?: Element
  /** 
   * Name for type in this map
   */
  alias?: string;
  /** 
   * Name for type in this map
   */
  _alias?: Element
  /** 
   * Documentation on use of structure
   */
  documentation?: string;
  /** 
   * Documentation on use of structure
   */
  _documentation?: Element
}
export interface StructureMapGroupInput {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Name for this instance of data
   */
  name: id;
  /** 
   * Name for this instance of data
   */
  _name?: Element
  /** 
   * Type for this instance of data
   */
  type?: string;
  /** 
   * Type for this instance of data
   */
  _type?: Element
  /** 
   * source | target
   */
  mode: code;
  /** 
   * source | target
   */
  _mode?: Element
  /** 
   * Documentation for this instance of data
   */
  documentation?: string;
  /** 
   * Documentation for this instance of data
   */
  _documentation?: Element
}
export interface StructureMapGroupRuleSource {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Type or variable this rule applies to
   */
  context: id;
  /** 
   * Type or variable this rule applies to
   */
  _context?: Element
  /** 
   * Specified minimum cardinality
   */
  min?: integer;
  /** 
   * Specified minimum cardinality
   */
  _min?: Element
  /** 
   * Specified maximum cardinality (number or *)
   */
  max?: string;
  /** 
   * Specified maximum cardinality (number or *)
   */
  _max?: Element
  /** 
   * Rule only applies if source has this type
   */
  type?: string;
  /** 
   * Rule only applies if source has this type
   */
  _type?: Element
  /** 
   * Default value if no value exists
   */
  defaultValueBase64Binary?: base64Binary;
  /** 
   * Default value if no value exists
   */
  _defaultValueBase64Binary?: Element
  /** 
   * Default value if no value exists
   */
  defaultValueBoolean?: boolean;
  /** 
   * Default value if no value exists
   */
  _defaultValueBoolean?: Element
  /** 
   * Default value if no value exists
   */
  defaultValueCanonical?: canonical;
  /** 
   * Default value if no value exists
   */
  _defaultValueCanonical?: Element
  /** 
   * Default value if no value exists
   */
  defaultValueCode?: code;
  /** 
   * Default value if no value exists
   */
  _defaultValueCode?: Element
  /** 
   * Default value if no value exists
   */
  defaultValueDate?: date;
  /** 
   * Default value if no value exists
   */
  _defaultValueDate?: Element
  /** 
   * Default value if no value exists
   */
  defaultValueDateTime?: dateTime;
  /** 
   * Default value if no value exists
   */
  _defaultValueDateTime?: Element
  /** 
   * Default value if no value exists
   */
  defaultValueDecimal?: decimal;
  /** 
   * Default value if no value exists
   */
  _defaultValueDecimal?: Element
  /** 
   * Default value if no value exists
   */
  defaultValueId?: id;
  /** 
   * Default value if no value exists
   */
  _defaultValueId?: Element
  /** 
   * Default value if no value exists
   */
  defaultValueInstant?: instant;
  /** 
   * Default value if no value exists
   */
  _defaultValueInstant?: Element
  /** 
   * Default value if no value exists
   */
  defaultValueInteger?: integer;
  /** 
   * Default value if no value exists
   */
  _defaultValueInteger?: Element
  /** 
   * Default value if no value exists
   */
  defaultValueMarkdown?: markdown;
  /** 
   * Default value if no value exists
   */
  _defaultValueMarkdown?: Element
  /** 
   * Default value if no value exists
   */
  defaultValueOid?: oid;
  /** 
   * Default value if no value exists
   */
  _defaultValueOid?: Element
  /** 
   * Default value if no value exists
   */
  defaultValuePositiveInt?: positiveInt;
  /** 
   * Default value if no value exists
   */
  _defaultValuePositiveInt?: Element
  /** 
   * Default value if no value exists
   */
  defaultValueString?: string;
  /** 
   * Default value if no value exists
   */
  _defaultValueString?: Element
  /** 
   * Default value if no value exists
   */
  defaultValueTime?: time;
  /** 
   * Default value if no value exists
   */
  _defaultValueTime?: Element
  /** 
   * Default value if no value exists
   */
  defaultValueUnsignedInt?: unsignedInt;
  /** 
   * Default value if no value exists
   */
  _defaultValueUnsignedInt?: Element
  /** 
   * Default value if no value exists
   */
  defaultValueUri?: uri;
  /** 
   * Default value if no value exists
   */
  _defaultValueUri?: Element
  /** 
   * Default value if no value exists
   */
  defaultValueUrl?: url;
  /** 
   * Default value if no value exists
   */
  _defaultValueUrl?: Element
  /** 
   * Default value if no value exists
   */
  defaultValueUuid?: uuid;
  /** 
   * Default value if no value exists
   */
  _defaultValueUuid?: Element
  /** 
   * Default value if no value exists
   */
  defaultValueAddress?: Address;
  /** 
   * Default value if no value exists
   */
  defaultValueAge?: Age;
  /** 
   * Default value if no value exists
   */
  defaultValueAnnotation?: Annotation;
  /** 
   * Default value if no value exists
   */
  defaultValueAttachment?: Attachment;
  /** 
   * Default value if no value exists
   */
  defaultValueCodeableConcept?: CodeableConcept;
  /** 
   * Default value if no value exists
   */
  defaultValueCoding?: Coding;
  /** 
   * Default value if no value exists
   */
  defaultValueContactPoint?: ContactPoint;
  /** 
   * Default value if no value exists
   */
  defaultValueCount?: Count;
  /** 
   * Default value if no value exists
   */
  defaultValueDistance?: Distance;
  /** 
   * Default value if no value exists
   */
  defaultValueDuration?: Duration;
  /** 
   * Default value if no value exists
   */
  defaultValueHumanName?: HumanName;
  /** 
   * Default value if no value exists
   */
  defaultValueIdentifier?: Identifier;
  /** 
   * Default value if no value exists
   */
  defaultValueMoney?: Money;
  /** 
   * Default value if no value exists
   */
  defaultValuePeriod?: Period;
  /** 
   * Default value if no value exists
   */
  defaultValueQuantity?: Quantity;
  /** 
   * Default value if no value exists
   */
  defaultValueRange?: Range;
  /** 
   * Default value if no value exists
   */
  defaultValueRatio?: Ratio;
  /** 
   * Default value if no value exists
   */
  defaultValueReference?: Reference;
  /** 
   * Default value if no value exists
   */
  defaultValueSampledData?: SampledData;
  /** 
   * Default value if no value exists
   */
  defaultValueSignature?: Signature;
  /** 
   * Default value if no value exists
   */
  defaultValueTiming?: Timing;
  /** 
   * Default value if no value exists
   */
  defaultValueContactDetail?: ContactDetail;
  /** 
   * Default value if no value exists
   */
  defaultValueContributor?: Contributor;
  /** 
   * Default value if no value exists
   */
  defaultValueDataRequirement?: DataRequirement;
  /** 
   * Default value if no value exists
   */
  defaultValueExpression?: Expression;
  /** 
   * Default value if no value exists
   */
  defaultValueParameterDefinition?: ParameterDefinition;
  /** 
   * Default value if no value exists
   */
  defaultValueRelatedArtifact?: RelatedArtifact;
  /** 
   * Default value if no value exists
   */
  defaultValueTriggerDefinition?: TriggerDefinition;
  /** 
   * Default value if no value exists
   */
  defaultValueUsageContext?: UsageContext;
  /** 
   * Default value if no value exists
   */
  defaultValueDosage?: Dosage;
  /** 
   * Default value if no value exists
   */
  defaultValueMeta?: Meta;
  /** 
   * Optional field for this source
   */
  element?: string;
  /** 
   * Optional field for this source
   */
  _element?: Element
  /** 
   * first | not_first | last | not_last | only_one
   */
  listMode?: code;
  /** 
   * first | not_first | last | not_last | only_one
   */
  _listMode?: Element
  /** 
   * Named context for field, if a field is specified
   */
  variable?: id;
  /** 
   * Named context for field, if a field is specified
   */
  _variable?: Element
  /** 
   * FHIRPath expression  - must be true or the rule does not apply
   */
  condition?: string;
  /** 
   * FHIRPath expression  - must be true or the rule does not apply
   */
  _condition?: Element
  /** 
   * FHIRPath expression  - must be true or the mapping engine throws an error instead of completing
   */
  check?: string;
  /** 
   * FHIRPath expression  - must be true or the mapping engine throws an error instead of completing
   */
  _check?: Element
  /** 
   * Message to put in log if source exists (FHIRPath)
   */
  logMessage?: string;
  /** 
   * Message to put in log if source exists (FHIRPath)
   */
  _logMessage?: Element
}
export interface StructureMapGroupRuleTargetParameter {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Parameter value - variable or literal
   */
  valueId?: id;
  /** 
   * Parameter value - variable or literal
   */
  _valueId?: Element
  /** 
   * Parameter value - variable or literal
   */
  valueString?: string;
  /** 
   * Parameter value - variable or literal
   */
  _valueString?: Element
  /** 
   * Parameter value - variable or literal
   */
  valueBoolean?: boolean;
  /** 
   * Parameter value - variable or literal
   */
  _valueBoolean?: Element
  /** 
   * Parameter value - variable or literal
   */
  valueInteger?: integer;
  /** 
   * Parameter value - variable or literal
   */
  _valueInteger?: Element
  /** 
   * Parameter value - variable or literal
   */
  valueDecimal?: decimal;
  /** 
   * Parameter value - variable or literal
   */
  _valueDecimal?: Element
}
export interface StructureMapGroupRuleTarget {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Type or variable this rule applies to
   */
  context?: id;
  /** 
   * Type or variable this rule applies to
   */
  _context?: Element
  /** 
   * type | variable
   */
  contextType?: code;
  /** 
   * type | variable
   */
  _contextType?: Element
  /** 
   * Field to create in the context
   */
  element?: string;
  /** 
   * Field to create in the context
   */
  _element?: Element
  /** 
   * Named context for field, if desired, and a field is specified
   */
  variable?: id;
  /** 
   * Named context for field, if desired, and a field is specified
   */
  _variable?: Element
  /** 
   * first | share | last | collate
   */
  listMode?: Array<code>;
  /** 
   * first | share | last | collate
   */
  _listMode?: Array<Element>
  /** 
   * Internal rule reference for shared list items
   */
  listRuleId?: id;
  /** 
   * Internal rule reference for shared list items
   */
  _listRuleId?: Element
  /** 
   * create | copy +
   */
  transform?: code;
  /** 
   * create | copy +
   */
  _transform?: Element
  /** 
   * Parameters to the transform
   */
  parameter?: Array<StructureMapGroupRuleTargetParameter>;
}
export interface StructureMapGroupRuleDependent {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Name of a rule or group to apply
   */
  name: id;
  /** 
   * Name of a rule or group to apply
   */
  _name?: Element
  /** 
   * Variable to pass to the rule or group
   */
  variable: Array<string>;
  /** 
   * Variable to pass to the rule or group
   */
  _variable?: Array<Element>
}
export interface StructureMapGroupRule {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Name of the rule for internal references
   */
  name: id;
  /** 
   * Name of the rule for internal references
   */
  _name?: Element
  /** 
   * Source inputs to the mapping
   */
  source: Array<StructureMapGroupRuleSource>;
  /** 
   * Content to create because of this mapping rule
   */
  target?: Array<StructureMapGroupRuleTarget>;
  /** 
   * Rules contained in this rule
   */
  rule?: Array<StructureMapGroupRule>;
  /** 
   * Which other rules to apply in the context of this rule
   */
  dependent?: Array<StructureMapGroupRuleDependent>;
  /** 
   * Documentation for this instance of data
   */
  documentation?: string;
  /** 
   * Documentation for this instance of data
   */
  _documentation?: Element
}
export interface StructureMapGroup {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Human-readable label
   */
  name: id;
  /** 
   * Human-readable label
   */
  _name?: Element
  /** 
   * Another group that this group adds rules to
   */
  extends?: id;
  /** 
   * Another group that this group adds rules to
   */
  _extends?: Element
  /** 
   * none | types | type-and-types
   */
  typeMode: code;
  /** 
   * none | types | type-and-types
   */
  _typeMode?: Element
  /** 
   * Additional description/explanation for group
   */
  documentation?: string;
  /** 
   * Additional description/explanation for group
   */
  _documentation?: Element
  /** 
   * Named instance provided when invoking the map
   */
  input: Array<StructureMapGroupInput>;
  /** 
   * Transform Rule from source to target
   */
  rule: Array<StructureMapGroupRule>;
}
export interface StructureMap {
resourceType: "StructureMap"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Canonical identifier for this structure map, represented as a URI (globally unique)
   */
  url: uri;
  /** 
   * Canonical identifier for this structure map, represented as a URI (globally unique)
   */
  _url?: Element
  /** 
   * Additional identifier for the structure map
   */
  identifier?: Array<Identifier>;
  /** 
   * Business version of the structure map
   */
  version?: string;
  /** 
   * Business version of the structure map
   */
  _version?: Element
  /** 
   * Name for this structure map (computer friendly)
   */
  name: string;
  /** 
   * Name for this structure map (computer friendly)
   */
  _name?: Element
  /** 
   * Name for this structure map (human friendly)
   */
  title?: string;
  /** 
   * Name for this structure map (human friendly)
   */
  _title?: Element
  /** 
   * draft | active | retired | unknown
   */
  status: code;
  /** 
   * draft | active | retired | unknown
   */
  _status?: Element
  /** 
   * For testing purposes, not real usage
   */
  experimental?: boolean;
  /** 
   * For testing purposes, not real usage
   */
  _experimental?: Element
  /** 
   * Date last changed
   */
  date?: dateTime;
  /** 
   * Date last changed
   */
  _date?: Element
  /** 
   * Name of the publisher (organization or individual)
   */
  publisher?: string;
  /** 
   * Name of the publisher (organization or individual)
   */
  _publisher?: Element
  /** 
   * Contact details for the publisher
   */
  contact?: Array<ContactDetail>;
  /** 
   * Natural language description of the structure map
   */
  description?: markdown;
  /** 
   * Natural language description of the structure map
   */
  _description?: Element
  /** 
   * The context that the content is intended to support
   */
  useContext?: Array<UsageContext>;
  /** 
   * Intended jurisdiction for structure map (if applicable)
   */
  jurisdiction?: Array<CodeableConcept>;
  /** 
   * Why this structure map is defined
   */
  purpose?: markdown;
  /** 
   * Why this structure map is defined
   */
  _purpose?: Element
  /** 
   * Use and/or publishing restrictions
   */
  copyright?: markdown;
  /** 
   * Use and/or publishing restrictions
   */
  _copyright?: Element
  /** 
   * Structure Definition used by this map
   */
  structure?: Array<StructureMapStructure>;
  /** 
   * Other maps used by this map (canonical URLs)
   */
  import?: Array<canonical>;
  /** 
   * Other maps used by this map (canonical URLs)
   */
  _import?: Array<Element>
  /** 
   * Named sections for reader convenience
   */
  group: Array<StructureMapGroup>;
}

export interface SubscriptionChannel {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * rest-hook | websocket | email | sms | message
   */
  type: code;
  /** 
   * rest-hook | websocket | email | sms | message
   */
  _type?: Element
  /** 
   * Where the channel points to
   */
  endpoint?: url;
  /** 
   * Where the channel points to
   */
  _endpoint?: Element
  /** 
   * MIME type to send, or omit for no payload
   */
  payload?: code;
  /** 
   * MIME type to send, or omit for no payload
   */
  _payload?: Element
  /** 
   * Usage depends on the channel type
   */
  header?: Array<string>;
  /** 
   * Usage depends on the channel type
   */
  _header?: Array<Element>
}
export interface Subscription {
resourceType: "Subscription"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * requested | active | error | off
   */
  status: code;
  /** 
   * requested | active | error | off
   */
  _status?: Element
  /** 
   * Contact details for source (e.g. troubleshooting)
   */
  contact?: Array<ContactPoint>;
  /** 
   * When to automatically delete the subscription
   */
  end?: instant;
  /** 
   * When to automatically delete the subscription
   */
  _end?: Element
  /** 
   * Description of why this subscription was created
   */
  reason: string;
  /** 
   * Description of why this subscription was created
   */
  _reason?: Element
  /** 
   * Rule for server push
   */
  criteria: string;
  /** 
   * Rule for server push
   */
  _criteria?: Element
  /** 
   * Latest error note
   */
  error?: string;
  /** 
   * Latest error note
   */
  _error?: Element
  /** 
   * The channel on which to report matches to the criteria
   */
  channel: SubscriptionChannel;
}

export interface SubscriptionStatusNotificationEvent {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Event number
   */
  eventNumber: string;
  /** 
   * Event number
   */
  _eventNumber?: Element
  /** 
   * The instant this event occurred
   */
  timestamp?: instant;
  /** 
   * The instant this event occurred
   */
  _timestamp?: Element
  /** 
   * The focus of this event
   */
  focus?: Reference;
  /** 
   * Additional context for this event
   */
  additionalContext?: Array<Reference>;
}
export interface SubscriptionStatus {
resourceType: "SubscriptionStatus"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * requested | active | error | off | entered-in-error
   */
  status?: code;
  /** 
   * requested | active | error | off | entered-in-error
   */
  _status?: Element
  /** 
   * handshake | heartbeat | event-notification | query-status | query-event
   */
  type: code;
  /** 
   * handshake | heartbeat | event-notification | query-status | query-event
   */
  _type?: Element
  /** 
   * Events since the Subscription was created
   */
  eventsSinceSubscriptionStart?: string;
  /** 
   * Events since the Subscription was created
   */
  _eventsSinceSubscriptionStart?: Element
  /** 
   * Detailed information about any events relevant to this notification
   */
  notificationEvent?: Array<SubscriptionStatusNotificationEvent>;
  /** 
   * Reference to the Subscription responsible for this notification
   */
  subscription: Reference;
  /** 
   * Reference to the SubscriptionTopic this notification relates to
   */
  topic?: canonical;
  /** 
   * Reference to the SubscriptionTopic this notification relates to
   */
  _topic?: Element
  /** 
   * List of errors on the subscription
   */
  error?: Array<CodeableConcept>;
}

export interface SubscriptionTopicResourceTriggerQueryCriteria {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Rule applied to previous resource state
   */
  previous?: string;
  /** 
   * Rule applied to previous resource state
   */
  _previous?: Element
  /** 
   * test-passes | test-fails
   */
  resultForCreate?: code;
  /** 
   * test-passes | test-fails
   */
  _resultForCreate?: Element
  /** 
   * Rule applied to current resource state
   */
  current?: string;
  /** 
   * Rule applied to current resource state
   */
  _current?: Element
  /** 
   * test-passes | test-fails
   */
  resultForDelete?: code;
  /** 
   * test-passes | test-fails
   */
  _resultForDelete?: Element
  /** 
   * Both must be true flag
   */
  requireBoth?: boolean;
  /** 
   * Both must be true flag
   */
  _requireBoth?: Element
}
export interface SubscriptionTopicResourceTrigger {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Text representation of the resource trigger
   */
  description?: markdown;
  /** 
   * Text representation of the resource trigger
   */
  _description?: Element
  /** 
   * Data Type or Resource (reference to definition) for this trigger definition
   */
  resource: uri;
  /** 
   * Data Type or Resource (reference to definition) for this trigger definition
   */
  _resource?: Element
  /** 
   * create | update | delete
   */
  supportedInteraction?: Array<code>;
  /** 
   * create | update | delete
   */
  _supportedInteraction?: Array<Element>
  /** 
   * Query based trigger rule
   */
  queryCriteria?: SubscriptionTopicResourceTriggerQueryCriteria;
  /** 
   * FHIRPath based trigger rule
   */
  fhirPathCriteria?: string;
  /** 
   * FHIRPath based trigger rule
   */
  _fhirPathCriteria?: Element
}
export interface SubscriptionTopicEventTrigger {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Text representation of the event trigger
   */
  description?: markdown;
  /** 
   * Text representation of the event trigger
   */
  _description?: Element
  /** 
   * Event which can trigger a notification from the SubscriptionTopic
   */
  event: CodeableConcept;
  /** 
   * Data Type or Resource (reference to definition) for this trigger definition
   */
  resource: uri;
  /** 
   * Data Type or Resource (reference to definition) for this trigger definition
   */
  _resource?: Element
}
export interface SubscriptionTopicCanFilterBy {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Description of this filter parameter
   */
  description?: markdown;
  /** 
   * Description of this filter parameter
   */
  _description?: Element
  /** 
   * URL of the triggering Resource that this filter applies to
   */
  resource?: uri;
  /** 
   * URL of the triggering Resource that this filter applies to
   */
  _resource?: Element
  /** 
   * Human-readable and computation-friendly name for a filter parameter usable by subscriptions on this topic, via Subscription.filterBy.filterParameter
   */
  filterParameter: string;
  /** 
   * Human-readable and computation-friendly name for a filter parameter usable by subscriptions on this topic, via Subscription.filterBy.filterParameter
   */
  _filterParameter?: Element
  /** 
   * Canonical URL for a filterParameter definition
   */
  filterDefinition?: uri;
  /** 
   * Canonical URL for a filterParameter definition
   */
  _filterDefinition?: Element
  /** 
   * = | eq | ne | gt | lt | ge | le | sa | eb | ap | above | below | in | not-in | of-type
   */
  modifier?: Array<code>;
  /** 
   * = | eq | ne | gt | lt | ge | le | sa | eb | ap | above | below | in | not-in | of-type
   */
  _modifier?: Array<Element>
}
export interface SubscriptionTopicNotificationShape {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * URL of the Resource that is the focus (main) resource in a notification shape
   */
  resource: uri;
  /** 
   * URL of the Resource that is the focus (main) resource in a notification shape
   */
  _resource?: Element
  /** 
   * Include directives, rooted in the resource for this shape
   */
  include?: Array<string>;
  /** 
   * Include directives, rooted in the resource for this shape
   */
  _include?: Array<Element>
  /** 
   * Reverse include directives, rooted in the resource for this shape
   */
  revInclude?: Array<string>;
  /** 
   * Reverse include directives, rooted in the resource for this shape
   */
  _revInclude?: Array<Element>
}
export interface SubscriptionTopic {
resourceType: "SubscriptionTopic"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Canonical identifier for this subscription topic definition, represented as a URI (globally unique)
   */
  url: uri;
  /** 
   * Canonical identifier for this subscription topic definition, represented as a URI (globally unique)
   */
  _url?: Element
  /** 
   * Business Identifier for this subscription topic
   */
  identifier?: Array<Identifier>;
  /** 
   * Business version of the subscription topic
   */
  version?: string;
  /** 
   * Business version of the subscription topic
   */
  _version?: Element
  /** 
   * Name for this subscription topic (Human friendly)
   */
  title?: string;
  /** 
   * Name for this subscription topic (Human friendly)
   */
  _title?: Element
  /** 
   * Based on FHIR protocol or definition
   */
  derivedFrom?: Array<canonical>;
  /** 
   * Based on FHIR protocol or definition
   */
  _derivedFrom?: Array<Element>
  /** 
   * draft | active | retired | unknown
   */
  status: code;
  /** 
   * draft | active | retired | unknown
   */
  _status?: Element
  /** 
   * If for testing purposes, not real usage
   */
  experimental?: boolean;
  /** 
   * If for testing purposes, not real usage
   */
  _experimental?: Element
  /** 
   * Date status first applied
   */
  date?: dateTime;
  /** 
   * Date status first applied
   */
  _date?: Element
  /** 
   * The name of the individual or organization that published the SubscriptionTopic
   */
  publisher?: string;
  /** 
   * The name of the individual or organization that published the SubscriptionTopic
   */
  _publisher?: Element
  /** 
   * Contact details for the publisher
   */
  contact?: Array<ContactDetail>;
  /** 
   * Natural language description of the SubscriptionTopic
   */
  description?: markdown;
  /** 
   * Natural language description of the SubscriptionTopic
   */
  _description?: Element
  /** 
   * Content intends to support these contexts
   */
  useContext?: Array<UsageContext>;
  /** 
   * Intended jurisdiction of the SubscriptionTopic (if applicable)
   */
  jurisdiction?: Array<CodeableConcept>;
  /** 
   * Why this SubscriptionTopic is defined
   */
  purpose?: markdown;
  /** 
   * Why this SubscriptionTopic is defined
   */
  _purpose?: Element
  /** 
   * Use and/or publishing restrictions
   */
  copyright?: markdown;
  /** 
   * Use and/or publishing restrictions
   */
  _copyright?: Element
  /** 
   * When SubscriptionTopic is/was approved by publisher
   */
  approvalDate?: date;
  /** 
   * When SubscriptionTopic is/was approved by publisher
   */
  _approvalDate?: Element
  /** 
   * Date the Subscription Topic was last reviewed by the publisher
   */
  lastReviewDate?: date;
  /** 
   * Date the Subscription Topic was last reviewed by the publisher
   */
  _lastReviewDate?: Element
  /** 
   * The effective date range for the SubscriptionTopic
   */
  effectivePeriod?: Period;
  /** 
   * Definition of a resource-based trigger for the subscription topic
   */
  resourceTrigger?: Array<SubscriptionTopicResourceTrigger>;
  /** 
   * Event definitions the SubscriptionTopic
   */
  eventTrigger?: Array<SubscriptionTopicEventTrigger>;
  /** 
   * Properties by which a Subscription can filter notifications from the SubscriptionTopic
   */
  canFilterBy?: Array<SubscriptionTopicCanFilterBy>;
  /** 
   * Properties for describing the shape of notifications generated by this topic
   */
  notificationShape?: Array<SubscriptionTopicNotificationShape>;
}

export interface SubstanceInstance {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Identifier of the package/container
   */
  identifier?: Identifier;
  /** 
   * When no longer valid to use
   */
  expiry?: dateTime;
  /** 
   * When no longer valid to use
   */
  _expiry?: Element
  /** 
   * Amount of substance in the package
   */
  quantity?: Quantity;
}
export interface SubstanceIngredient {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Optional amount (concentration)
   */
  quantity?: Ratio;
  /** 
   * A component of the substance
   */
  substanceCodeableConcept?: CodeableConcept;
  /** 
   * A component of the substance
   */
  substanceReference?: Reference;
}
export interface Substance {
resourceType: "Substance"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Unique identifier
   */
  identifier?: Array<Identifier>;
  /** 
   * active | inactive | entered-in-error
   */
  status?: code;
  /** 
   * active | inactive | entered-in-error
   */
  _status?: Element
  /** 
   * What class/type of substance this is
   */
  category?: Array<CodeableConcept>;
  /** 
   * What substance this is
   */
  code: CodeableConcept;
  /** 
   * Textual description of the substance, comments
   */
  description?: string;
  /** 
   * Textual description of the substance, comments
   */
  _description?: Element
  /** 
   * If this describes a specific package/container of the substance
   */
  instance?: Array<SubstanceInstance>;
  /** 
   * Composition information about the substance
   */
  ingredient?: Array<SubstanceIngredient>;
}

export interface SubstanceDefinitionMoiety {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Role that the moiety is playing
   */
  role?: CodeableConcept;
  /** 
   * Identifier by which this moiety substance is known
   */
  identifier?: Identifier;
  /** 
   * Textual name for this moiety substance
   */
  name?: string;
  /** 
   * Textual name for this moiety substance
   */
  _name?: Element
  /** 
   * Stereochemistry type
   */
  stereochemistry?: CodeableConcept;
  /** 
   * Optical activity type
   */
  opticalActivity?: CodeableConcept;
  /** 
   * Molecular formula for this moiety (e.g. with the Hill system)
   */
  molecularFormula?: string;
  /** 
   * Molecular formula for this moiety (e.g. with the Hill system)
   */
  _molecularFormula?: Element
  /** 
   * Quantitative value for this moiety
   */
  amountQuantity?: Quantity;
  /** 
   * Quantitative value for this moiety
   */
  amountString?: string;
  /** 
   * Quantitative value for this moiety
   */
  _amountString?: Element
  /** 
   * The measurement type of the quantitative value
   */
  measurementType?: CodeableConcept;
}
export interface SubstanceDefinitionProperty {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * A code expressing the type of property
   */
  type: CodeableConcept;
  /** 
   * A value for the property
   */
  valueCodeableConcept?: CodeableConcept;
  /** 
   * A value for the property
   */
  valueQuantity?: Quantity;
  /** 
   * A value for the property
   */
  valueDate?: date;
  /** 
   * A value for the property
   */
  _valueDate?: Element
  /** 
   * A value for the property
   */
  valueBoolean?: boolean;
  /** 
   * A value for the property
   */
  _valueBoolean?: Element
  /** 
   * A value for the property
   */
  valueAttachment?: Attachment;
}
export interface SubstanceDefinitionMolecularWeight {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The method by which the weight was determined
   */
  method?: CodeableConcept;
  /** 
   * Type of molecular weight e.g. exact, average, weight average
   */
  type?: CodeableConcept;
  /** 
   * Used to capture quantitative values for a variety of elements
   */
  amount: Quantity;
}
export interface SubstanceDefinitionStructureRepresentation {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The kind of structural representation (e.g. full, partial)
   */
  type?: CodeableConcept;
  /** 
   * The structural representation or characterization as a text string in a standard format
   */
  representation?: string;
  /** 
   * The structural representation or characterization as a text string in a standard format
   */
  _representation?: Element
  /** 
   * The format of the representation e.g. InChI, SMILES, MOLFILE (note: not the physical file format)
   */
  format?: CodeableConcept;
  /** 
   * An attachment with the structural representation e.g. a structure graphic or AnIML file
   */
  document?: Reference;
}
export interface SubstanceDefinitionStructure {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Stereochemistry type
   */
  stereochemistry?: CodeableConcept;
  /** 
   * Optical activity type
   */
  opticalActivity?: CodeableConcept;
  /** 
   * Molecular formula (e.g. using the Hill system)
   */
  molecularFormula?: string;
  /** 
   * Molecular formula (e.g. using the Hill system)
   */
  _molecularFormula?: Element
  /** 
   * Specified per moiety according to the Hill system
   */
  molecularFormulaByMoiety?: string;
  /** 
   * Specified per moiety according to the Hill system
   */
  _molecularFormulaByMoiety?: Element
  /** 
   * The molecular weight or weight range
   */
  molecularWeight?: SubstanceDefinitionMolecularWeight;
  /** 
   * The method used to find the structure e.g. X-ray, NMR
   */
  technique?: Array<CodeableConcept>;
  /** 
   * Source of information for the structure
   */
  sourceDocument?: Array<Reference>;
  /** 
   * A depiction of the structure or characterization of the substance
   */
  representation?: Array<SubstanceDefinitionStructureRepresentation>;
}
export interface SubstanceDefinitionCode {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The specific code
   */
  code?: CodeableConcept;
  /** 
   * Status of the code assignment, for example 'provisional', 'approved'
   */
  status?: CodeableConcept;
  /** 
   * The date at which the code status was changed
   */
  statusDate?: dateTime;
  /** 
   * The date at which the code status was changed
   */
  _statusDate?: Element
  /** 
   * Any comment can be provided in this field
   */
  note?: Array<Annotation>;
  /** 
   * Supporting literature
   */
  source?: Array<Reference>;
}
export interface SubstanceDefinitionNameOfficial {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Which authority uses this official name
   */
  authority?: CodeableConcept;
  /** 
   * The status of the official name, for example 'draft', 'active'
   */
  status?: CodeableConcept;
  /** 
   * Date of official name change
   */
  date?: dateTime;
  /** 
   * Date of official name change
   */
  _date?: Element
}
export interface SubstanceDefinitionName {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The actual name
   */
  name: string;
  /** 
   * The actual name
   */
  _name?: Element
  /** 
   * Name type e.g. 'systematic',  'scientific, 'brand'
   */
  type?: CodeableConcept;
  /** 
   * The status of the name e.g. 'current', 'proposed'
   */
  status?: CodeableConcept;
  /** 
   * If this is the preferred name for this substance
   */
  preferred?: boolean;
  /** 
   * If this is the preferred name for this substance
   */
  _preferred?: Element
  /** 
   * Human language that the name is written in
   */
  language?: Array<CodeableConcept>;
  /** 
   * The use context of this name e.g. as an active ingredient or as a food colour additive
   */
  domain?: Array<CodeableConcept>;
  /** 
   * The jurisdiction where this name applies
   */
  jurisdiction?: Array<CodeableConcept>;
  /** 
   * A synonym of this particular name, by which the substance is also known
   */
  synonym?: Array<SubstanceDefinitionName>;
  /** 
   * A translation for this name into another human language
   */
  translation?: Array<SubstanceDefinitionName>;
  /** 
   * Details of the official nature of this name
   */
  official?: Array<SubstanceDefinitionNameOfficial>;
  /** 
   * Supporting literature
   */
  source?: Array<Reference>;
}
export interface SubstanceDefinitionRelationship {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * A pointer to another substance, as a resource or a representational code
   */
  substanceDefinitionReference?: Reference;
  /** 
   * A pointer to another substance, as a resource or a representational code
   */
  substanceDefinitionCodeableConcept?: CodeableConcept;
  /** 
   * For example "salt to parent", "active moiety"
   */
  type: CodeableConcept;
  /** 
   * For example where an enzyme strongly bonds with a particular substance, this is a defining relationship for that enzyme, out of several possible relationships
   */
  isDefining?: boolean;
  /** 
   * For example where an enzyme strongly bonds with a particular substance, this is a defining relationship for that enzyme, out of several possible relationships
   */
  _isDefining?: Element
  /** 
   * A numeric factor for the relationship, e.g. that a substance salt has some percentage of active substance in relation to some other
   */
  amountQuantity?: Quantity;
  /** 
   * A numeric factor for the relationship, e.g. that a substance salt has some percentage of active substance in relation to some other
   */
  amountRatio?: Ratio;
  /** 
   * A numeric factor for the relationship, e.g. that a substance salt has some percentage of active substance in relation to some other
   */
  amountString?: string;
  /** 
   * A numeric factor for the relationship, e.g. that a substance salt has some percentage of active substance in relation to some other
   */
  _amountString?: Element
  /** 
   * For use when the numeric has an uncertain range
   */
  ratioHighLimitAmount?: Ratio;
  /** 
   * An operator for the amount, for example "average", "approximately", "less than"
   */
  comparator?: CodeableConcept;
  /** 
   * Supporting literature
   */
  source?: Array<Reference>;
}
export interface SubstanceDefinitionSourceMaterial {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Classification of the origin of the raw material. e.g. cat hair is an Animal source type
   */
  type?: CodeableConcept;
  /** 
   * The genus of an organism e.g. the Latin epithet of the plant/animal scientific name
   */
  genus?: CodeableConcept;
  /** 
   * The species of an organism e.g. the Latin epithet of the species of the plant/animal
   */
  species?: CodeableConcept;
  /** 
   * An anatomical origin of the source material within an organism
   */
  part?: CodeableConcept;
  /** 
   * The country or countries where the material is harvested
   */
  countryOfOrigin?: Array<CodeableConcept>;
}
export interface SubstanceDefinition {
resourceType: "SubstanceDefinition"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Identifier by which this substance is known
   */
  identifier?: Array<Identifier>;
  /** 
   * A business level version identifier of the substance
   */
  version?: string;
  /** 
   * A business level version identifier of the substance
   */
  _version?: Element
  /** 
   * Status of substance within the catalogue e.g. active, retired
   */
  status?: CodeableConcept;
  /** 
   * A categorization, high level e.g. polymer or nucleic acid, or food, chemical, biological, or lower e.g. polymer linear or branch chain, or type of impurity
   */
  classification?: Array<CodeableConcept>;
  /** 
   * If the substance applies to human or veterinary use
   */
  domain?: CodeableConcept;
  /** 
   * The quality standard, established benchmark, to which substance complies (e.g. USP/NF, BP)
   */
  grade?: Array<CodeableConcept>;
  /** 
   * Textual description of the substance
   */
  description?: markdown;
  /** 
   * Textual description of the substance
   */
  _description?: Element
  /** 
   * Supporting literature
   */
  informationSource?: Array<Reference>;
  /** 
   * Textual comment about the substance's catalogue or registry record
   */
  note?: Array<Annotation>;
  /** 
   * The entity that creates, makes, produces or fabricates the substance
   */
  manufacturer?: Array<Reference>;
  /** 
   * An entity that is the source for the substance. It may be different from the manufacturer
   */
  supplier?: Array<Reference>;
  /** 
   * Moiety, for structural modifications
   */
  moiety?: Array<SubstanceDefinitionMoiety>;
  /** 
   * General specifications for this substance
   */
  property?: Array<SubstanceDefinitionProperty>;
  /** 
   * The molecular weight or weight range
   */
  molecularWeight?: Array<SubstanceDefinitionMolecularWeight>;
  /** 
   * Structural information
   */
  structure?: SubstanceDefinitionStructure;
  /** 
   * Codes associated with the substance
   */
  code?: Array<SubstanceDefinitionCode>;
  /** 
   * Names applicable to this substance
   */
  name?: Array<SubstanceDefinitionName>;
  /** 
   * A link between this substance and another
   */
  relationship?: Array<SubstanceDefinitionRelationship>;
  /** 
   * Material or taxonomic/anatomical source
   */
  sourceMaterial?: SubstanceDefinitionSourceMaterial;
}

export interface SupplyDeliverySuppliedItem {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Amount dispensed
   */
  quantity?: Quantity;
  /** 
   * Medication, Substance, or Device supplied
   */
  itemCodeableConcept?: CodeableConcept;
  /** 
   * Medication, Substance, or Device supplied
   */
  itemReference?: Reference;
}
export interface SupplyDelivery {
resourceType: "SupplyDelivery"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * External identifier
   */
  identifier?: Array<Identifier>;
  /** 
   * Fulfills plan, proposal or order
   */
  basedOn?: Array<Reference>;
  /** 
   * Part of referenced event
   */
  partOf?: Array<Reference>;
  /** 
   * in-progress | completed | abandoned | entered-in-error
   */
  status?: code;
  /** 
   * in-progress | completed | abandoned | entered-in-error
   */
  _status?: Element
  /** 
   * Patient for whom the item is supplied
   */
  patient?: Reference;
  /** 
   * Category of dispense event
   */
  type?: CodeableConcept;
  /** 
   * The item that is delivered or supplied
   */
  suppliedItem?: SupplyDeliverySuppliedItem;
  /** 
   * When event occurred
   */
  occurrenceDateTime?: dateTime;
  /** 
   * When event occurred
   */
  _occurrenceDateTime?: Element
  /** 
   * When event occurred
   */
  occurrencePeriod?: Period;
  /** 
   * When event occurred
   */
  occurrenceTiming?: Timing;
  /** 
   * Dispenser
   */
  supplier?: Reference;
  /** 
   * Where the Supply was sent
   */
  destination?: Reference;
  /** 
   * Who collected the Supply
   */
  receiver?: Array<Reference>;
}

export interface SupplyRequestParameter {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Item detail
   */
  code?: CodeableConcept;
  /** 
   * Value of detail
   */
  valueCodeableConcept?: CodeableConcept;
  /** 
   * Value of detail
   */
  valueQuantity?: Quantity;
  /** 
   * Value of detail
   */
  valueRange?: Range;
  /** 
   * Value of detail
   */
  valueBoolean?: boolean;
  /** 
   * Value of detail
   */
  _valueBoolean?: Element
}
export interface SupplyRequest {
resourceType: "SupplyRequest"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business Identifier for SupplyRequest
   */
  identifier?: Array<Identifier>;
  /** 
   * draft | active | suspended +
   */
  status?: code;
  /** 
   * draft | active | suspended +
   */
  _status?: Element
  /** 
   * The kind of supply (central, non-stock, etc.)
   */
  category?: CodeableConcept;
  /** 
   * routine | urgent | asap | stat
   */
  priority?: code;
  /** 
   * routine | urgent | asap | stat
   */
  _priority?: Element
  /** 
   * Medication, Substance, or Device requested to be supplied
   */
  itemCodeableConcept?: CodeableConcept;
  /** 
   * Medication, Substance, or Device requested to be supplied
   */
  itemReference?: Reference;
  /** 
   * The requested amount of the item indicated
   */
  quantity: Quantity;
  /** 
   * Ordered item details
   */
  parameter?: Array<SupplyRequestParameter>;
  /** 
   * When the request should be fulfilled
   */
  occurrenceDateTime?: dateTime;
  /** 
   * When the request should be fulfilled
   */
  _occurrenceDateTime?: Element
  /** 
   * When the request should be fulfilled
   */
  occurrencePeriod?: Period;
  /** 
   * When the request should be fulfilled
   */
  occurrenceTiming?: Timing;
  /** 
   * When the request was made
   */
  authoredOn?: dateTime;
  /** 
   * When the request was made
   */
  _authoredOn?: Element
  /** 
   * Individual making the request
   */
  requester?: Reference;
  /** 
   * Who is intended to fulfill the request
   */
  supplier?: Array<Reference>;
  /** 
   * The reason why the supply item was requested
   */
  reasonCode?: Array<CodeableConcept>;
  /** 
   * The reason why the supply item was requested
   */
  reasonReference?: Array<Reference>;
  /** 
   * The origin of the supply
   */
  deliverFrom?: Reference;
  /** 
   * The destination of the supply
   */
  deliverTo?: Reference;
}

export interface TaskRestriction {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * How many times to repeat
   */
  repetitions?: positiveInt;
  /** 
   * How many times to repeat
   */
  _repetitions?: Element
  /** 
   * When fulfillment sought
   */
  period?: Period;
  /** 
   * For whom is fulfillment sought?
   */
  recipient?: Array<Reference>;
}
export interface TaskInput {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Label for the input
   */
  type: CodeableConcept;
  /** 
   * Content to use in performing the task
   */
  valueBase64Binary?: base64Binary;
  /** 
   * Content to use in performing the task
   */
  _valueBase64Binary?: Element
  /** 
   * Content to use in performing the task
   */
  valueBoolean?: boolean;
  /** 
   * Content to use in performing the task
   */
  _valueBoolean?: Element
  /** 
   * Content to use in performing the task
   */
  valueCanonical?: canonical;
  /** 
   * Content to use in performing the task
   */
  _valueCanonical?: Element
  /** 
   * Content to use in performing the task
   */
  valueCode?: code;
  /** 
   * Content to use in performing the task
   */
  _valueCode?: Element
  /** 
   * Content to use in performing the task
   */
  valueDate?: date;
  /** 
   * Content to use in performing the task
   */
  _valueDate?: Element
  /** 
   * Content to use in performing the task
   */
  valueDateTime?: dateTime;
  /** 
   * Content to use in performing the task
   */
  _valueDateTime?: Element
  /** 
   * Content to use in performing the task
   */
  valueDecimal?: decimal;
  /** 
   * Content to use in performing the task
   */
  _valueDecimal?: Element
  /** 
   * Content to use in performing the task
   */
  valueId?: id;
  /** 
   * Content to use in performing the task
   */
  _valueId?: Element
  /** 
   * Content to use in performing the task
   */
  valueInstant?: instant;
  /** 
   * Content to use in performing the task
   */
  _valueInstant?: Element
  /** 
   * Content to use in performing the task
   */
  valueInteger?: integer;
  /** 
   * Content to use in performing the task
   */
  _valueInteger?: Element
  /** 
   * Content to use in performing the task
   */
  valueMarkdown?: markdown;
  /** 
   * Content to use in performing the task
   */
  _valueMarkdown?: Element
  /** 
   * Content to use in performing the task
   */
  valueOid?: oid;
  /** 
   * Content to use in performing the task
   */
  _valueOid?: Element
  /** 
   * Content to use in performing the task
   */
  valuePositiveInt?: positiveInt;
  /** 
   * Content to use in performing the task
   */
  _valuePositiveInt?: Element
  /** 
   * Content to use in performing the task
   */
  valueString?: string;
  /** 
   * Content to use in performing the task
   */
  _valueString?: Element
  /** 
   * Content to use in performing the task
   */
  valueTime?: time;
  /** 
   * Content to use in performing the task
   */
  _valueTime?: Element
  /** 
   * Content to use in performing the task
   */
  valueUnsignedInt?: unsignedInt;
  /** 
   * Content to use in performing the task
   */
  _valueUnsignedInt?: Element
  /** 
   * Content to use in performing the task
   */
  valueUri?: uri;
  /** 
   * Content to use in performing the task
   */
  _valueUri?: Element
  /** 
   * Content to use in performing the task
   */
  valueUrl?: url;
  /** 
   * Content to use in performing the task
   */
  _valueUrl?: Element
  /** 
   * Content to use in performing the task
   */
  valueUuid?: uuid;
  /** 
   * Content to use in performing the task
   */
  _valueUuid?: Element
  /** 
   * Content to use in performing the task
   */
  valueAddress?: Address;
  /** 
   * Content to use in performing the task
   */
  valueAge?: Age;
  /** 
   * Content to use in performing the task
   */
  valueAnnotation?: Annotation;
  /** 
   * Content to use in performing the task
   */
  valueAttachment?: Attachment;
  /** 
   * Content to use in performing the task
   */
  valueCodeableConcept?: CodeableConcept;
  /** 
   * Content to use in performing the task
   */
  valueCoding?: Coding;
  /** 
   * Content to use in performing the task
   */
  valueContactPoint?: ContactPoint;
  /** 
   * Content to use in performing the task
   */
  valueCount?: Count;
  /** 
   * Content to use in performing the task
   */
  valueDistance?: Distance;
  /** 
   * Content to use in performing the task
   */
  valueDuration?: Duration;
  /** 
   * Content to use in performing the task
   */
  valueHumanName?: HumanName;
  /** 
   * Content to use in performing the task
   */
  valueIdentifier?: Identifier;
  /** 
   * Content to use in performing the task
   */
  valueMoney?: Money;
  /** 
   * Content to use in performing the task
   */
  valuePeriod?: Period;
  /** 
   * Content to use in performing the task
   */
  valueQuantity?: Quantity;
  /** 
   * Content to use in performing the task
   */
  valueRange?: Range;
  /** 
   * Content to use in performing the task
   */
  valueRatio?: Ratio;
  /** 
   * Content to use in performing the task
   */
  valueReference?: Reference;
  /** 
   * Content to use in performing the task
   */
  valueSampledData?: SampledData;
  /** 
   * Content to use in performing the task
   */
  valueSignature?: Signature;
  /** 
   * Content to use in performing the task
   */
  valueTiming?: Timing;
  /** 
   * Content to use in performing the task
   */
  valueContactDetail?: ContactDetail;
  /** 
   * Content to use in performing the task
   */
  valueContributor?: Contributor;
  /** 
   * Content to use in performing the task
   */
  valueDataRequirement?: DataRequirement;
  /** 
   * Content to use in performing the task
   */
  valueExpression?: Expression;
  /** 
   * Content to use in performing the task
   */
  valueParameterDefinition?: ParameterDefinition;
  /** 
   * Content to use in performing the task
   */
  valueRelatedArtifact?: RelatedArtifact;
  /** 
   * Content to use in performing the task
   */
  valueTriggerDefinition?: TriggerDefinition;
  /** 
   * Content to use in performing the task
   */
  valueUsageContext?: UsageContext;
  /** 
   * Content to use in performing the task
   */
  valueDosage?: Dosage;
  /** 
   * Content to use in performing the task
   */
  valueMeta?: Meta;
}
export interface TaskOutput {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Label for output
   */
  type: CodeableConcept;
  /** 
   * Result of output
   */
  valueBase64Binary?: base64Binary;
  /** 
   * Result of output
   */
  _valueBase64Binary?: Element
  /** 
   * Result of output
   */
  valueBoolean?: boolean;
  /** 
   * Result of output
   */
  _valueBoolean?: Element
  /** 
   * Result of output
   */
  valueCanonical?: canonical;
  /** 
   * Result of output
   */
  _valueCanonical?: Element
  /** 
   * Result of output
   */
  valueCode?: code;
  /** 
   * Result of output
   */
  _valueCode?: Element
  /** 
   * Result of output
   */
  valueDate?: date;
  /** 
   * Result of output
   */
  _valueDate?: Element
  /** 
   * Result of output
   */
  valueDateTime?: dateTime;
  /** 
   * Result of output
   */
  _valueDateTime?: Element
  /** 
   * Result of output
   */
  valueDecimal?: decimal;
  /** 
   * Result of output
   */
  _valueDecimal?: Element
  /** 
   * Result of output
   */
  valueId?: id;
  /** 
   * Result of output
   */
  _valueId?: Element
  /** 
   * Result of output
   */
  valueInstant?: instant;
  /** 
   * Result of output
   */
  _valueInstant?: Element
  /** 
   * Result of output
   */
  valueInteger?: integer;
  /** 
   * Result of output
   */
  _valueInteger?: Element
  /** 
   * Result of output
   */
  valueMarkdown?: markdown;
  /** 
   * Result of output
   */
  _valueMarkdown?: Element
  /** 
   * Result of output
   */
  valueOid?: oid;
  /** 
   * Result of output
   */
  _valueOid?: Element
  /** 
   * Result of output
   */
  valuePositiveInt?: positiveInt;
  /** 
   * Result of output
   */
  _valuePositiveInt?: Element
  /** 
   * Result of output
   */
  valueString?: string;
  /** 
   * Result of output
   */
  _valueString?: Element
  /** 
   * Result of output
   */
  valueTime?: time;
  /** 
   * Result of output
   */
  _valueTime?: Element
  /** 
   * Result of output
   */
  valueUnsignedInt?: unsignedInt;
  /** 
   * Result of output
   */
  _valueUnsignedInt?: Element
  /** 
   * Result of output
   */
  valueUri?: uri;
  /** 
   * Result of output
   */
  _valueUri?: Element
  /** 
   * Result of output
   */
  valueUrl?: url;
  /** 
   * Result of output
   */
  _valueUrl?: Element
  /** 
   * Result of output
   */
  valueUuid?: uuid;
  /** 
   * Result of output
   */
  _valueUuid?: Element
  /** 
   * Result of output
   */
  valueAddress?: Address;
  /** 
   * Result of output
   */
  valueAge?: Age;
  /** 
   * Result of output
   */
  valueAnnotation?: Annotation;
  /** 
   * Result of output
   */
  valueAttachment?: Attachment;
  /** 
   * Result of output
   */
  valueCodeableConcept?: CodeableConcept;
  /** 
   * Result of output
   */
  valueCoding?: Coding;
  /** 
   * Result of output
   */
  valueContactPoint?: ContactPoint;
  /** 
   * Result of output
   */
  valueCount?: Count;
  /** 
   * Result of output
   */
  valueDistance?: Distance;
  /** 
   * Result of output
   */
  valueDuration?: Duration;
  /** 
   * Result of output
   */
  valueHumanName?: HumanName;
  /** 
   * Result of output
   */
  valueIdentifier?: Identifier;
  /** 
   * Result of output
   */
  valueMoney?: Money;
  /** 
   * Result of output
   */
  valuePeriod?: Period;
  /** 
   * Result of output
   */
  valueQuantity?: Quantity;
  /** 
   * Result of output
   */
  valueRange?: Range;
  /** 
   * Result of output
   */
  valueRatio?: Ratio;
  /** 
   * Result of output
   */
  valueReference?: Reference;
  /** 
   * Result of output
   */
  valueSampledData?: SampledData;
  /** 
   * Result of output
   */
  valueSignature?: Signature;
  /** 
   * Result of output
   */
  valueTiming?: Timing;
  /** 
   * Result of output
   */
  valueContactDetail?: ContactDetail;
  /** 
   * Result of output
   */
  valueContributor?: Contributor;
  /** 
   * Result of output
   */
  valueDataRequirement?: DataRequirement;
  /** 
   * Result of output
   */
  valueExpression?: Expression;
  /** 
   * Result of output
   */
  valueParameterDefinition?: ParameterDefinition;
  /** 
   * Result of output
   */
  valueRelatedArtifact?: RelatedArtifact;
  /** 
   * Result of output
   */
  valueTriggerDefinition?: TriggerDefinition;
  /** 
   * Result of output
   */
  valueUsageContext?: UsageContext;
  /** 
   * Result of output
   */
  valueDosage?: Dosage;
  /** 
   * Result of output
   */
  valueMeta?: Meta;
}
export interface Task {
resourceType: "Task"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Task Instance Identifier
   */
  identifier?: Array<Identifier>;
  /** 
   * Formal definition of task
   */
  instantiatesCanonical?: canonical;
  /** 
   * Formal definition of task
   */
  _instantiatesCanonical?: Element
  /** 
   * Formal definition of task
   */
  instantiatesUri?: uri;
  /** 
   * Formal definition of task
   */
  _instantiatesUri?: Element
  /** 
   * Request fulfilled by this task
   */
  basedOn?: Array<Reference>;
  /** 
   * Requisition or grouper id
   */
  groupIdentifier?: Identifier;
  /** 
   * Composite task
   */
  partOf?: Array<Reference>;
  /** 
   * draft | requested | received | accepted | +
   */
  status: code;
  /** 
   * draft | requested | received | accepted | +
   */
  _status?: Element
  /** 
   * Reason for current status
   */
  statusReason?: CodeableConcept;
  /** 
   * E.g. "Specimen collected", "IV prepped"
   */
  businessStatus?: CodeableConcept;
  /** 
   * unknown | proposal | plan | order | original-order | reflex-order | filler-order | instance-order | option
   */
  intent: code;
  /** 
   * unknown | proposal | plan | order | original-order | reflex-order | filler-order | instance-order | option
   */
  _intent?: Element
  /** 
   * routine | urgent | asap | stat
   */
  priority?: code;
  /** 
   * routine | urgent | asap | stat
   */
  _priority?: Element
  /** 
   * Task Type
   */
  code?: CodeableConcept;
  /** 
   * Human-readable explanation of task
   */
  description?: string;
  /** 
   * Human-readable explanation of task
   */
  _description?: Element
  /** 
   * What task is acting on
   */
  focus?: Reference;
  /** 
   * Beneficiary of the Task
   */
  for?: Reference;
  /** 
   * Healthcare event during which this task originated
   */
  encounter?: Reference;
  /** 
   * Start and end time of execution
   */
  executionPeriod?: Period;
  /** 
   * Task Creation Date
   */
  authoredOn?: dateTime;
  /** 
   * Task Creation Date
   */
  _authoredOn?: Element
  /** 
   * Task Last Modified Date
   */
  lastModified?: dateTime;
  /** 
   * Task Last Modified Date
   */
  _lastModified?: Element
  /** 
   * Who is asking for task to be done
   */
  requester?: Reference;
  /** 
   * Requested performer
   */
  performerType?: Array<CodeableConcept>;
  /** 
   * Responsible individual
   */
  owner?: Reference;
  /** 
   * Where task occurs
   */
  location?: Reference;
  /** 
   * Why task is needed
   */
  reasonCode?: CodeableConcept;
  /** 
   * Why task is needed
   */
  reasonReference?: Reference;
  /** 
   * Associated insurance coverage
   */
  insurance?: Array<Reference>;
  /** 
   * Comments made about the task
   */
  note?: Array<Annotation>;
  /** 
   * Key events in history of the Task
   */
  relevantHistory?: Array<Reference>;
  /** 
   * Constraints on fulfillment tasks
   */
  restriction?: TaskRestriction;
  /** 
   * Information used to perform task
   */
  input?: Array<TaskInput>;
  /** 
   * Information produced as part of task
   */
  output?: Array<TaskOutput>;
}

export interface TerminologyCapabilitiesSoftware {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * A name the software is known by
   */
  name: string;
  /** 
   * A name the software is known by
   */
  _name?: Element
  /** 
   * Version covered by this statement
   */
  version?: string;
  /** 
   * Version covered by this statement
   */
  _version?: Element
}
export interface TerminologyCapabilitiesImplementation {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Describes this specific instance
   */
  description: string;
  /** 
   * Describes this specific instance
   */
  _description?: Element
  /** 
   * Base URL for the implementation
   */
  url?: url;
  /** 
   * Base URL for the implementation
   */
  _url?: Element
}
export interface TerminologyCapabilitiesCodeSystemVersionFilter {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Code of the property supported
   */
  code: code;
  /** 
   * Code of the property supported
   */
  _code?: Element
  /** 
   * Operations supported for the property
   */
  op: Array<code>;
  /** 
   * Operations supported for the property
   */
  _op?: Array<Element>
}
export interface TerminologyCapabilitiesCodeSystemVersion {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Version identifier for this version
   */
  code?: string;
  /** 
   * Version identifier for this version
   */
  _code?: Element
  /** 
   * If this is the default version for this code system
   */
  isDefault?: boolean;
  /** 
   * If this is the default version for this code system
   */
  _isDefault?: Element
  /** 
   * If compositional grammar is supported
   */
  compositional?: boolean;
  /** 
   * If compositional grammar is supported
   */
  _compositional?: Element
  /** 
   * Language Displays supported
   */
  language?: Array<code>;
  /** 
   * Language Displays supported
   */
  _language?: Array<Element>
  /** 
   * Filter Properties supported
   */
  filter?: Array<TerminologyCapabilitiesCodeSystemVersionFilter>;
  /** 
   * Properties supported for $lookup
   */
  property?: Array<code>;
  /** 
   * Properties supported for $lookup
   */
  _property?: Array<Element>
}
export interface TerminologyCapabilitiesCodeSystem {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * URI for the Code System
   */
  uri?: canonical;
  /** 
   * URI for the Code System
   */
  _uri?: Element
  /** 
   * Version of Code System supported
   */
  version?: Array<TerminologyCapabilitiesCodeSystemVersion>;
  /** 
   * Whether subsumption is supported
   */
  subsumption?: boolean;
  /** 
   * Whether subsumption is supported
   */
  _subsumption?: Element
}
export interface TerminologyCapabilitiesExpansionParameter {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Expansion Parameter name
   */
  name: code;
  /** 
   * Expansion Parameter name
   */
  _name?: Element
  /** 
   * Description of support for parameter
   */
  documentation?: string;
  /** 
   * Description of support for parameter
   */
  _documentation?: Element
}
export interface TerminologyCapabilitiesExpansion {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Whether the server can return nested value sets
   */
  hierarchical?: boolean;
  /** 
   * Whether the server can return nested value sets
   */
  _hierarchical?: Element
  /** 
   * Whether the server supports paging on expansion
   */
  paging?: boolean;
  /** 
   * Whether the server supports paging on expansion
   */
  _paging?: Element
  /** 
   * Allow request for incomplete expansions?
   */
  incomplete?: boolean;
  /** 
   * Allow request for incomplete expansions?
   */
  _incomplete?: Element
  /** 
   * Supported expansion parameter
   */
  parameter?: Array<TerminologyCapabilitiesExpansionParameter>;
  /** 
   * Documentation about text searching works
   */
  textFilter?: markdown;
  /** 
   * Documentation about text searching works
   */
  _textFilter?: Element
}
export interface TerminologyCapabilitiesValidateCode {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Whether translations are validated
   */
  translations: boolean;
  /** 
   * Whether translations are validated
   */
  _translations?: Element
}
export interface TerminologyCapabilitiesTranslation {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Whether the client must identify the map
   */
  needsMap: boolean;
  /** 
   * Whether the client must identify the map
   */
  _needsMap?: Element
}
export interface TerminologyCapabilitiesClosure {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * If cross-system closure is supported
   */
  translation?: boolean;
  /** 
   * If cross-system closure is supported
   */
  _translation?: Element
}
export interface TerminologyCapabilities {
resourceType: "TerminologyCapabilities"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Canonical identifier for this terminology capabilities, represented as a URI (globally unique)
   */
  url?: uri;
  /** 
   * Canonical identifier for this terminology capabilities, represented as a URI (globally unique)
   */
  _url?: Element
  /** 
   * Business version of the terminology capabilities
   */
  version?: string;
  /** 
   * Business version of the terminology capabilities
   */
  _version?: Element
  /** 
   * Name for this terminology capabilities (computer friendly)
   */
  name?: string;
  /** 
   * Name for this terminology capabilities (computer friendly)
   */
  _name?: Element
  /** 
   * Name for this terminology capabilities (human friendly)
   */
  title?: string;
  /** 
   * Name for this terminology capabilities (human friendly)
   */
  _title?: Element
  /** 
   * draft | active | retired | unknown
   */
  status: code;
  /** 
   * draft | active | retired | unknown
   */
  _status?: Element
  /** 
   * For testing purposes, not real usage
   */
  experimental?: boolean;
  /** 
   * For testing purposes, not real usage
   */
  _experimental?: Element
  /** 
   * Date last changed
   */
  date: dateTime;
  /** 
   * Date last changed
   */
  _date?: Element
  /** 
   * Name of the publisher (organization or individual)
   */
  publisher?: string;
  /** 
   * Name of the publisher (organization or individual)
   */
  _publisher?: Element
  /** 
   * Contact details for the publisher
   */
  contact?: Array<ContactDetail>;
  /** 
   * Natural language description of the terminology capabilities
   */
  description?: markdown;
  /** 
   * Natural language description of the terminology capabilities
   */
  _description?: Element
  /** 
   * The context that the content is intended to support
   */
  useContext?: Array<UsageContext>;
  /** 
   * Intended jurisdiction for terminology capabilities (if applicable)
   */
  jurisdiction?: Array<CodeableConcept>;
  /** 
   * Why this terminology capabilities is defined
   */
  purpose?: markdown;
  /** 
   * Why this terminology capabilities is defined
   */
  _purpose?: Element
  /** 
   * Use and/or publishing restrictions
   */
  copyright?: markdown;
  /** 
   * Use and/or publishing restrictions
   */
  _copyright?: Element
  /** 
   * instance | capability | requirements
   */
  kind: code;
  /** 
   * instance | capability | requirements
   */
  _kind?: Element
  /** 
   * Software that is covered by this terminology capability statement
   */
  software?: TerminologyCapabilitiesSoftware;
  /** 
   * If this describes a specific instance
   */
  implementation?: TerminologyCapabilitiesImplementation;
  /** 
   * Whether lockedDate is supported
   */
  lockedDate?: boolean;
  /** 
   * Whether lockedDate is supported
   */
  _lockedDate?: Element
  /** 
   * A code system supported by the server
   */
  codeSystem?: Array<TerminologyCapabilitiesCodeSystem>;
  /** 
   * Information about the [ValueSet/$expand](valueset-operation-expand.html) operation
   */
  expansion?: TerminologyCapabilitiesExpansion;
  /** 
   * explicit | all
   */
  codeSearch?: code;
  /** 
   * explicit | all
   */
  _codeSearch?: Element
  /** 
   * Information about the [ValueSet/$validate-code](valueset-operation-validate-code.html) operation
   */
  validateCode?: TerminologyCapabilitiesValidateCode;
  /** 
   * Information about the [ConceptMap/$translate](conceptmap-operation-translate.html) operation
   */
  translation?: TerminologyCapabilitiesTranslation;
  /** 
   * Information about the [ConceptMap/$closure](conceptmap-operation-closure.html) operation
   */
  closure?: TerminologyCapabilitiesClosure;
}

export interface TestReportParticipant {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * test-engine | client | server
   */
  type: code;
  /** 
   * test-engine | client | server
   */
  _type?: Element
  /** 
   * The uri of the participant. An absolute URL is preferred
   */
  uri: uri;
  /** 
   * The uri of the participant. An absolute URL is preferred
   */
  _uri?: Element
  /** 
   * The display name of the participant
   */
  display?: string;
  /** 
   * The display name of the participant
   */
  _display?: Element
}
export interface TestReportSetupActionOperation {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * pass | skip | fail | warning | error
   */
  result: code;
  /** 
   * pass | skip | fail | warning | error
   */
  _result?: Element
  /** 
   * A message associated with the result
   */
  message?: markdown;
  /** 
   * A message associated with the result
   */
  _message?: Element
  /** 
   * A link to further details on the result
   */
  detail?: uri;
  /** 
   * A link to further details on the result
   */
  _detail?: Element
}
export interface TestReportSetupActionAssert {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * pass | skip | fail | warning | error
   */
  result: code;
  /** 
   * pass | skip | fail | warning | error
   */
  _result?: Element
  /** 
   * A message associated with the result
   */
  message?: markdown;
  /** 
   * A message associated with the result
   */
  _message?: Element
  /** 
   * A link to further details on the result
   */
  detail?: string;
  /** 
   * A link to further details on the result
   */
  _detail?: Element
}
export interface TestReportSetupAction {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The operation to perform
   */
  operation?: TestReportSetupActionOperation;
  /** 
   * The assertion to perform
   */
  assert?: TestReportSetupActionAssert;
}
export interface TestReportSetup {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * A setup operation or assert that was executed
   */
  action: Array<TestReportSetupAction>;
}
export interface TestReportTestAction {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The operation performed
   */
  operation?: TestReportSetupActionOperation;
  /** 
   * The assertion performed
   */
  assert?: TestReportSetupActionAssert;
}
export interface TestReportTest {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Tracking/logging name of this test
   */
  name?: string;
  /** 
   * Tracking/logging name of this test
   */
  _name?: Element
  /** 
   * Tracking/reporting short description of the test
   */
  description?: string;
  /** 
   * Tracking/reporting short description of the test
   */
  _description?: Element
  /** 
   * A test operation or assert that was performed
   */
  action: Array<TestReportTestAction>;
}
export interface TestReportTeardownAction {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The teardown operation performed
   */
  operation: TestReportSetupActionOperation;
}
export interface TestReportTeardown {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * One or more teardown operations performed
   */
  action: Array<TestReportTeardownAction>;
}
export interface TestReport {
resourceType: "TestReport"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * External identifier
   */
  identifier?: Identifier;
  /** 
   * Informal name of the executed TestScript
   */
  name?: string;
  /** 
   * Informal name of the executed TestScript
   */
  _name?: Element
  /** 
   * completed | in-progress | waiting | stopped | entered-in-error
   */
  status: code;
  /** 
   * completed | in-progress | waiting | stopped | entered-in-error
   */
  _status?: Element
  /** 
   * Reference to the  version-specific TestScript that was executed to produce this TestReport
   */
  testScript: Reference;
  /** 
   * pass | fail | pending
   */
  result: code;
  /** 
   * pass | fail | pending
   */
  _result?: Element
  /** 
   * The final score (percentage of tests passed) resulting from the execution of the TestScript
   */
  score?: decimal;
  /** 
   * The final score (percentage of tests passed) resulting from the execution of the TestScript
   */
  _score?: Element
  /** 
   * Name of the tester producing this report (Organization or individual)
   */
  tester?: string;
  /** 
   * Name of the tester producing this report (Organization or individual)
   */
  _tester?: Element
  /** 
   * When the TestScript was executed and this TestReport was generated
   */
  issued?: dateTime;
  /** 
   * When the TestScript was executed and this TestReport was generated
   */
  _issued?: Element
  /** 
   * A participant in the test execution, either the execution engine, a client, or a server
   */
  participant?: Array<TestReportParticipant>;
  /** 
   * The results of the series of required setup operations before the tests were executed
   */
  setup?: TestReportSetup;
  /** 
   * A test executed from the test script
   */
  test?: Array<TestReportTest>;
  /** 
   * The results of running the series of required clean up steps
   */
  teardown?: TestReportTeardown;
}

export interface TestScriptOrigin {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The index of the abstract origin server starting at 1
   */
  index: integer;
  /** 
   * The index of the abstract origin server starting at 1
   */
  _index?: Element
  /** 
   * FHIR-Client | FHIR-SDC-FormFiller
   */
  profile: Coding;
}
export interface TestScriptDestination {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The index of the abstract destination server starting at 1
   */
  index: integer;
  /** 
   * The index of the abstract destination server starting at 1
   */
  _index?: Element
  /** 
   * FHIR-Server | FHIR-SDC-FormManager | FHIR-SDC-FormReceiver | FHIR-SDC-FormProcessor
   */
  profile: Coding;
}
export interface TestScriptMetadataLink {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * URL to the specification
   */
  url: uri;
  /** 
   * URL to the specification
   */
  _url?: Element
  /** 
   * Short description
   */
  description?: string;
  /** 
   * Short description
   */
  _description?: Element
}
export interface TestScriptMetadataCapability {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Are the capabilities required?
   */
  required: boolean;
  /** 
   * Are the capabilities required?
   */
  _required?: Element
  /** 
   * Are the capabilities validated?
   */
  validated: boolean;
  /** 
   * Are the capabilities validated?
   */
  _validated?: Element
  /** 
   * The expected capabilities of the server
   */
  description?: string;
  /** 
   * The expected capabilities of the server
   */
  _description?: Element
  /** 
   * Which origin server these requirements apply to
   */
  origin?: Array<integer>;
  /** 
   * Which origin server these requirements apply to
   */
  _origin?: Array<Element>
  /** 
   * Which server these requirements apply to
   */
  destination?: integer;
  /** 
   * Which server these requirements apply to
   */
  _destination?: Element
  /** 
   * Links to the FHIR specification
   */
  link?: Array<uri>;
  /** 
   * Links to the FHIR specification
   */
  _link?: Array<Element>
  /** 
   * Required Capability Statement
   */
  capabilities: canonical;
  /** 
   * Required Capability Statement
   */
  _capabilities?: Element
}
export interface TestScriptMetadata {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Links to the FHIR specification
   */
  link?: Array<TestScriptMetadataLink>;
  /** 
   * Capabilities  that are assumed to function correctly on the FHIR server being tested
   */
  capability: Array<TestScriptMetadataCapability>;
}
export interface TestScriptFixture {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Whether or not to implicitly create the fixture during setup
   */
  autocreate: boolean;
  /** 
   * Whether or not to implicitly create the fixture during setup
   */
  _autocreate?: Element
  /** 
   * Whether or not to implicitly delete the fixture during teardown
   */
  autodelete: boolean;
  /** 
   * Whether or not to implicitly delete the fixture during teardown
   */
  _autodelete?: Element
  /** 
   * Reference of the resource
   */
  resource?: Reference;
}
export interface TestScriptVariable {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Descriptive name for this variable
   */
  name: string;
  /** 
   * Descriptive name for this variable
   */
  _name?: Element
  /** 
   * Default, hard-coded, or user-defined value for this variable
   */
  defaultValue?: string;
  /** 
   * Default, hard-coded, or user-defined value for this variable
   */
  _defaultValue?: Element
  /** 
   * Natural language description of the variable
   */
  description?: string;
  /** 
   * Natural language description of the variable
   */
  _description?: Element
  /** 
   * The FHIRPath expression against the fixture body
   */
  expression?: string;
  /** 
   * The FHIRPath expression against the fixture body
   */
  _expression?: Element
  /** 
   * HTTP header field name for source
   */
  headerField?: string;
  /** 
   * HTTP header field name for source
   */
  _headerField?: Element
  /** 
   * Hint help text for default value to enter
   */
  hint?: string;
  /** 
   * Hint help text for default value to enter
   */
  _hint?: Element
  /** 
   * XPath or JSONPath against the fixture body
   */
  path?: string;
  /** 
   * XPath or JSONPath against the fixture body
   */
  _path?: Element
  /** 
   * Fixture Id of source expression or headerField within this variable
   */
  sourceId?: id;
  /** 
   * Fixture Id of source expression or headerField within this variable
   */
  _sourceId?: Element
}
export interface TestScriptSetupActionOperationRequestHeader {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * HTTP header field name
   */
  field: string;
  /** 
   * HTTP header field name
   */
  _field?: Element
  /** 
   * HTTP headerfield value
   */
  value: string;
  /** 
   * HTTP headerfield value
   */
  _value?: Element
}
export interface TestScriptSetupActionOperation {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The operation code type that will be executed
   */
  type?: Coding;
  /** 
   * Resource type
   */
  resource?: code;
  /** 
   * Resource type
   */
  _resource?: Element
  /** 
   * Tracking/logging operation label
   */
  label?: string;
  /** 
   * Tracking/logging operation label
   */
  _label?: Element
  /** 
   * Tracking/reporting operation description
   */
  description?: string;
  /** 
   * Tracking/reporting operation description
   */
  _description?: Element
  /** 
   * Mime type to accept in the payload of the response, with charset etc.
   */
  accept?: code;
  /** 
   * Mime type to accept in the payload of the response, with charset etc.
   */
  _accept?: Element
  /** 
   * Mime type of the request payload contents, with charset etc.
   */
  contentType?: code;
  /** 
   * Mime type of the request payload contents, with charset etc.
   */
  _contentType?: Element
  /** 
   * Server responding to the request
   */
  destination?: integer;
  /** 
   * Server responding to the request
   */
  _destination?: Element
  /** 
   * Whether or not to send the request url in encoded format
   */
  encodeRequestUrl: boolean;
  /** 
   * Whether or not to send the request url in encoded format
   */
  _encodeRequestUrl?: Element
  /** 
   * delete | get | options | patch | post | put | head
   */
  method?: code;
  /** 
   * delete | get | options | patch | post | put | head
   */
  _method?: Element
  /** 
   * Server initiating the request
   */
  origin?: integer;
  /** 
   * Server initiating the request
   */
  _origin?: Element
  /** 
   * Explicitly defined path parameters
   */
  params?: string;
  /** 
   * Explicitly defined path parameters
   */
  _params?: Element
  /** 
   * Each operation can have one or more header elements
   */
  requestHeader?: Array<TestScriptSetupActionOperationRequestHeader>;
  /** 
   * Fixture Id of mapped request
   */
  requestId?: id;
  /** 
   * Fixture Id of mapped request
   */
  _requestId?: Element
  /** 
   * Fixture Id of mapped response
   */
  responseId?: id;
  /** 
   * Fixture Id of mapped response
   */
  _responseId?: Element
  /** 
   * Fixture Id of body for PUT and POST requests
   */
  sourceId?: id;
  /** 
   * Fixture Id of body for PUT and POST requests
   */
  _sourceId?: Element
  /** 
   * Id of fixture used for extracting the [id],  [type], and [vid] for GET requests
   */
  targetId?: id;
  /** 
   * Id of fixture used for extracting the [id],  [type], and [vid] for GET requests
   */
  _targetId?: Element
  /** 
   * Request URL
   */
  url?: string;
  /** 
   * Request URL
   */
  _url?: Element
}
export interface TestScriptSetupActionAssert {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Tracking/logging assertion label
   */
  label?: string;
  /** 
   * Tracking/logging assertion label
   */
  _label?: Element
  /** 
   * Tracking/reporting assertion description
   */
  description?: string;
  /** 
   * Tracking/reporting assertion description
   */
  _description?: Element
  /** 
   * response | request
   */
  direction?: code;
  /** 
   * response | request
   */
  _direction?: Element
  /** 
   * Id of the source fixture to be evaluated
   */
  compareToSourceId?: string;
  /** 
   * Id of the source fixture to be evaluated
   */
  _compareToSourceId?: Element
  /** 
   * The FHIRPath expression to evaluate against the source fixture
   */
  compareToSourceExpression?: string;
  /** 
   * The FHIRPath expression to evaluate against the source fixture
   */
  _compareToSourceExpression?: Element
  /** 
   * XPath or JSONPath expression to evaluate against the source fixture
   */
  compareToSourcePath?: string;
  /** 
   * XPath or JSONPath expression to evaluate against the source fixture
   */
  _compareToSourcePath?: Element
  /** 
   * Mime type to compare against the 'Content-Type' header
   */
  contentType?: code;
  /** 
   * Mime type to compare against the 'Content-Type' header
   */
  _contentType?: Element
  /** 
   * The FHIRPath expression to be evaluated
   */
  expression?: string;
  /** 
   * The FHIRPath expression to be evaluated
   */
  _expression?: Element
  /** 
   * HTTP header field name
   */
  headerField?: string;
  /** 
   * HTTP header field name
   */
  _headerField?: Element
  /** 
   * Fixture Id of minimum content resource
   */
  minimumId?: string;
  /** 
   * Fixture Id of minimum content resource
   */
  _minimumId?: Element
  /** 
   * Perform validation on navigation links?
   */
  navigationLinks?: boolean;
  /** 
   * Perform validation on navigation links?
   */
  _navigationLinks?: Element
  /** 
   * equals | notEquals | in | notIn | greaterThan | lessThan | empty | notEmpty | contains | notContains | eval
   */
  operator?: code;
  /** 
   * equals | notEquals | in | notIn | greaterThan | lessThan | empty | notEmpty | contains | notContains | eval
   */
  _operator?: Element
  /** 
   * XPath or JSONPath expression
   */
  path?: string;
  /** 
   * XPath or JSONPath expression
   */
  _path?: Element
  /** 
   * delete | get | options | patch | post | put | head
   */
  requestMethod?: code;
  /** 
   * delete | get | options | patch | post | put | head
   */
  _requestMethod?: Element
  /** 
   * Request URL comparison value
   */
  requestURL?: string;
  /** 
   * Request URL comparison value
   */
  _requestURL?: Element
  /** 
   * Resource type
   */
  resource?: code;
  /** 
   * Resource type
   */
  _resource?: Element
  /** 
   * okay | created | noContent | notModified | bad | forbidden | notFound | methodNotAllowed | conflict | gone | preconditionFailed | unprocessable
   */
  response?: code;
  /** 
   * okay | created | noContent | notModified | bad | forbidden | notFound | methodNotAllowed | conflict | gone | preconditionFailed | unprocessable
   */
  _response?: Element
  /** 
   * HTTP response code to test
   */
  responseCode?: string;
  /** 
   * HTTP response code to test
   */
  _responseCode?: Element
  /** 
   * Fixture Id of source expression or headerField
   */
  sourceId?: id;
  /** 
   * Fixture Id of source expression or headerField
   */
  _sourceId?: Element
  /** 
   * Profile Id of validation profile reference
   */
  validateProfileId?: id;
  /** 
   * Profile Id of validation profile reference
   */
  _validateProfileId?: Element
  /** 
   * The value to compare to
   */
  value?: string;
  /** 
   * The value to compare to
   */
  _value?: Element
  /** 
   * Will this assert produce a warning only on error?
   */
  warningOnly: boolean;
  /** 
   * Will this assert produce a warning only on error?
   */
  _warningOnly?: Element
}
export interface TestScriptSetupAction {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The setup operation to perform
   */
  operation?: TestScriptSetupActionOperation;
  /** 
   * The assertion to perform
   */
  assert?: TestScriptSetupActionAssert;
}
export interface TestScriptSetup {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * A setup operation or assert to perform
   */
  action: Array<TestScriptSetupAction>;
}
export interface TestScriptTestAction {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The setup operation to perform
   */
  operation?: TestScriptSetupActionOperation;
  /** 
   * The setup assertion to perform
   */
  assert?: TestScriptSetupActionAssert;
}
export interface TestScriptTest {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Tracking/logging name of this test
   */
  name?: string;
  /** 
   * Tracking/logging name of this test
   */
  _name?: Element
  /** 
   * Tracking/reporting short description of the test
   */
  description?: string;
  /** 
   * Tracking/reporting short description of the test
   */
  _description?: Element
  /** 
   * A test operation or assert to perform
   */
  action: Array<TestScriptTestAction>;
}
export interface TestScriptTeardownAction {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The teardown operation to perform
   */
  operation: TestScriptSetupActionOperation;
}
export interface TestScriptTeardown {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * One or more teardown operations to perform
   */
  action: Array<TestScriptTeardownAction>;
}
export interface TestScript {
resourceType: "TestScript"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Canonical identifier for this test script, represented as a URI (globally unique)
   */
  url: uri;
  /** 
   * Canonical identifier for this test script, represented as a URI (globally unique)
   */
  _url?: Element
  /** 
   * Additional identifier for the test script
   */
  identifier?: Identifier;
  /** 
   * Business version of the test script
   */
  version?: string;
  /** 
   * Business version of the test script
   */
  _version?: Element
  /** 
   * Name for this test script (computer friendly)
   */
  name: string;
  /** 
   * Name for this test script (computer friendly)
   */
  _name?: Element
  /** 
   * Name for this test script (human friendly)
   */
  title?: string;
  /** 
   * Name for this test script (human friendly)
   */
  _title?: Element
  /** 
   * draft | active | retired | unknown
   */
  status: code;
  /** 
   * draft | active | retired | unknown
   */
  _status?: Element
  /** 
   * For testing purposes, not real usage
   */
  experimental?: boolean;
  /** 
   * For testing purposes, not real usage
   */
  _experimental?: Element
  /** 
   * Date last changed
   */
  date?: dateTime;
  /** 
   * Date last changed
   */
  _date?: Element
  /** 
   * Name of the publisher (organization or individual)
   */
  publisher?: string;
  /** 
   * Name of the publisher (organization or individual)
   */
  _publisher?: Element
  /** 
   * Contact details for the publisher
   */
  contact?: Array<ContactDetail>;
  /** 
   * Natural language description of the test script
   */
  description?: markdown;
  /** 
   * Natural language description of the test script
   */
  _description?: Element
  /** 
   * The context that the content is intended to support
   */
  useContext?: Array<UsageContext>;
  /** 
   * Intended jurisdiction for test script (if applicable)
   */
  jurisdiction?: Array<CodeableConcept>;
  /** 
   * Why this test script is defined
   */
  purpose?: markdown;
  /** 
   * Why this test script is defined
   */
  _purpose?: Element
  /** 
   * Use and/or publishing restrictions
   */
  copyright?: markdown;
  /** 
   * Use and/or publishing restrictions
   */
  _copyright?: Element
  /** 
   * An abstract server representing a client or sender in a message exchange
   */
  origin?: Array<TestScriptOrigin>;
  /** 
   * An abstract server representing a destination or receiver in a message exchange
   */
  destination?: Array<TestScriptDestination>;
  /** 
   * Required capability that is assumed to function correctly on the FHIR server being tested
   */
  metadata?: TestScriptMetadata;
  /** 
   * Fixture in the test script - by reference (uri)
   */
  fixture?: Array<TestScriptFixture>;
  /** 
   * Reference of the validation profile
   */
  profile?: Array<Reference>;
  /** 
   * Placeholder for evaluated elements
   */
  variable?: Array<TestScriptVariable>;
  /** 
   * A series of required setup operations before tests are executed
   */
  setup?: TestScriptSetup;
  /** 
   * A test in this script
   */
  test?: Array<TestScriptTest>;
  /** 
   * A series of required clean up steps
   */
  teardown?: TestScriptTeardown;
}

export interface ValueSetComposeIncludeConceptDesignation {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Human language of the designation
   */
  language?: code;
  /** 
   * Human language of the designation
   */
  _language?: Element
  /** 
   * Types of uses of designations
   */
  use?: Coding;
  /** 
   * The text value for this designation
   */
  value: string;
  /** 
   * The text value for this designation
   */
  _value?: Element
}
export interface ValueSetComposeIncludeConcept {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Code or expression from system
   */
  code: code;
  /** 
   * Code or expression from system
   */
  _code?: Element
  /** 
   * Text to display for this code for this value set in this valueset
   */
  display?: string;
  /** 
   * Text to display for this code for this value set in this valueset
   */
  _display?: Element
  /** 
   * Additional representations for this concept
   */
  designation?: Array<ValueSetComposeIncludeConceptDesignation>;
}
export interface ValueSetComposeIncludeFilter {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * A property/filter defined by the code system
   */
  property: code;
  /** 
   * A property/filter defined by the code system
   */
  _property?: Element
  /** 
   * = | is-a | descendent-of | is-not-a | regex | in | not-in | generalizes | exists
   */
  op: code;
  /** 
   * = | is-a | descendent-of | is-not-a | regex | in | not-in | generalizes | exists
   */
  _op?: Element
  /** 
   * Code from the system, or regex criteria, or boolean value for exists
   */
  value: string;
  /** 
   * Code from the system, or regex criteria, or boolean value for exists
   */
  _value?: Element
}
export interface ValueSetComposeInclude {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The system the codes come from
   */
  system?: uri;
  /** 
   * The system the codes come from
   */
  _system?: Element
  /** 
   * Specific version of the code system referred to
   */
  version?: string;
  /** 
   * Specific version of the code system referred to
   */
  _version?: Element
  /** 
   * A concept defined in the system
   */
  concept?: Array<ValueSetComposeIncludeConcept>;
  /** 
   * Select codes/concepts by their properties (including relationships)
   */
  filter?: Array<ValueSetComposeIncludeFilter>;
  /** 
   * Select the contents included in this value set
   */
  valueSet?: Array<canonical>;
  /** 
   * Select the contents included in this value set
   */
  _valueSet?: Array<Element>
}
export interface ValueSetCompose {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Fixed date for references with no specified version (transitive)
   */
  lockedDate?: date;
  /** 
   * Fixed date for references with no specified version (transitive)
   */
  _lockedDate?: Element
  /** 
   * Whether inactive codes are in the value set
   */
  inactive?: boolean;
  /** 
   * Whether inactive codes are in the value set
   */
  _inactive?: Element
  /** 
   * Include one or more codes from a code system or other value set(s)
   */
  include: Array<ValueSetComposeInclude>;
  /** 
   * Explicitly exclude codes from a code system or other value sets
   */
  exclude?: Array<ValueSetComposeInclude>;
}
export interface ValueSetExpansionParameter {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Name as assigned by the client or server
   */
  name: string;
  /** 
   * Name as assigned by the client or server
   */
  _name?: Element
  /** 
   * Value of the named parameter
   */
  valueString?: string;
  /** 
   * Value of the named parameter
   */
  _valueString?: Element
  /** 
   * Value of the named parameter
   */
  valueBoolean?: boolean;
  /** 
   * Value of the named parameter
   */
  _valueBoolean?: Element
  /** 
   * Value of the named parameter
   */
  valueInteger?: integer;
  /** 
   * Value of the named parameter
   */
  _valueInteger?: Element
  /** 
   * Value of the named parameter
   */
  valueDecimal?: decimal;
  /** 
   * Value of the named parameter
   */
  _valueDecimal?: Element
  /** 
   * Value of the named parameter
   */
  valueUri?: uri;
  /** 
   * Value of the named parameter
   */
  _valueUri?: Element
  /** 
   * Value of the named parameter
   */
  valueCode?: code;
  /** 
   * Value of the named parameter
   */
  _valueCode?: Element
  /** 
   * Value of the named parameter
   */
  valueDateTime?: dateTime;
  /** 
   * Value of the named parameter
   */
  _valueDateTime?: Element
}
export interface ValueSetExpansionContains {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * System value for the code
   */
  system?: uri;
  /** 
   * System value for the code
   */
  _system?: Element
  /** 
   * If user cannot select this entry
   */
  abstract?: boolean;
  /** 
   * If user cannot select this entry
   */
  _abstract?: Element
  /** 
   * If concept is inactive in the code system
   */
  inactive?: boolean;
  /** 
   * If concept is inactive in the code system
   */
  _inactive?: Element
  /** 
   * Version in which this code/display is defined
   */
  version?: string;
  /** 
   * Version in which this code/display is defined
   */
  _version?: Element
  /** 
   * Code - if blank, this is not a selectable code
   */
  code?: code;
  /** 
   * Code - if blank, this is not a selectable code
   */
  _code?: Element
  /** 
   * User display for the concept
   */
  display?: string;
  /** 
   * User display for the concept
   */
  _display?: Element
  /** 
   * Additional representations for this item
   */
  designation?: Array<ValueSetComposeIncludeConceptDesignation>;
  /** 
   * Codes contained under this entry
   */
  contains?: Array<ValueSetExpansionContains>;
}
export interface ValueSetExpansion {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Identifies the value set expansion (business identifier)
   */
  identifier?: uri;
  /** 
   * Identifies the value set expansion (business identifier)
   */
  _identifier?: Element
  /** 
   * Time ValueSet expansion happened
   */
  timestamp: dateTime;
  /** 
   * Time ValueSet expansion happened
   */
  _timestamp?: Element
  /** 
   * Total number of codes in the expansion
   */
  total?: integer;
  /** 
   * Total number of codes in the expansion
   */
  _total?: Element
  /** 
   * Offset at which this resource starts
   */
  offset?: integer;
  /** 
   * Offset at which this resource starts
   */
  _offset?: Element
  /** 
   * Parameter that controlled the expansion process
   */
  parameter?: Array<ValueSetExpansionParameter>;
  /** 
   * Codes in the value set
   */
  contains?: Array<ValueSetExpansionContains>;
}
export interface ValueSet {
resourceType: "ValueSet"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Canonical identifier for this value set, represented as a URI (globally unique)
   */
  url?: uri;
  /** 
   * Canonical identifier for this value set, represented as a URI (globally unique)
   */
  _url?: Element
  /** 
   * Additional identifier for the value set (business identifier)
   */
  identifier?: Array<Identifier>;
  /** 
   * Business version of the value set
   */
  version?: string;
  /** 
   * Business version of the value set
   */
  _version?: Element
  /** 
   * Name for this value set (computer friendly)
   */
  name?: string;
  /** 
   * Name for this value set (computer friendly)
   */
  _name?: Element
  /** 
   * Name for this value set (human friendly)
   */
  title?: string;
  /** 
   * Name for this value set (human friendly)
   */
  _title?: Element
  /** 
   * draft | active | retired | unknown
   */
  status: code;
  /** 
   * draft | active | retired | unknown
   */
  _status?: Element
  /** 
   * For testing purposes, not real usage
   */
  experimental?: boolean;
  /** 
   * For testing purposes, not real usage
   */
  _experimental?: Element
  /** 
   * Date last changed
   */
  date?: dateTime;
  /** 
   * Date last changed
   */
  _date?: Element
  /** 
   * Name of the publisher (organization or individual)
   */
  publisher?: string;
  /** 
   * Name of the publisher (organization or individual)
   */
  _publisher?: Element
  /** 
   * Contact details for the publisher
   */
  contact?: Array<ContactDetail>;
  /** 
   * Natural language description of the value set
   */
  description?: markdown;
  /** 
   * Natural language description of the value set
   */
  _description?: Element
  /** 
   * The context that the content is intended to support
   */
  useContext?: Array<UsageContext>;
  /** 
   * Intended jurisdiction for value set (if applicable)
   */
  jurisdiction?: Array<CodeableConcept>;
  /** 
   * Indicates whether or not any change to the content logical definition may occur
   */
  immutable?: boolean;
  /** 
   * Indicates whether or not any change to the content logical definition may occur
   */
  _immutable?: Element
  /** 
   * Why this value set is defined
   */
  purpose?: markdown;
  /** 
   * Why this value set is defined
   */
  _purpose?: Element
  /** 
   * Use and/or publishing restrictions
   */
  copyright?: markdown;
  /** 
   * Use and/or publishing restrictions
   */
  _copyright?: Element
  /** 
   * Content logical definition of the value set (CLD)
   */
  compose?: ValueSetCompose;
  /** 
   * Used when the value set is "expanded"
   */
  expansion?: ValueSetExpansion;
}

export interface VerificationResultPrimarySource {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Reference to the primary source
   */
  who?: Reference;
  /** 
   * Type of primary source (License Board; Primary Education; Continuing Education; Postal Service; Relationship owner; Registration Authority; legal source; issuing source; authoritative source)
   */
  type?: Array<CodeableConcept>;
  /** 
   * Method for exchanging information with the primary source
   */
  communicationMethod?: Array<CodeableConcept>;
  /** 
   * successful | failed | unknown
   */
  validationStatus?: CodeableConcept;
  /** 
   * When the target was validated against the primary source
   */
  validationDate?: dateTime;
  /** 
   * When the target was validated against the primary source
   */
  _validationDate?: Element
  /** 
   * yes | no | undetermined
   */
  canPushUpdates?: CodeableConcept;
  /** 
   * specific | any | source
   */
  pushTypeAvailable?: Array<CodeableConcept>;
}
export interface VerificationResultAttestation {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * The individual or organization attesting to information
   */
  who?: Reference;
  /** 
   * When the who is asserting on behalf of another (organization or individual)
   */
  onBehalfOf?: Reference;
  /** 
   * The method by which attested information was submitted/retrieved
   */
  communicationMethod?: CodeableConcept;
  /** 
   * The date the information was attested to
   */
  date?: date;
  /** 
   * The date the information was attested to
   */
  _date?: Element
  /** 
   * A digital identity certificate associated with the attestation source
   */
  sourceIdentityCertificate?: string;
  /** 
   * A digital identity certificate associated with the attestation source
   */
  _sourceIdentityCertificate?: Element
  /** 
   * A digital identity certificate associated with the proxy entity submitting attested information on behalf of the attestation source
   */
  proxyIdentityCertificate?: string;
  /** 
   * A digital identity certificate associated with the proxy entity submitting attested information on behalf of the attestation source
   */
  _proxyIdentityCertificate?: Element
  /** 
   * Proxy signature
   */
  proxySignature?: Signature;
  /** 
   * Attester signature
   */
  sourceSignature?: Signature;
}
export interface VerificationResultValidator {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Reference to the organization validating information
   */
  organization: Reference;
  /** 
   * A digital identity certificate associated with the validator
   */
  identityCertificate?: string;
  /** 
   * A digital identity certificate associated with the validator
   */
  _identityCertificate?: Element
  /** 
   * Validator signature
   */
  attestationSignature?: Signature;
}
export interface VerificationResult {
resourceType: "VerificationResult"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * A resource that was validated
   */
  target?: Array<Reference>;
  /** 
   * The fhirpath location(s) within the resource that was validated
   */
  targetLocation?: Array<string>;
  /** 
   * The fhirpath location(s) within the resource that was validated
   */
  _targetLocation?: Array<Element>
  /** 
   * none | initial | periodic
   */
  need?: CodeableConcept;
  /** 
   * attested | validated | in-process | req-revalid | val-fail | reval-fail
   */
  status: code;
  /** 
   * attested | validated | in-process | req-revalid | val-fail | reval-fail
   */
  _status?: Element
  /** 
   * When the validation status was updated
   */
  statusDate?: dateTime;
  /** 
   * When the validation status was updated
   */
  _statusDate?: Element
  /** 
   * nothing | primary | multiple
   */
  validationType?: CodeableConcept;
  /** 
   * The primary process by which the target is validated (edit check; value set; primary source; multiple sources; standalone; in context)
   */
  validationProcess?: Array<CodeableConcept>;
  /** 
   * Frequency of revalidation
   */
  frequency?: Timing;
  /** 
   * The date/time validation was last completed (including failed validations)
   */
  lastPerformed?: dateTime;
  /** 
   * The date/time validation was last completed (including failed validations)
   */
  _lastPerformed?: Element
  /** 
   * The date when target is next validated, if appropriate
   */
  nextScheduled?: date;
  /** 
   * The date when target is next validated, if appropriate
   */
  _nextScheduled?: Element
  /** 
   * fatal | warn | rec-only | none
   */
  failureAction?: CodeableConcept;
  /** 
   * Information about the primary source(s) involved in validation
   */
  primarySource?: Array<VerificationResultPrimarySource>;
  /** 
   * Information about the entity attesting to information
   */
  attestation?: VerificationResultAttestation;
  /** 
   * Information about the entity validating information
   */
  validator?: Array<VerificationResultValidator>;
}

export interface VisionPrescriptionLensSpecificationPrism {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Amount of adjustment
   */
  amount: decimal;
  /** 
   * Amount of adjustment
   */
  _amount?: Element
  /** 
   * up | down | in | out
   */
  base: code;
  /** 
   * up | down | in | out
   */
  _base?: Element
}
export interface VisionPrescriptionLensSpecification {
  /** 
   * Unique id for inter-element referencing
   */
  id?: id;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored even if unrecognized
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Product to be supplied
   */
  product: CodeableConcept;
  /** 
   * right | left
   */
  eye: code;
  /** 
   * right | left
   */
  _eye?: Element
  /** 
   * Power of the lens
   */
  sphere?: decimal;
  /** 
   * Power of the lens
   */
  _sphere?: Element
  /** 
   * Lens power for astigmatism
   */
  cylinder?: decimal;
  /** 
   * Lens power for astigmatism
   */
  _cylinder?: Element
  /** 
   * Lens meridian which contain no power for astigmatism
   */
  axis?: integer;
  /** 
   * Lens meridian which contain no power for astigmatism
   */
  _axis?: Element
  /** 
   * Eye alignment compensation
   */
  prism?: Array<VisionPrescriptionLensSpecificationPrism>;
  /** 
   * Added power for multifocal levels
   */
  add?: decimal;
  /** 
   * Added power for multifocal levels
   */
  _add?: Element
  /** 
   * Contact lens power
   */
  power?: decimal;
  /** 
   * Contact lens power
   */
  _power?: Element
  /** 
   * Contact lens back curvature
   */
  backCurve?: decimal;
  /** 
   * Contact lens back curvature
   */
  _backCurve?: Element
  /** 
   * Contact lens diameter
   */
  diameter?: decimal;
  /** 
   * Contact lens diameter
   */
  _diameter?: Element
  /** 
   * Lens wear duration
   */
  duration?: Quantity;
  /** 
   * Color required
   */
  color?: string;
  /** 
   * Color required
   */
  _color?: Element
  /** 
   * Brand required
   */
  brand?: string;
  /** 
   * Brand required
   */
  _brand?: Element
  /** 
   * Notes for coatings
   */
  note?: Array<Annotation>;
}
export interface VisionPrescription {
resourceType: "VisionPrescription"
  /** 
   * Logical id of this artifact
   */
  id?: id;
  /** 
   * Metadata about the resource
   */
  meta?: Meta;
  /** 
   * A set of rules under which this content was created
   */
  implicitRules?: uri;
  /** 
   * A set of rules under which this content was created
   */
  _implicitRules?: Element
  /** 
   * Language of the resource content
   */
  language?: code;
  /** 
   * Language of the resource content
   */
  _language?: Element
  /** 
   * Text summary of the resource, for human interpretation
   */
  text?: Narrative;
  /** 
   * Contained, inline Resources
   */
  contained?: Array<Resource>;
  /** 
   * Additional content defined by implementations
   */
  extension?: Array<Extension>;
  /** 
   * Extensions that cannot be ignored
   */
  modifierExtension?: Array<Extension>;
  /** 
   * Business Identifier for vision prescription
   */
  identifier?: Array<Identifier>;
  /** 
   * active | cancelled | draft | entered-in-error
   */
  status: code;
  /** 
   * active | cancelled | draft | entered-in-error
   */
  _status?: Element
  /** 
   * Response creation date
   */
  created: dateTime;
  /** 
   * Response creation date
   */
  _created?: Element
  /** 
   * Who prescription is for
   */
  patient: Reference;
  /** 
   * Created during encounter / admission / stay
   */
  encounter?: Reference;
  /** 
   * When prescription was authorized
   */
  dateWritten: dateTime;
  /** 
   * When prescription was authorized
   */
  _dateWritten?: Element
  /** 
   * Who authorized the vision prescription
   */
  prescriber: Reference;
  /** 
   * Vision lens authorization
   */
  lensSpecification: Array<VisionPrescriptionLensSpecification>;
}
export type ResourceMap = {
  Account: Account;
  ActivityDefinition: ActivityDefinition;
  AdministrableProductDefinition: AdministrableProductDefinition;
  AdverseEvent: AdverseEvent;
  AllergyIntolerance: AllergyIntolerance;
  Appointment: Appointment;
  AppointmentResponse: AppointmentResponse;
  AuditEvent: AuditEvent;
  Basic: Basic;
  Binary: Binary;
  BiologicallyDerivedProduct: BiologicallyDerivedProduct;
  BodyStructure: BodyStructure;
  Bundle: Bundle;
  CapabilityStatement: CapabilityStatement;
  CarePlan: CarePlan;
  CareTeam: CareTeam;
  CatalogEntry: CatalogEntry;
  ChargeItem: ChargeItem;
  ChargeItemDefinition: ChargeItemDefinition;
  Citation: Citation;
  Claim: Claim;
  ClaimResponse: ClaimResponse;
  ClinicalImpression: ClinicalImpression;
  ClinicalUseDefinition: ClinicalUseDefinition;
  CodeSystem: CodeSystem;
  Communication: Communication;
  CommunicationRequest: CommunicationRequest;
  CompartmentDefinition: CompartmentDefinition;
  Composition: Composition;
  ConceptMap: ConceptMap;
  Condition: Condition;
  Consent: Consent;
  Contract: Contract;
  Coverage: Coverage;
  CoverageEligibilityRequest: CoverageEligibilityRequest;
  CoverageEligibilityResponse: CoverageEligibilityResponse;
  DetectedIssue: DetectedIssue;
  Device: Device;
  DeviceDefinition: DeviceDefinition;
  DeviceMetric: DeviceMetric;
  DeviceRequest: DeviceRequest;
  DeviceUseStatement: DeviceUseStatement;
  DiagnosticReport: DiagnosticReport;
  DocumentManifest: DocumentManifest;
  DocumentReference: DocumentReference;
  Encounter: Encounter;
  Endpoint: Endpoint;
  EnrollmentRequest: EnrollmentRequest;
  EnrollmentResponse: EnrollmentResponse;
  EpisodeOfCare: EpisodeOfCare;
  EventDefinition: EventDefinition;
  Evidence: Evidence;
  EvidenceReport: EvidenceReport;
  EvidenceVariable: EvidenceVariable;
  ExampleScenario: ExampleScenario;
  ExplanationOfBenefit: ExplanationOfBenefit;
  FamilyMemberHistory: FamilyMemberHistory;
  Flag: Flag;
  Goal: Goal;
  GraphDefinition: GraphDefinition;
  Group: Group;
  GuidanceResponse: GuidanceResponse;
  HealthcareService: HealthcareService;
  ImagingStudy: ImagingStudy;
  Immunization: Immunization;
  ImmunizationEvaluation: ImmunizationEvaluation;
  ImmunizationRecommendation: ImmunizationRecommendation;
  ImplementationGuide: ImplementationGuide;
  Ingredient: Ingredient;
  InsurancePlan: InsurancePlan;
  Invoice: Invoice;
  Library: Library;
  Linkage: Linkage;
  List: List;
  Location: Location;
  ManufacturedItemDefinition: ManufacturedItemDefinition;
  Measure: Measure;
  MeasureReport: MeasureReport;
  Media: Media;
  Medication: Medication;
  MedicationAdministration: MedicationAdministration;
  MedicationDispense: MedicationDispense;
  MedicationKnowledge: MedicationKnowledge;
  MedicationRequest: MedicationRequest;
  MedicationStatement: MedicationStatement;
  MedicinalProductDefinition: MedicinalProductDefinition;
  MessageDefinition: MessageDefinition;
  MessageHeader: MessageHeader;
  MolecularSequence: MolecularSequence;
  NamingSystem: NamingSystem;
  NutritionOrder: NutritionOrder;
  NutritionProduct: NutritionProduct;
  Observation: Observation;
  ObservationDefinition: ObservationDefinition;
  OperationDefinition: OperationDefinition;
  OperationOutcome: OperationOutcome;
  Organization: Organization;
  OrganizationAffiliation: OrganizationAffiliation;
  PackagedProductDefinition: PackagedProductDefinition;
  Parameters: Parameters;
  Patient: Patient;
  PaymentNotice: PaymentNotice;
  PaymentReconciliation: PaymentReconciliation;
  Person: Person;
  PlanDefinition: PlanDefinition;
  Practitioner: Practitioner;
  PractitionerRole: PractitionerRole;
  Procedure: Procedure;
  Provenance: Provenance;
  Questionnaire: Questionnaire;
  QuestionnaireResponse: QuestionnaireResponse;
  RegulatedAuthorization: RegulatedAuthorization;
  RelatedPerson: RelatedPerson;
  RequestGroup: RequestGroup;
  ResearchDefinition: ResearchDefinition;
  ResearchElementDefinition: ResearchElementDefinition;
  ResearchStudy: ResearchStudy;
  ResearchSubject: ResearchSubject;
  RiskAssessment: RiskAssessment;
  Schedule: Schedule;
  SearchParameter: SearchParameter;
  ServiceRequest: ServiceRequest;
  Slot: Slot;
  Specimen: Specimen;
  SpecimenDefinition: SpecimenDefinition;
  StructureDefinition: StructureDefinition;
  StructureMap: StructureMap;
  Subscription: Subscription;
  SubscriptionStatus: SubscriptionStatus;
  SubscriptionTopic: SubscriptionTopic;
  Substance: Substance;
  SubstanceDefinition: SubstanceDefinition;
  SupplyDelivery: SupplyDelivery;
  SupplyRequest: SupplyRequest;
  Task: Task;
  TerminologyCapabilities: TerminologyCapabilities;
  TestReport: TestReport;
  TestScript: TestScript;
  ValueSet: ValueSet;
  VerificationResult: VerificationResult;
  VisionPrescription: VisionPrescription;
}


export type ResourceType = keyof ResourceMap

export type AResource<T extends keyof ResourceMap> = ResourceMap[T];

export type ConcreteType = ResourceMap[keyof ResourceMap]

export type Resource = ConcreteType
export type DomainResource = ConcreteType

type ComplexMap = {
  Element: Element;
  BackboneElement: BackboneElement;
  Address: Address;
  Age: Age;
  Annotation: Annotation;
  Attachment: Attachment;
  CodeableConcept: CodeableConcept;
  CodeableReference: CodeableReference;
  Coding: Coding;
  ContactDetail: ContactDetail;
  ContactPoint: ContactPoint;
  Contributor: Contributor;
  Count: Count;
  DataRequirement: DataRequirement;
  Distance: Distance;
  Dosage: Dosage;
  Duration: Duration;
  ElementDefinition: ElementDefinition;
  Expression: Expression;
  Extension: Extension;
  HumanName: HumanName;
  Identifier: Identifier;
  MarketingStatus: MarketingStatus;
  Meta: Meta;
  Money: Money;
  Narrative: Narrative;
  ParameterDefinition: ParameterDefinition;
  Period: Period;
  Population: Population;
  ProdCharacteristic: ProdCharacteristic;
  ProductShelfLife: ProductShelfLife;
  Quantity: Quantity;
  Range: Range;
  Ratio: Ratio;
  RatioRange: RatioRange;
  Reference: Reference;
  RelatedArtifact: RelatedArtifact;
  SampledData: SampledData;
  Signature: Signature;
  Timing: Timing;
  TriggerDefinition: TriggerDefinition;
  UsageContext: UsageContext;
}


type ComplexTypes = keyof ComplexMap

type DataMap = ComplexMap & ResourceMap

export type DataType = keyof DataMap

export type AData<T extends DataType> = DataMap[T];