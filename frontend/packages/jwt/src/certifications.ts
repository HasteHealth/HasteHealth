import * as jose from "jose";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

import { ALGORITHMS_ALLOWED } from "./constants.js";

interface BaseOption {
  alg: ALGORITHMS_ALLOWED;
}

interface JWTCertificationFileConfig extends BaseOption {
  type: "file";
  directory: string;
  kid: string;
}

interface JWTCertificationEnvironmentConfig extends BaseOption {
  type: "environment";
  kid: string;
  privateKey: string;
  publicKey: string;
}

/**
 * Generates a keypair using the provided algorithm.
 * @param alg
 * @returns privateKey and publicKey generated using jose library.
 */
async function generateKeyPair(
  alg: ALGORITHMS_ALLOWED,
  options: { extractable: boolean },
) {
  const { privateKey, publicKey } = await jose.generateKeyPair(alg, options);
  return { privateKey, publicKey };
}

async function writeToFile(
  directory: string,
  kid: string,
  publicKey: CryptoKey,
  privateKey: CryptoKey,
) {
  const p8PublicKey = await jose.exportSPKI(publicKey);
  const p8Private = await jose.exportPKCS8(privateKey);

  if (!existsSync(directory)) {
    mkdirSync(directory, { recursive: true });
  }

  writeFileSync(path.join(directory, `${kid}.spki`), p8PublicKey);
  writeFileSync(path.join(directory, `${kid}.p8`), p8Private);
}

async function getPublicPrivateKey(option: JWTCertificationConfig): Promise<{
  publicKey: CryptoKey;
  privateKey: CryptoKey;
}> {
  switch (option.type) {
    case "environment": {
      const publicKey = await jose.importSPKI(option.publicKey, option.alg);
      const privateKey = await jose.importPKCS8(option.privateKey, option.alg);
      return { publicKey, privateKey };
    }
    case "file": {
      const publicKey = await jose.importSPKI(
        readFileSync(
          path.join(option.directory, `${option.kid}.spki`),
          "utf-8",
        ),
        option.alg,
      );
      const privateKey = await jose.importPKCS8(
        readFileSync(path.join(option.directory, `${option.kid}.p8`), "utf-8"),
        option.alg,
      );

      return { privateKey, publicKey };
    }
  }
}

/**
 *
 * @param directory
 * @param alg
 * @returns JSON Web Key Set with all public keys in the directory. Uses filename to distinguish between keys (sets the kid field in the JWK).
 */
export async function getJWKS(option: JWTCertificationConfig) {
  const { publicKey } = await getPublicPrivateKey(option);

  const pubJWK = await jose.exportJWK(publicKey);

  return {
    keys: [{ ...pubJWK, alg: option.alg, use: "sig", kid: option.kid }],
  };
}

export type JWTCertificationConfig =
  | JWTCertificationFileConfig
  | JWTCertificationEnvironmentConfig;

/**
 *
 * @param directory
 * @param kid
 * @param alg
 * @returns Returns the private key for a given kid.
 */
export async function getSigningKey(
  option: JWTCertificationConfig,
): Promise<{ key: CryptoKey; kid: string }> {
  const { privateKey } = await getPublicPrivateKey(option);
  return {
    key: privateKey,
    kid: option.kid,
  };
}

/**
 * Create certifications if not exist. Saves by default
 * private keys under /${directory}/{kid}.p8 and public keys under /${directory}/{kid}.spki.
 * Should only be used in development environment as these should be injected in environment in prod.
 * @param directory
 * @param kid
 * @param alg
 */
export async function createCertsIfNoneExists(option: JWTCertificationConfig) {
  try {
    await getSigningKey(option);
  } catch {
    if (option.type === "environment")
      throw new Error(`Cannot create certs for 'environment' type.`);

    const { publicKey, privateKey } = await generateKeyPair(option.alg, {
      extractable: true,
    });
    await writeToFile(option.directory, option.kid, publicKey, privateKey);
  }
}
