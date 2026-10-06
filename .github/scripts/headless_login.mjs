// `haste-health login` without a browser: submits the server's sign-in and
// scope approval forms, then the CLI's loopback listener receives the code and
// stores the tokens for the active profile.
import { spawn } from "node:child_process";
import { once } from "node:events";
import { createInterface } from "node:readline";

// A sign-in takes a handful of requests; more means a redirect loop.
const MAX_REQUESTS = 20;
const TIMEOUT_MS = 60_000;

// The entities the server's HTML templates escape.
const ENTITIES = { amp: "&", lt: "<", gt: ">", quot: '"', "#39": "'" };

export async function headlessLogin(cliPath, email, password) {
  const cli = spawn(cliPath, ["login", "--no-browser"], {
    stdio: ["ignore", "pipe", "inherit"],
  });
  const exited = once(cli, "exit");
  const timeout = setTimeout(() => cli.kill(), TIMEOUT_MS);

  try {
    let authorizeUrl;
    for await (const line of createInterface({ input: cli.stdout })) {
      console.log(line);
      authorizeUrl = line.match(/https?:\/\/\S+/)?.[0];
      if (authorizeUrl) break;
    }
    if (!authorizeUrl) {
      throw new Error("`haste-health login` printed no authorization URL.");
    }

    cli.stdout.pipe(process.stdout);
    await submitForms(authorizeUrl, email, password);

    const [code] = await exited;
    if (code !== 0) {
      throw new Error(`\`haste-health login\` exited with ${code}.`);
    }
  } catch (error) {
    cli.kill();
    throw error;
  } finally {
    clearTimeout(timeout);
  }
}

// Follows the authorization flow from `authorizeUrl` until it redirects to the
// CLI's redirect URI, submitting each page's form as a browser would.
async function submitForms(authorizeUrl, email, password) {
  const redirectUri = new URL(authorizeUrl).searchParams.get("redirect_uri");
  if (!redirectUri) throw new Error(`No redirect_uri in ${authorizeUrl}`);

  // By hand: a cookie jar won't return the `Secure` session cookie over http.
  const cookies = new Map();
  const session = { redirectUri, email, password, cookies, signedIn: false };
  await follow(session, { url: new URL(authorizeUrl), method: "GET" }, 0);
}

// Sends `request`, then whatever it leads to, until the code reaches the CLI.
async function follow(session, request, sent) {
  if (sent === MAX_REQUESTS) {
    throw new Error(`No authorization code after ${MAX_REQUESTS} requests.`);
  }
  const response = await send(session.cookies, request);
  const location = response.headers.get("location");
  const next = location
    ? await followRedirect(session, new URL(location, request.url))
    : await submitForm(session, request.url, response);
  if (next) await follow(session, next, sent + 1);
}

async function send(cookies, { url, method, body }) {
  const headers = {
    cookie: [...cookies].map(([name, value]) => `${name}=${value}`).join("; "),
  };
  // Without fetch's `;charset=UTF-8`, which the server rejects.
  if (body) headers["content-type"] = "application/x-www-form-urlencoded";

  const response = await fetch(url, {
    method,
    body,
    headers,
    redirect: "manual",
  });
  for (const cookie of response.headers.getSetCookie()) {
    const [name, value] = cookie.split(";")[0].split(/=(.*)/);
    cookies.set(name.trim(), value.trim());
  }
  return response;
}

// The next request, or none once the redirect has handed the code to the CLI.
async function followRedirect(session, url) {
  if (!url.href.startsWith(session.redirectUri)) return { url, method: "GET" };

  const error = url.searchParams.get("error");
  if (error) throw new Error(`Authorization failed: ${error}`);
  await fetch(url);
  return undefined;
}

// Submits the page's form, filling in the credentials on the sign-in form.
async function submitForm(session, url, response) {
  const body = await response.text();
  if (!response.ok) {
    throw new Error(
      `${url} answered ${response.status}: ${body.slice(0, 500)}`,
    );
  }
  const form = firstForm(body);
  if (!form) throw new Error(`No form at ${url}: ${body.slice(0, 500)}`);

  if (form.fields.has("password")) {
    // The sign-in form again means the credentials were rejected.
    if (session.signedIn) {
      throw new Error(`Sign-in as ${session.email} was rejected.`);
    }
    form.fields.set("email", session.email);
    form.fields.set("password", session.password);
    session.signedIn = true;
  }
  return { url: new URL(form.action, url), method: "POST", body: form.fields };
}

// The page's first form and the fields a browser would submit: the sign-in
// form, or the scope approval's accept form.
function firstForm(html) {
  const form = html.match(/<form\b([^>]*)>([\s\S]*?)<\/form>/i);
  if (!form) return undefined;

  const fields = new URLSearchParams();
  for (const [, tag] of form[2].matchAll(/<input\b([^>]*)>/gi)) {
    const input = parseAttributes(tag);
    if (!input.name) continue;
    if (input.type === "checkbox") {
      if ("checked" in input) fields.append(input.name, input.value || "on");
    } else {
      fields.append(input.name, input.value ?? "");
    }
  }

  return { action: parseAttributes(form[1]).action ?? "", fields };
}

// Attributes as the server's templates write them: `name` or `name="value"`.
function parseAttributes(tag) {
  const attributes = {};
  for (const [, name, value = ""] of tag.matchAll(
    /([^\s=/>]+)(?:="([^"]*)")?/g,
  )) {
    attributes[name.toLowerCase()] = value.replace(
      /&(amp|lt|gt|quot|#39);/g,
      (_, entity) => ENTITIES[entity],
    );
  }
  return attributes;
}
