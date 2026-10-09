// Audit Autofix's steps (audit_autofix.yml):
//   node autofix.mjs plan                     pick the fix jobs, unless the audit's pull request is open
//   node autofix.mjs fix <audit> [image]      apply the tooling's fixes, then report
//   node autofix.mjs report <audit> [image]   audit again: summary.md, and remaining.md if anything is left
//   node autofix.mjs quota <audit> [image]    whether Claude may run, within its limits
//   node autofix.mjs claude <audit> [image]   let Claude fix what is left
//   node autofix.mjs publish <audit>          open the pull request
// A fix job keeps its files in $RUNNER_TEMP/autofix; publish reads each job's
// from $RUNNER_TEMP/results.
import { spawn } from "node:child_process";
import {
  appendFileSync,
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  renameSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { audits, cmd, output, root, run } from "./audits.mjs";

const { RUNNER_TEMP, GITHUB_REPOSITORY, GITHUB_SERVER_URL, GITHUB_RUN_ID } =
  process.env;
const out = join(RUNNER_TEMP, "autofix");
// The Claude step's name in audit_autofix.yml, which quota counts.
const claudeStep = "Fix the rest with Claude";
// Where audit_autofix.yml installs Claude Code.
const claudeBin = join(
  RUNNER_TEMP,
  "claude-code/node_modules/@anthropic-ai/claude-code-linux-x64/claude",
);

const [command, name, image = ""] = process.argv.slice(2);
const audit = audits[name];
const session = image || name;

const commands = {
  // Outputs the failed audit and its fix jobs, unless its pull request is open.
  plan() {
    const [failed, { images = [""] }] = Object.entries(audits).find(
      ([, a]) => a.workflow === process.env.WORKFLOW,
    );
    const pullRequests = JSON.parse(
      output(
        cmd`gh pr list --repo ${GITHUB_REPOSITORY} --head autofix/${failed} --json url,isCrossRepository`,
      ),
    );
    // A fork's pull request can use the same branch name.
    const open = pullRequests.find((pr) => !pr.isCrossRepository);
    if (open)
      return console.log(
        `::notice::${open.url} is still open for the ${failed} audit.`,
      );
    setOutput("audit", failed);
    setOutput(
      "jobs",
      JSON.stringify(images.map((image) => ({ audit: failed, image }))),
    );
  },

  async fix() {
    mkdirSync(out, { recursive: true });
    const findings = await audit.audit(image);
    writeFileSync(join(out, "before.json"), JSON.stringify(findings));
    if (findings.length) await audit.fix(findings, image);
    await commands.report();
  },

  async report() {
    const before = JSON.parse(readFileSync(join(out, "before.json"), "utf8"));
    const label = image ? `the ${image} image` : `the ${name} audit`;
    rmSync(join(out, "remaining.md"), { force: true });
    if (!before.length)
      return writeMarkdown("summary.md", [`No findings in ${label}.`]);

    const { findings, notes = [] } = audit.check
      ? await audit.check(image)
      : { findings: await audit.audit(image) };
    writeMarkdown("summary.md", [
      `### Findings in ${label}`,
      table(before),
      ...notes,
    ]);
    if (findings.length)
      writeMarkdown("remaining.md", [
        "### Still found after the fixes",
        table(findings),
      ]);
  },

  // Allows Claude when findings are left, it can sign in, fewer than
  // CLAUDE_DAILY_SESSIONS sessions started in the last 24 hours, and this job
  // had none in that time (manual runs skip that last check).
  quota() {
    if (!existsSync(join(out, "remaining.md")))
      return setOutput("claude", "false");
    const deny = (reason) => {
      console.log(`::notice::No Claude session: ${reason}`);
      writeMarkdown("claude.md", ["### Claude", `Not run: ${reason}`]);
      setOutput("claude", "false");
    };

    if (
      process.env.HAS_CLAUDE_TOKEN !== "true" &&
      !process.env.ANTHROPIC_FEDERATION_RULE_ID
    ) {
      return deny(
        "there is no CLAUDE_CODE_OAUTH_TOKEN secret or ANTHROPIC_* variables to sign in with.",
      );
    }
    const started = claudeSessionsSince(Date.now() - 24 * 60 * 60 * 1000);
    const limit = Number(process.env.CLAUDE_DAILY_SESSIONS);
    if (started.length >= limit)
      return deny(
        `${started.length} of the ${limit} daily sessions have started.`,
      );
    if (
      process.env.GITHUB_EVENT_NAME !== "workflow_dispatch" &&
      started.includes(`${claudeStep} (${session})`)
    ) {
      return deny(
        `${session} had a session in the last 24 hours. Run Audit Autofix by hand to retry sooner.`,
      );
    }
    setOutput("claude", "true");
  },

  // Signs in with CLAUDE_CODE_OAUTH_TOKEN (a Claude subscription) when set,
  // otherwise with workload identity federation.
  async claude() {
    const env = { ...process.env };
    // These would take precedence over either sign-in.
    delete env.ANTHROPIC_API_KEY;
    delete env.ANTHROPIC_AUTH_TOKEN;
    delete env.ANTHROPIC_PROFILE;
    let refresher;
    if (env.CLAUDE_CODE_OAUTH_TOKEN) {
      delete env.ANTHROPIC_FEDERATION_RULE_ID;
    } else {
      delete env.CLAUDE_CODE_OAUTH_TOKEN;
      // Claude Code exchanges this GitHub OIDC token itself and re-reads the
      // file when it refreshes. GitHub's tokens last about five minutes.
      const tokenFile = (env.ANTHROPIC_IDENTITY_TOKEN_FILE = join(
        RUNNER_TEMP,
        "oidc-token",
      ));
      await writeOidcToken(tokenFile);
      refresher = setInterval(
        () => writeOidcToken(tokenFile).catch(console.error),
        2 * 60 * 1000,
      );
    }

    const { commands: allowed, verify, ignores } = audit.claude;
    const subject = image
      ? `${name} audit on main (${image} image)`
      : `${name} audit on main`;
    const ignoreRule = ignores
      ? `Only add an ignore (${ignores}) if the vulnerable code can't be reached, with a comment saying why.`
      : "";
    const prompt = `The ${subject} still finds these after the automatic fixes:

${readFileSync(join(out, "remaining.md"), "utf8")}
Fix them in this checkout.
- Make the smallest change: a patched version within the current requirement, then a newer requirement, then an upgrade of the dependency that pulls it in. Make the code changes a breaking upgrade needs.
- If the fix is only on an upstream branch, use a git dependency pinned to a rev (through [patch.crates-io] if other crates need it). Never vendor code.
- Leave what you can't fix and say why. ${ignoreRule}
- Don't add tests, commit or push.
- ${verify.replace("<image>", image)} Fix anything that fails.

End with a Markdown summary for the pull request under "### Claude": what you changed and why, what you left and why, any ignores you added, and what you ran to verify.`;

    const tools = [
      "Read",
      "Edit",
      "Write",
      "Glob",
      "Grep",
      ...["cd", "ls", "git diff", "git status", ...allowed].map(
        (c) => `Bash(${c}:*)`,
      ),
    ];
    const args = [
      "--print",
      prompt,
      "--model",
      env.CLAUDE_MODEL,
      "--effort",
      env.CLAUDE_EFFORT,
      "--max-budget-usd",
      env.CLAUDE_BUDGET_USD,
      "--permission-mode",
      "dontAsk",
      "--no-session-persistence",
      // On Linux Claude Code looks for CLAUDE.md, not the repository's Claude.md.
      "--append-system-prompt-file",
      "Claude.md",
      "--allowedTools",
      ...tools,
    ];
    const claude = spawn(claudeBin, args, {
      cwd: root,
      env,
      stdio: ["ignore", "pipe", "inherit"],
    });
    let summary = "";
    claude.stdout.on("data", (chunk) => (summary += chunk));
    const code = await new Promise((resolve) => claude.on("close", resolve));
    clearInterval(refresher);

    // The repository is public: a token in Claude's summary or changes would
    // be published. Discard them instead.
    const token = env.CLAUDE_CODE_OAUTH_TOKEN;
    run(cmd`git add --all`);
    const changes = output(cmd`git diff --cached --text`);
    run(cmd`git reset --quiet`);
    if (token && (summary.includes(token) || changes.includes(token))) {
      run(cmd`git reset --quiet --hard`);
      run(cmd`git clean --force -d --quiet`);
      writeMarkdown("claude.md", [
        "### Claude",
        "Its output contained the Claude token, so it was discarded along with all changes.",
      ]);
      process.exitCode = 1;
      return;
    }

    if (code === 0) writeFileSync(join(out, "claude.md"), summary);
    else
      writeMarkdown("claude.md", [
        "### Claude",
        "Stopped with an error; see the run log. Its changes are included.",
      ]);
    process.exitCode = code ?? 1;
  },

  // Opens one pull request with every fix job's changes. A closed pull request
  // with the same changes stays closed.
  publish() {
    const results = join(RUNNER_TEMP, "results");
    const jobs = readdirSync(results).map((dir) => join(results, dir));
    const read = (file) =>
      existsSync(file) ? readFileSync(file, "utf8").trim() : "";
    const branch = `autofix/${name}`;
    const title = `Fix ${name} audit findings`;
    const runUrl = `${GITHUB_SERVER_URL}/${GITHUB_REPOSITORY}/actions/runs/${GITHUB_RUN_ID}`;
    const body = [
      `Fixes for the failing ${name} audit on main, by [Audit Autofix](${runUrl}). It leaves this audit alone until this pull request is merged or closed.`,
      ...jobs.flatMap((job) =>
        ["summary.md", "claude.md", "remaining.md"].map((file) =>
          read(join(job, file)),
        ),
      ),
      audit.checks.length
        ? `Checks run on this branch: ${audit.checks.join(", ")}.`
        : "",
    ]
      .filter(Boolean)
      .join("\n\n");
    appendFileSync(process.env.GITHUB_STEP_SUMMARY, `# ${title}\n\n${body}\n`);

    run(cmd`git checkout --quiet -b ${branch}`);
    jobs.forEach(applyChanges);

    if (!output(cmd`git diff --cached --name-only`).trim()) {
      console.log("No changes to propose.");
    } else {
      run(cmd`git -c user.name=github-actions[bot] -c user.email=41898282+github-actions[bot]@users.noreply.github.com
        commit --quiet --message ${title}`);
      const proposed = output(
        cmd`git ls-remote origin refs/heads/${branch}`,
      ).trim();
      if (proposed) run(cmd`git fetch --quiet --depth=2 origin ${branch}`);
      if (proposed && patchId("FETCH_HEAD") === patchId("HEAD")) {
        console.log(
          `::notice::These changes were proposed on ${branch} before, and that pull request was closed.`,
        );
      } else {
        run(cmd`git push --quiet --force origin ${branch}`);
        console.log(
          output(
            cmd`gh pr create --head ${branch} --title ${title} --body-file -`,
            { input: body },
          ),
        );
        // Pull requests opened with GITHUB_TOKEN don't trigger workflows.
        for (const workflow of audit.checks)
          run(cmd`gh workflow run ${workflow} --ref ${branch}`);
      }
    }

    if (jobs.some((job) => existsSync(join(job, "remaining.md")))) {
      console.log("::error::Some findings need a person; see the run summary.");
      process.exitCode = 1;
    }
  },
};

// Stages a fix job's changes, only within the audit's paths: the fix jobs ran
// untrusted code.
function applyChanges(job) {
  const patch = join(job, "changes.patch");
  if (!statSync(patch).size) return;
  const only = (flag) => audit.paths.map((path) => `--${flag}=${path}/*`);
  const outside = output([
    "git",
    "apply",
    "--numstat",
    ...only("exclude"),
    patch,
  ]).trim();
  if (outside)
    console.log(
      `::warning::Leaving out changes outside ${audit.paths.join(", ")}:\n${outside}`,
    );
  run(["git", "apply", "--index", ...only("include"), patch]);
}

// Names of the Claude steps that started (running or finished) since `time`.
function claudeSessionsSince(time) {
  const lines = (argv) => output(argv).split("\n").filter(Boolean);
  const since = new Date(time).toISOString();
  const runIds =
    lines(cmd`gh api --method GET --paginate repos/${GITHUB_REPOSITORY}/actions/workflows/audit_autofix.yml/runs
    -f created=>=${since} --jq .workflow_runs[].id`);
  return runIds
    .flatMap((id) =>
      lines(
        cmd`gh api --paginate repos/${GITHUB_REPOSITORY}/actions/runs/${id}/jobs --jq ${".jobs[].steps[]? | tojson"}`,
      ),
    )
    .map((line) => JSON.parse(line))
    .filter(
      (step) =>
        step.name.startsWith(claudeStep) &&
        ["in_progress", "completed"].includes(step.status) &&
        step.conclusion !== "skipped",
    )
    .map((step) => step.name);
}

async function writeOidcToken(file) {
  const {
    ACTIONS_ID_TOKEN_REQUEST_URL: url,
    ACTIONS_ID_TOKEN_REQUEST_TOKEN: token,
  } = process.env;
  const response = await fetch(`${url}&audience=https://api.anthropic.com`, {
    headers: { Authorization: `Bearer ${token}` },
  });
  if (!response.ok)
    throw new Error(`GitHub OIDC token request failed: ${response.status}`);
  writeFileSync(`${file}.new`, (await response.json()).value);
  renameSync(`${file}.new`, file);
}

const patchId = (rev) =>
  output(cmd`git patch-id --stable`, {
    input: output(cmd`git diff ${rev}^ ${rev}`),
  }).split(" ")[0];

const setOutput = (key, value) =>
  appendFileSync(process.env.GITHUB_OUTPUT, `${key}=${value}\n`);

const writeMarkdown = (file, sections) =>
  writeFileSync(join(out, file), `${sections.join("\n\n")}\n`);

function table(findings) {
  const cell = (text) => String(text ?? "").replaceAll("|", String.raw`\|`);
  const rows = findings.map(
    (f) =>
      `| [${cell(f.id)}](${f.url}) | ${[f.package, f.installed, f.fixed, f.title].map(cell).join(" | ")} |`,
  );
  return [
    "| Finding | Package | Installed | Fixed in | Title |",
    "| --- | --- | --- | --- | --- |",
    ...rows,
  ].join("\n");
}

if (!commands[command])
  throw new Error(
    `Unknown command '${command}'. See the top of ${import.meta.filename}.`,
  );
await commands[command]();
