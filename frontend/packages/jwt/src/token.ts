import * as jose from "jose";

import { ALGORITHMS, JWT } from "./types.js";

export function parseJwt<A, Token extends JWT<A>>(
  token: Token,
): Token["payload"] {
  const base64Url = token.split(".")[1];
  const base64 = base64Url.replace(/-/g, "+").replace(/_/g, "/");
  const jsonPayload = decodeURIComponent(
    window
      .atob(base64)
      .split("")
      .map(function (c) {
        return "%" + ("00" + c.charCodeAt(0).toString(16)).slice(-2);
      })
      .join(""),
  );

  return JSON.parse(jsonPayload);
}

/**
 * Creates a JWT token using the provided private key and payload.
 */
export async function createToken<Payload extends jose.JWTPayload>({
  signingKey,
  payload,
  expiresIn = "1h",
}: {
  signingKey: { kid: string; key: CryptoKey };
  payload: Payload;
  expiresIn?: string;
}): Promise<JWT<Payload>> {
  const signedJWT = (await new jose.SignJWT(payload)
    .setProtectedHeader({ kid: signingKey.kid, alg: ALGORITHMS.RS384 })
    .setIssuedAt()
    .setExpirationTime(expiresIn)
    .sign(signingKey.key)) as JWT<Payload>;

  return signedJWT;
}
