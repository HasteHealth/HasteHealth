import * as jose from "jose";

import {
  Address,
  ClientApplication,
  Membership,
  OperationDefinition,
  Reference,
  canonical,
  id,
} from "@oxidized-health/fhir-types/r4/types";

import { CUSTOM_CLAIMS } from "./constants.js";

export * from "./constants.js";

declare const __tenant: unique symbol;
export type TenantId = string & { [__tenant]: string };
export interface TenantClaim<role> {
  id: TenantId;
  userRole: role;
}

declare const __project: unique symbol;
export type ProjectId = string & {
  [__project]: string;
};

declare const __subject: unique symbol;
export type Subject = string & { [__subject]: string };
declare const __iss: unique symbol;
export type Issuer = string & { [__iss]: string };

export type TOKEN_RESOURCE_TYPES =
  | OperationDefinition["resourceType"]
  | ClientApplication["resourceType"]
  | Membership["resourceType"];

export interface OxidizedHealthCustomClaims<role> {
  /**
   * Token can be associated with an Operation, Client or Membership this claim distinguishes between the three.
   */
  [CUSTOM_CLAIMS.RESOURCE_TYPE]: TOKEN_RESOURCE_TYPES;
  /**
   * The ID of the resource the token is associated with.
   */
  [CUSTOM_CLAIMS.RESOURCE_ID]: id;
  [CUSTOM_CLAIMS.ACCESS_POLICY_VERSION_IDS]: id[];
  /**
   * The users role for the tenant.
   */
  [CUSTOM_CLAIMS.ROLE]: role;
  /**
   * The tenant the token is associated with.
   */
  [CUSTOM_CLAIMS.TENANT]: TenantId;
}

export interface SMARTPayload {
  fhirUser?: canonical;
  patient?: id;
  encounter?: id;
  fhirContext?: Reference[];
}

export interface AccessTokenPayload<role>
  extends OxidizedHealthCustomClaims<role>,
    SMARTPayload,
    jose.JWTPayload {
  /**
   * REQUIRED. Issuer Identifier for the Issuer of the response.
   * The iss value is a case-sensitive URL using the https scheme that contains scheme, host, and optionally,
   *  port number and path components and no query or fragment components.
   */
  iss: Issuer;
  /**
   * sub REQUIRED. Subject Identifier.
   * A locally unique and never reassigned identifier within the Issuer for the End-User,
   * which is intended to be consumed by the Client, e.g., 24400320 or AItOawmwtWwcT0k51BayewNvutrJUqsvl6qs7A4.
   * It MUST NOT exceed 255 ASCII [RFC20] characters in length. The sub value is a case-sensitive string.
   */
  sub: Subject;
  /**
   * REQUIRED. Audience(s) that this ID Token is intended for.
   * It MUST contain the OAuth 2.0 client_id of the Relying Party as an audience value.
   * It MAY also contain identifiers for other audiences. In the general case, the aud value is an array of case-sensitive strings.
   * In the common special case when there is one audience, the aud value MAY be a single case-sensitive string.
   */
  aud: string;
  /**
   * scope Required. OAuth 2.0 scopes. Space-separated string.
   */
  scope: string;
}

/**
 * ID Token Payload. Extends Access token with additional claims from https://openid.net/specs/openid-connect-core-1_0.html#StandardClaims.
 */
