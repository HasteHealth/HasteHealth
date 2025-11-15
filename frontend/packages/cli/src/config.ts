import toml, { JsonMap } from "@iarna/toml";
import * as inquirer from "@inquirer/prompts";
import { Command } from "commander";
import fs from "node:fs";
import { homedir } from "node:os";
import path from "node:path";

import logger from "./logger.js";

export const CONFIG_LOCATION = path.join(
  homedir(),
  ".haste-health/config.toml"
);

export interface Tenant extends JsonMap {
  name: string;
  r4_url: string;
  r4b_url: string;
  oidc_discovery_uri: string;
  auth: {
    type: "public" | "client_credentials";
    client_id?: string;
    client_secret?: string;
  };
}

export interface Config extends JsonMap {
  lastUpdated: string;
  defaultTenant: Tenant["name"];
  tenants: Tenant[];
}

function initializeConfig(location: string): Config {
  logger.info("Initializing configuration file");

  const initialConfig: Config = {
    defaultTenant: "",
    lastUpdated: new Date().toISOString(),
    tenants: [],
  };

  saveConfig(location, initialConfig);
  return initialConfig;
}

export function loadConfig(location: string): Config {
  let config: Config | undefined;
  try {
    const configuration = fs.readFileSync(location, "utf8");
    config = toml.parse(configuration) as Config;
  } catch (e) {
    config = initializeConfig(location);
  }
  return config;
}

export function saveConfig(location: string, config: Config) {
  if (!fs.existsSync(path.join(location, "../"))) {
    fs.mkdirSync(path.join(location, "../"), { recursive: true });
  }
  fs.writeFileSync(location, toml.stringify(config), "utf8");
}

export function getCurrentTenant(
  location: string,
  config: Config
): Tenant | undefined {
  if (!config.defaultTenant) {
    config = {
      ...config,
      defaultTenant: config.tenants[0]?.name,
    };
    saveConfig(location, config);
  }

  return config.tenants.find((tenant) => tenant.name === config.defaultTenant);
}

export function configurationCommands(command: Command) {
  command
    .command("switch-tenant")
    .description("Switch tenant to use to add a tenant use add-tenant")
    .option("-t, --tenant <tenant>", "Tenant name")
    .action(async (options) => {
      const config = loadConfig(CONFIG_LOCATION);

      if (options.tenant) {
        const tenant = config.tenants.find((t) => t.name === options.tenant);
        if (!tenant) throw new Error("No tenant found with that name");
        saveConfig(CONFIG_LOCATION, { ...config, defaultTenant: tenant.name });
      } else {
        const defaultTenant = await inquirer.select({
          message: "Switch tenant to use",
          choices: config.tenants.map((t) => ({
            name: t.name,
            value: t.name,
          })),
        });

        saveConfig(CONFIG_LOCATION, { ...config, defaultTenant });
      }
    });

  command
    .command("delete-tenant")
    .description("Delete a tenant")
    .requiredOption("-t, --tenant <tenant>", "Tenant name")
    .action(async (options) => {
      const config = loadConfig(CONFIG_LOCATION);
      const tenants = config.tenants.filter((t) => t.name !== options.tenant);

      saveConfig(CONFIG_LOCATION, {
        ...config,
        defaultTenant:
          options.tenant === config.defaultTenant ? "" : config.defaultTenant,
        tenants,
      });
    });

  command
    .command("show-tenant")
    .description("Display the current tenant.")
    .action(async () => {
      const config = loadConfig(CONFIG_LOCATION);
      const currentTenant = getCurrentTenant(CONFIG_LOCATION, config);
      if (currentTenant) {
        logger.info(currentTenant);
      } else {
        logger.warn("No tenant configured");
      }
    });

  command
    .command("add-tenant")
    .description("Add a tenant to the configuration file")
    .option("-r4, --r4-url <r4Url>", "r4 URL")
    .option("-r4b, --r4b-url <r4bUrl>", "r4b URL")
    .option("-i, --oidc <oidcDiscovery>", "Enter the OIDC discovery URL")
    .option("-a, --auth <authMethod>", "Enter auth method")
    .option("-n, --name <name>", "Tenant name")
    .option("--client-id <clientId>", "Client id")
    .option("--client-secret <clientSecret>", "Client secret")
    .action(async (options) => {
      const config = loadConfig(CONFIG_LOCATION);

      const name = options.name
        ? options.name
        : await inquirer.input({
            message: "Enter tenant name:",
            validate: async (value) => {
              const existingTenant = config.tenants.find(
                (t) => t.name === value
              );
              return existingTenant !== undefined
                ? `Tenant with name '${value}' already exists. Must be unique.`
                : true;
            },
          });

      const r4Url = options.r4Url
        ? options.r4Url
        : await inquirer.input({
            message: "Enter the R4 URL",
          });
      const r4bUrl = options.r4bUrl
        ? options.r4bUrl
        : await inquirer.input({
            message: "Enter an R4B URL. ",
          });

      const authMethod = options.auth
        ? options.auth
        : await inquirer.select({
            message: "Select an auth method",
            choices: [
              { name: "public", value: "public" },
              { name: "client_credentials", value: "client_credentials" },
            ],
          });

      let oidcDiscovery = "";
      let client_id = "";
      let client_secret = "";

      if (authMethod === "client_credentials") {
        oidcDiscovery = options.oidc
          ? options.oidc
          : await inquirer.input({
              message: "Enter the url for discovery. ",
            });
        client_id = options.clientId
          ? options.clientId
          : await inquirer.input({
              message: "Enter client id:",
            });
        client_secret = options.clientSecret
          ? options.clientSecret
          : await inquirer.password({
              message: "Enter client secret:",
            });
      }

      config.tenants.push({
        r4_url: r4Url,
        r4b_url: r4bUrl,
        oidc_discovery_uri: oidcDiscovery,
        name,
        auth: {
          type: authMethod,
          client_id,
          client_secret,
        },
      });
      saveConfig(CONFIG_LOCATION, config);
    });
}