export interface IDTokenPayload<role> extends AccessTokenPayload<role> {
  /**
   * End-User's full name in displayable form including all name parts, possibly including titles and suffixes, ordered according to the End-User's locale and preferences.
   */
  name?: string;
  /**
   *  Given name(s) or first name(s) of the End-User. Note that in some cultures, people can have multiple given names; all can be present, with the names being separated by space characters.
   */
  given_name?: string;
  /**
   * Surname(s) or last name(s) of the End-User. Note that in some cultures, people can have multiple family names or no family name; all can be present, with the names being separated by space characters.
   */
  family_name?: string;
  /**
   * Middle name(s) of the End-User. Note that in some cultures, people can have multiple middle names; all can be present, with the names being separated by space characters. Also note that in some cultures, middle names are not used.
   */
  middle_name?: string;
  /**
   * Casual name of the End-User that may or may not be the same as the given_name. For instance, a nickname value of Mike might be returned alongside a given_name value of Michael.
   */
  nickname?: string;
  /**
   * Shorthand name by which the End-User wishes to be referred to at the RP, such as janedoe or j.doe. This value MAY be any valid JSON string including special characters such as @, /, or whitespace. The RP MUST NOT rely upon this value being unique, as discussed in Section 5.7.
   */
  preferred_username?: string;
  /**
   * URL of the End-User's profile page. The contents of this Web page SHOULD be about the End-User.
   */
  profile?: string;
  /**
   * URL of the End-User's profile picture. This URL MUST refer to an image file (for example, a PNG, JPEG, or GIF image file), rather than to a Web page containing an image. Note that this URL SHOULD specifically reference a profile photo of the End-User suitable for displaying when describing the End-User, rather than an arbitrary photo taken by the End-User.
   */
  picture?: string;
  /**
   * URL of the End-User's Web page or blog. This Web page SHOULD contain information published by the End-User or an organization that the End-User is affiliated with.
   */
  website?: string;
  /**
   * End-User's preferred e-mail address. Its value MUST conform to the RFC 5322 [RFC5322] addr-spec syntax. The RP MUST NOT rely upon this value being unique, as discussed in Section 5.7.
   */
  email?: string;
  /**
   * True if the End-User's e-mail address has been verified; otherwise false. When this Claim Value is true, this means that the OP took affirmative steps to ensure that this e-mail address was controlled by the End-User at the time the verification was performed. The means by which an e-mail address is verified is context specific, and dependent upon the trust framework or contractual agreements within which the parties are operating.
   */
  email_verified?: boolean;
  /**
   * End-User's gender. Values defined by this specification are female and male. Other values MAY be used when neither of the defined values are applicable.
   */
  gender?: string;
  /**
   * End-User's birthday, represented as an ISO 8601-1 [ISO8601‑1] YYYY-MM-DD format. The year MAY be 0000, indicating that it is omitted. To represent only the year, YYYY format is allowed. Note that depending on the underlying platform's date related function, providing just year can result in varying month and day, so the implementers need to take this factor into account to correctly process the dates.
   */
  birthdate?: string;
  /**
   * String from IANA Time Zone Database [IANA.time‑zones] representing the End-User's time zone. For example, Europe/Paris or America/Los_Angeles.
   */
  zoneinfo?: string;
  /**
   * End-User's locale, represented as a BCP47 [RFC5646] language tag. This is typically an ISO 639 Alpha-2 [ISO639] language code in lowercase and an ISO 3166-1 Alpha-2 [ISO3166‑1] country code in uppercase, separated by a dash. For example, en-US or fr-CA. As a compatibility note, some implementations have used an underscore as the separator rather than a dash, for example, en_US; Relying Parties MAY choose to accept this locale syntax as well.
   */
  locale?: string;
  /**
   * End-User's preferred telephone number. E.164 [E.164] is RECOMMENDED as the format of this Claim, for example, +1 (425) 555-1212 or +56 (2) 687 2400. If the phone number contains an extension, it is RECOMMENDED that the extension be represented using the RFC 3966 [RFC3966] extension syntax, for example, +1 (604) 555-1234;ext=5678.
   */
  phone_number?: string;
  /**
   * True if the End-User's phone number has been verified; otherwise false. When this Claim Value is true, this means that the OP took affirmative steps to ensure that this phone number was controlled by the End-User at the time the verification was performed. The means by which a phone number is verified is context specific, and dependent upon the trust framework or contractual agreements within which the parties are operating. When true, the phone_number Claim MUST be in E.164 format and any extensions MUST be represented in RFC 3966 format.
   */
  phone_number_verified?: boolean;
  /**
   * JSON object	End-User's preferred postal address. The value of the address member is a JSON [RFC8259] structure containing some or all of the members defined in Section 5.1.1.
   */
  address?: Address;
  /**
   * Time the End-User's information was last updated. Its value is a JSON number representing the number of seconds from 1970-01-01T00:00:00Z as measured in UTC until the date/time.
   */
  updated_at?: number;
}

export type JWT<Payload> = string & { ["payload"]: Payload };
export type AccessToken<role> = JWT<AccessTokenPayload<role>>;
export type IDToken<role> = JWT<IDTokenPayload<role>>;
