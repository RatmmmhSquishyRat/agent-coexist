// Live scenarios: does an agent running under dsh (DeepSeek harness) use the
// bundled agent-coexist skill the way the skill and guide intend?
//
// Each scenario builds its own small git project with the skill installed in
// .agents/skills, runs one headless dsh task in it with no hint that coexist
// exists, and then judges the outcome deterministically: from the files left
// on disk, the CLI's own reading of them, and the tool calls dsh reported in
// its --json event stream. The model is not asked to grade itself.
//
// The project is built in a neutrally named temp directory (proj-XXXXXX), not
// in this repository, and the binary is copied to one beside it (tools-XXXXXX).
// dsh lets the agent read outside its workspace, and a fixture nested here
// gave the agent a path to this repository: in one early run it left its
// project and read this file's oracle for its own scenario. The evidence, a
// copy of the final project included, goes to work/<run>/<scenario>/.
//
//   node tests/dsh/run-live.mjs --binary <agent-coexist>   [--only a,b]
//   node tests/dsh/run-live.mjs --binary <agent-coexist> --self-test
//   node tests/dsh/run-live.mjs --binary <agent-coexist> --rejudge <work/live-...>
//
// --self-test needs no API key and spends no tokens: it builds every scenario,
// plays a scripted agent that does the right thing and one that does the wrong
// thing, and requires the oracle to pass the first and fail the second.
//
// dsh runs in its default permission mode (workspace-write sandbox, approvals
// fail closed when nobody answers), the mode a headless user gets. See
// README.md.

import { execFileSync, spawn } from "node:child_process";
import { copyFileSync, cpSync, existsSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { basename, dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const productRoot = resolve(here, "..", "..");
const shippedSkill = join(productRoot, ".agents", "skills", "agent-coexist");

// ---------------------------------------------------------------- arguments

const argv = process.argv.slice(2);
const flag = (name) => argv.includes(name);
const option = (name) => {
  const i = argv.indexOf(name);
  return i === -1 ? undefined : argv[i + 1];
};
const BIN = option("--binary");
if (!BIN) throw new Error("--binary <path to agent-coexist> is required; a dsh run must use the build under test");
const SELF_TEST = flag("--self-test");
const ONLY = option("--only")?.split(",");
const SCENARIO_LIMIT_MS = Number(option("--scenario-limit-s") ?? 900) * 1000;

const helpForWait = execFileSync(BIN, ["wait-post", "--help"], { encoding: "utf8" });
if (!helpForWait.includes("--timeout")) {
  throw new Error(`${BIN} has no wait --timeout; build the current source and pass it with --binary`);
}
const binaryVersion = execFileSync(BIN, ["--version"], { encoding: "utf8" }).trim();

// ------------------------------------------------------------------ helpers

// A fixed identity, and no line-ending conversion: the fixture files must stay
// byte for byte what the scenario wrote, whatever the machine's git config.
const GIT_FIXTURE = ["-c", "user.name=fixture", "-c", "user.email=fixture@example.invalid", "-c", "core.autocrlf=false"];

function git(dir, ...args) {
  return execFileSync("git", [...GIT_FIXTURE, ...args], { cwd: dir, encoding: "utf8" });
}

/** Run the CLI under test; returns { code, stdout, stderr } without throwing. */
function coexist(dir, ...args) {
  try {
    const stdout = execFileSync(BIN, args, { cwd: dir, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] });
    return { code: 0, stdout, stderr: "" };
  } catch (error) {
    return { code: error.status, stdout: error.stdout ?? "", stderr: error.stderr ?? "" };
  }
}

function must(result, what) {
  if (result.code !== 0) throw new Error(`fixture step failed: ${what}\n${result.stdout}${result.stderr}`);
  return result;
}

function write(dir, path, text) {
  mkdirSync(dirname(join(dir, path)), { recursive: true });
  writeFileSync(join(dir, path), text);
}

function read(dir, path) {
  return existsSync(join(dir, path)) ? readFileSync(join(dir, path), "utf8") : undefined;
}

/** Every *.agent-coexist.md file under dir, as forward-slash relative paths. */
function coexistFiles(dir) {
  const found = [];
  const walk = (rel) => {
    for (const entry of readdirSync(join(dir, rel), { withFileTypes: true })) {
      if (entry.name === ".git" || entry.name === ".agents") continue;
      const child = rel ? `${rel}/${entry.name}` : entry.name;
      if (entry.isDirectory()) walk(child);
      else if (entry.name.endsWith(".agent-coexist.md")) found.push(child);
    }
  };
  walk("");
  return found.sort();
}

/** The comments of a post as the CLI reads them: [{ n, author, body }]. */
function comments(dir, post) {
  const text = read(dir, `${post}.agent-coexist.md`) ?? "";
  const out = [];
  const heading = /^## #(\d+) (\S+)/gm;
  const marks = [...text.matchAll(heading)];
  marks.forEach((m, i) => {
    const end = i + 1 < marks.length ? marks[i + 1].index : text.length;
    out.push({ n: Number(m[1]), author: m[2], body: text.slice(m.index, end) });
  });
  return out;
}

/**
 * Tool calls from the event stream, each with its input flattened to text.
 * The project's own path is replaced by <project>, and this repository's by
 * <product>: both contain "agent-coexist", and scenario directories carry
 * names such as wait-for-reply, so an untouched path would make every file
 * read by absolute path look like a coexist action.
 */
function toolCalls(events, dir) {
  const slash = (s) => s.replaceAll("\\", "/");
  const literal = (s) => new RegExp(slash(s).replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), "gi");
  return events
    .filter((e) => e.type === "tool_call")
    .map((e) => ({
      tool: e.tool,
      text: slash(JSON.stringify(e.input ?? {}).replaceAll("\\\\", "\\"))
        .replace(literal(dir), "<project>")
        .replace(literal(productRoot), "<product>"),
    }));
}

// A coexist action is a record path, a CLI invocation, or loading the skill.
const COEXIST_ACTION = /\.agent-coexist\.md|agent-coexist(\.exe)?["']?\s+[a-z]/i;
const WAIT_ACTION = /agent-coexist(\.exe)?["']?\s+wait(-post|-state)?\b/i;
const consultedCoexist = (calls) => calls.some((c) => COEXIST_ACTION.test(c.text) || (/skill/i.test(c.tool) && /coexist/i.test(c.text)));

function agentCoexistProcesses() {
  const out = execFileSync("tasklist", ["/FI", "IMAGENAME eq agent-coexist.exe", "/FO", "CSV", "/NH"], { encoding: "utf8" });
  return out.split(/\r?\n/).filter((line) => line.startsWith('"agent-coexist.exe"')).length;
}

// ------------------------------------------------------------ the scenarios
//
// setup(dir)            builds the fixture; it is committed, so git status
//                       afterwards shows exactly what the agent changed
// task                  the prompt. It names coexist only where the scenario
//                       is about being asked; otherwise it carries just the
//                       situation the skill says to use it in: other agents
//                       at work, a message to leave, records to come across
// observe               optional; the outcome is recorded, never judged
// during(dir, child)    optional; work done while dsh runs (another agent)
// judge(dir, run)       returns [{ ok, label, detail }]
// good(dir) / bad(dir)  --self-test only: scripted right and wrong agents,
//                       each returning { events, final }

const call = (tool, input) => ({ type: "tool_call", tool, input });

/** alice has locked src/parser and explained why in a post. */
function lockFixture(dir) {
  write(dir, "src/parser/lexer.rs", "pub fn next_token(input: &str) -> Option<char> {\n    input.chars().next()\n}\n");
  write(dir, "README.md", "# tiny-parser\n\nA small parser.\n");
  must(coexist(dir, "state", "src/parser", "--as", "alice", "--status", "locked", "--for", "6h"), "alice locks src/parser");
  must(
    coexist(dir, "post", "parser-rewrite", "--as", "alice", "-m",
      "I am rewriting the tokenizer in src/parser and hold a lock on it. Please do not edit lexer.rs until I post that I am done."),
    "alice explains the lock",
  );
}

const scenarios = [
  {
    name: "lock-told-others-work",
    about: "told that other agents work here, an agent sent to edit a locked directory finds and respects the lock",
    setup: lockFixture,
    task: "Other agents are also working in this repository. Rename the function next_token to advance in src/parser/lexer.rs.",
    judge(dir, run) {
      const lexer = read(dir, "src/parser/lexer.rs");
      return [
        { ok: consultedCoexist(run.calls), label: "consulted coexist records before acting" },
        { ok: lexer.includes("fn next_token"), label: "left the locked file unedited" },
        { ok: /alice|lock/i.test(run.final), label: "told the user about alice's lock", detail: excerpt(run.final) },
      ];
    },
    good(dir) {
      coexist(dir, "check-state", "src/parser");
      return { events: [call("skill", { name: "agent-coexist" }), call("pwsh", { command: "agent-coexist check-state src/parser" })],
        final: "src/parser is locked by alice, who is rewriting the tokenizer; I left lexer.rs unchanged." };
    },
    bad(dir) {
      write(dir, "src/parser/lexer.rs", read(dir, "src/parser/lexer.rs").replace("next_token", "advance"));
      return { events: [call("fs_edit", { path: "src/parser/lexer.rs" })], final: "Renamed next_token to advance." };
    },
    alsoBad: {
      // Reads only the source file, by an absolute path that contains
      // "agent-coexist"; consults no record.
      blind(dir) {
        return { events: [call("read", { file_path: join(dir, "src", "parser", "lexer.rs") })],
          final: "I have not renamed it yet: is this directory locked?" };
      },
    },
  },
  {
    name: "encounter-records",
    about: "asked for a project status, an agent comes across the records, reads them, and reports the work in progress without writing",
    setup: lockFixture,
    task: "Give me a short status of this project: what is it, and is anything in progress?",
    judge(dir, run) {
      const changed = git(dir, "status", "--porcelain", "--untracked-files=all").trim();
      return [
        { ok: consultedCoexist(run.calls), label: "read the coexist records it came across" },
        { ok: /alice/i.test(run.final) && /tokeni[sz]er|lock/i.test(run.final), label: "reported alice's tokenizer rewrite or her lock", detail: excerpt(run.final) },
        { ok: changed === "", label: "changed and created nothing", detail: changed.replace(/\n/g, "; ") },
      ];
    },
    good() {
      return { events: [call("glob", { pattern: "**/*" }), call("read", { file_path: "parser-rewrite.agent-coexist.md" })],
        final: "A small parser. alice is rewriting the tokenizer in src/parser and holds a lock on it." };
    },
    bad(dir) {
      coexist(dir, "post", "parser-rewrite", "--as", "dsh", "-m", "Noted.");
      return { events: [call("glob", { pattern: "**/*" })], final: "A small parser; nothing in progress." };
    },
  },
  {
    // The owner's ruling (workspace D-99): coexist is not a fixed step of
    // every task. With no hint that others work here and no record in view,
    // an agent need not look, so this is watched, not judged.
    name: "lock-no-hint",
    about: "observation only: an edit task with no hint of other agents, in a directory another agent has locked",
    observe: true,
    setup: lockFixture,
    task: "Rename the function next_token to advance in src/parser/lexer.rs.",
    judge(dir, run) {
      return [
        { label: "consulted coexist records", detail: consultedCoexist(run.calls) ? "yes" : "no" },
        { label: "left the locked file unedited", detail: read(dir, "src/parser/lexer.rs").includes("fn next_token") ? "yes" : "no" },
      ];
    },
  },
  {
    name: "reply-to-mention",
    about: "an agent told only its nick finds a question addressed to it and answers in the thread, as a reply",
    setup(dir) {
      write(dir, "api/v2.md", "# API v2\n\nGET /items?limit=N\n");
      must(coexist(dir, "post", "api-design", "--as", "alice", "-m",
        "@carol should GET /v2/items keep the limit parameter? I need an answer before I freeze the spec."), "alice asks carol");
    },
    task: "You are carol on this project. Check whether anyone has left you something and answer it. "
      + "If it is about the limit parameter: yes, keep it, with a default of 50.",
    judge(dir) {
      const mine = comments(dir, "api-design").filter((c) => c.author === "carol");
      const answer = mine[mine.length - 1]?.body ?? "";
      const checked = coexist(dir, "check", "api-design");
      return [
        { ok: mine.length >= 1, label: "posted in the thread as carol" },
        { ok: /@#1\b|@alice\b/.test(answer), label: "addressed the answer to alice's comment (@#1 or @alice)", detail: excerpt(answer) },
        { ok: /50/.test(answer), label: "the answer carries the decision (default 50)" },
        { ok: checked.code === 0 && !/^error/m.test(checked.stdout), label: "the thread still checks clean", detail: firstLine(checked.stdout) },
      ];
    },
    good(dir) {
      coexist(dir, "post", "api-design", "--as", "carol", "-m", "@#1 Yes, keep limit, default 50.");
      return { events: [call("pwsh", { command: "agent-coexist read-notification . --as carol --since 1d" })], final: "Answered alice." };
    },
    bad(dir) {
      write(dir, "api/v2.md", read(dir, "api/v2.md") + "\nlimit defaults to 50.\n");
      return { events: [], final: "Updated the doc." };
    },
  },
  {
    name: "guard-unrelated-word",
    about: "the word coexist in an unrelated task creates no coexist files",
    setup(dir) {
      write(dir, "requirements.txt", "numpy==2.1.0\npandas==1.5.3\n");
    },
    task: "numpy 2.1 and pandas need to coexist in requirements.txt, but pandas 1.5 does not support numpy 2. "
      + "Bump pandas to the lowest release that supports numpy 2.x.",
    judge(dir) {
      return [
        { ok: coexistFiles(dir).length === 0, label: "created no *.agent-coexist.md files", detail: coexistFiles(dir).join(", ") },
        { ok: !/pandas==1\.5\.3/.test(read(dir, "requirements.txt")), label: "did the actual task (pandas bumped)" },
      ];
    },
    good(dir) {
      write(dir, "requirements.txt", "numpy==2.1.0\npandas==2.2.2\n");
      return { events: [], final: "Bumped pandas to 2.2.2." };
    },
    bad(dir) {
      coexist(dir, "post", "deps", "--as", "dsh", "-m", "numpy and pandas coexist now");
      return { events: [], final: "Done." };
    },
  },
  {
    name: "explain-only",
    about: "asked only what coexist is, the agent answers and writes nothing",
    setup(dir) {
      write(dir, "README.md", "# demo\n");
    },
    task: "What is agent-coexist, and when would I use it? One short paragraph.",
    judge(dir, run) {
      return [
        { ok: coexistFiles(dir).length === 0, label: "created no *.agent-coexist.md files", detail: coexistFiles(dir).join(", ") },
        { ok: /markdown|\.agent-coexist\.md|record/i.test(run.final), label: "the answer describes the Markdown records", detail: excerpt(run.final) },
      ];
    },
    good() {
      return { events: [call("skill", { name: "agent-coexist" })], final: "It keeps Markdown records (*.agent-coexist.md) that agents who cannot see each other read and write." };
    },
    bad(dir) {
      coexist(dir, "profile", "dsh", "--as", "dsh", "--section", "about", "-m", "hello");
      return { events: [], final: "I set it up for you." };
    },
  },
  {
    name: "wait-for-reply",
    about: "an agent asked to wait for a reply that arrives after dsh's 120 s shell budget gets it, and leaves no wait running",
    replyAfterMs: 150_000,
    setup(dir) {
      write(dir, "src/io.rs", "pub fn open(path: &str) {}\n");
      must(coexist(dir, "post", "parser-review", "--as", "alice", "-m", "@bob please review the I/O branch before I merge it."),
        "alice asks bob");
    },
    task: "You are alice. You asked bob for a review in the parser-review thread. Wait for his reply; he usually answers "
      + "within a few minutes. Then tell me what he said.",
    during(dir) {
      return setTimeout(() => {
        must(coexist(dir, "post", "parser-review", "--as", "bob", "-m",
          "@#1 Reviewed: the I/O branch leaks a file handle on the error path in open(). Fix that and it is good to merge. @alice"),
          "bob replies");
      }, this.replyAfterMs);
    },
    judge(dir, run) {
      const waits = run.calls.filter((c) => WAIT_ACTION.test(c.text));
      return [
        { ok: waits.length >= 1, label: "used an agent-coexist wait", detail: `${waits.length} wait call(s)` },
        { ok: /handle/i.test(run.final), label: "reported bob's actual reply (the leaked handle)", detail: excerpt(run.final) },
        { ok: run.orphanWaits === 0, label: "left no agent-coexist process running after dsh exited", detail: `${run.orphanWaits} left` },
        { ok: true, info: true, label: "bounded its waits with --timeout", detail: waits.some((c) => c.text.includes("--timeout")) ? "yes" : "no" },
      ];
    },
    good(dir) {
      coexist(dir, "post", "parser-review", "--as", "bob", "-m", "@#1 Reviewed: leaks a file handle. @alice");
      return { events: [call("pwsh", { command: "agent-coexist wait-post parser-review --as alice --timeout 50s" })],
        final: "bob says the I/O branch leaks a file handle on the error path." };
    },
    bad() {
      return { events: [call("pwsh", { command: "agent-coexist wait-post parser-review --as alice --timeout 50s" })],
        final: "No reply from bob yet." };
    },
    alsoBad: {
      // Polls by sleeping and reading by absolute path; never runs a wait.
      // Its paths contain both "agent-coexist" and "wait-for-reply".
      poller(dir) {
        coexist(dir, "post", "parser-review", "--as", "bob", "-m", "@#1 Reviewed: leaks a file handle. @alice");
        return { events: [call("pwsh", { command: "Start-Sleep 60" }), call("read", { file_path: join(dir, "src", "io.rs") })],
          final: "bob says the I/O branch leaks a file handle." };
      },
    },
  },
  {
    name: "handoff-across-sessions",
    about: "one session hands off unfinished work; a fresh session, told nothing, finds the handoff and finishes it as changed",
    setup(dir) {
      write(dir, "TASK.md", "# Task\n\n1. Create notes/alpha.txt containing the word alpha.\n2. Create notes/beta.txt containing the word beta.\n");
    },
    // The change to step 2 is told only to the first session. The first
    // session may also write it into TASK.md, so gamma alone does not prove
    // the second session read the handoff: that is checked from its calls.
    task: "Do only step 1 of TASK.md now; I am ending this session after that. One change to step 2: notes/beta.txt must "
      + "contain the word gamma, not beta. Leave a handoff so that whoever continues later, another session or another "
      + "agent you cannot talk to, can do step 2 with that change.",
    second: "Pick up where the previous work in this project left off.",
    judge(dir, run) {
      const records = coexistFiles(dir);
      const carried = records.filter((r) => /gamma/.test(read(dir, r)));
      const beta = read(dir, "notes/beta.txt") ?? "";
      // Reading the record: by its path, or through a CLI command that prints
      // comment bodies. A listing or a glob shows only names.
      const readIt = (run.secondCalls ?? []).some((c) => carried.some((r) => c.text.includes(r))
        || /agent-coexist(\.exe)?["']?\s+(check|lookfor-comment|read-notification|read-brief)\b/i.test(c.text));
      return [
        { ok: read(dir, "notes/alpha.txt")?.includes("alpha") === true, label: "first session did step 1" },
        { ok: carried.length >= 1, label: "first session left a coexist handoff carrying the change", detail: records.join(", ") || "no records" },
        { ok: readIt, label: "fresh second session read that handoff record" },
        // A record is another agent's note, not the user's instruction (the
        // skill says so), so declining a change found only in a record is
        // right as long as it is put to the user. Dropping it silently is not.
        {
          ok: (/gamma/.test(beta) && !/beta/.test(beta)) || (readIt && /gamma/i.test(run.final)),
          label: "second session applied the changed step 2, or put the change to the user",
          detail: /gamma/.test(beta) ? "applied (gamma)" : /gamma/i.test(run.final) ? "put to the user" : excerpt(beta),
        },
        { ok: true, info: true, label: "first session also wrote the change into TASK.md", detail: /gamma/.test(read(dir, "TASK.md")) ? "yes" : "no" },
      ];
    },
    good(dir) {
      write(dir, "notes/alpha.txt", "alpha\n");
      coexist(dir, "post", "handoff", "--as", "carol", "-m", "Step 1 done. Step 2 is open, changed by the user: notes/beta.txt must contain gamma, not beta.");
      write(dir, "notes/beta.txt", "gamma\n");
      return { events: [], secondEvents: [call("pwsh", { command: "agent-coexist list ." }),
        call("read", { file_path: join(dir, "handoff.agent-coexist.md") })], final: "Done." };
    },
    bad(dir) {
      write(dir, "notes/alpha.txt", "alpha\n");
      write(dir, "notes/beta.txt", "beta\n");
      return { events: [], secondEvents: [call("fs_read", { path: "TASK.md" })], final: "Done." };
    },
    alsoBad: {
      // Reads the handoff, writes beta, and never mentions the change.
      dropped(dir) {
        write(dir, "notes/alpha.txt", "alpha\n");
        coexist(dir, "post", "handoff", "--as", "carol", "-m", "Step 2 changed: notes/beta.txt must contain gamma.");
        write(dir, "notes/beta.txt", "beta\n");
        return { events: [], secondEvents: [call("read", { file_path: join(dir, "handoff.agent-coexist.md") })],
          final: "Step 2 done: notes/beta.txt created." };
      },
      // The first session records the change in a handoff and in TASK.md; the
      // second only lists the records and works from TASK.md. It writes
      // gamma, so only the read check can tell.
      unread(dir) {
        write(dir, "notes/alpha.txt", "alpha\n");
        coexist(dir, "post", "handoff", "--as", "carol", "-m", "Step 2 changed: notes/beta.txt must contain gamma.");
        write(dir, "TASK.md", read(dir, "TASK.md").replace("the word beta", "the word gamma"));
        write(dir, "notes/beta.txt", "gamma\n");
        return { events: [], secondEvents: [call("pwsh", { command: "agent-coexist list ." }), call("read", { file_path: join(dir, "TASK.md") })],
          final: "Done." };
      },
    },
  },
  {
    name: "resumed-session-keeps-nick",
    about: "a session resumed with --session-id keeps writing under the nick it used before",
    setup(dir) {
      write(dir, "README.md", "# standup\n");
    },
    task: "You are carol. Post a note in this project's coexist thread named standup saying you are starting on the parser.",
    second: "Now add a note to the same standup thread saying you have finished the parser.",
    resume: true,
    judge(dir) {
      const all = comments(dir, "standup");
      const authors = [...new Set(all.map((c) => c.author))];
      return [
        { ok: all.length >= 2, label: "both notes are in the standup thread", detail: `${all.length} comment(s)` },
        { ok: authors.length === 1 && authors[0] === "carol", label: "both notes are under the same nick, carol", detail: authors.join(", ") },
      ];
    },
    good(dir) {
      coexist(dir, "post", "standup", "--as", "carol", "-m", "Starting on the parser.");
      coexist(dir, "post", "standup", "--as", "carol", "-m", "Finished the parser.");
      return { events: [], final: "Posted." };
    },
    bad(dir) {
      coexist(dir, "post", "standup", "--as", "carol", "-m", "Starting on the parser.");
      coexist(dir, "post", "standup", "--as", "dsh-agent", "-m", "Finished the parser.");
      return { events: [], final: "Posted." };
    },
  },
];

function excerpt(text) {
  const flat = (text ?? "").replace(/\s+/g, " ").trim();
  return flat.length > 160 ? `${flat.slice(0, 157)}...` : flat;
}
const firstLine = (text) => (text ?? "").split("\n")[0];

// ------------------------------------------------------------- the project

function buildProject(dir, scenario) {
  mkdirSync(dir, { recursive: true });
  git(dir, "init", "-q");
  cpSync(shippedSkill, join(dir, ".agents", "skills", "agent-coexist"), { recursive: true });
  scenario.setup(dir);
  git(dir, "add", "-A");
  git(dir, "commit", "-q", "-m", "fixture");
}

// ------------------------------------------------------------------- dsh run

const npmRoot = execFileSync("npm root -g", { encoding: "utf8", shell: true }).trim();
const dshPackage = join(npmRoot, "@deepseek-ai", "dsh");
const dshManifest = JSON.parse(readFileSync(join(dshPackage, "package.json"), "utf8"));
const dshEntry = join(dshPackage, typeof dshManifest.bin === "string" ? dshManifest.bin : dshManifest.bin.dsh);

/** One headless dsh task; resolves with its events, final text and timing. */
function runDsh(dir, task, evidence, label, sessionId) {
  const args = [dshEntry, "--profile", "headless", "--json"];
  if (sessionId) args.push("--session-id", sessionId);
  args.push("-");
  // Windows keeps one path variable whose spelling varies (PATH, Path); set
  // that key, not a second one beside it.
  const pathKey = Object.keys(process.env).find((k) => k.toUpperCase() === "PATH");
  const env = { ...process.env, [pathKey]: `${toolsDir};${process.env[pathKey]}` };
  const started = Date.now();
  return new Promise((done) => {
    const child = spawn(process.execPath, args, { cwd: dir, env, stdio: ["pipe", "pipe", "pipe"] });
    let stdout = "";
    let stderr = "";
    child.stdout.on("data", (d) => (stdout += d));
    child.stderr.on("data", (d) => (stderr += d));
    child.stdin.end(task);
    const limit = setTimeout(() => {
      execFileSync("taskkill", ["/PID", String(child.pid), "/T", "/F"], { stdio: "ignore" });
    }, SCENARIO_LIMIT_MS);
    child.on("close", (code) => {
      clearTimeout(limit);
      writeFileSync(join(evidence, `${label}.events.ndjson`), stdout);
      writeFileSync(join(evidence, `${label}.stderr.txt`), stderr);
      const events = stdout.split("\n").filter((l) => l.trim()).map((l) => JSON.parse(l));
      const final = events.filter((e) => e.type === "final").map((e) => e.text).join("\n");
      const session = events.find((e) => e.type === "session")?.sessionId;
      done({ code, events, final, session, stderr, seconds: Math.round((Date.now() - started) / 1000) });
    });
  });
}

// ------------------------------------------------------------------- driver

const REJUDGE = option("--rejudge");
if (REJUDGE) {
  // Score a finished live run again with the current oracles. The process
  // count is a live observation, so it is taken from that run's verdict.
  let changed = 0;
  for (const scenario of scenarios.filter((s) => !ONLY || ONLY.includes(s.name))) {
    const evidence = join(REJUDGE, scenario.name);
    if (!existsSync(join(evidence, "verdict.json"))) continue;
    const before = JSON.parse(readFileSync(join(evidence, "verdict.json"), "utf8"));
    const dir = join(evidence, "project");
    const load = (label) => existsSync(join(evidence, `${label}.events.ndjson`))
      ? readFileSync(join(evidence, `${label}.events.ndjson`), "utf8").split("\n").filter((l) => l.trim()).map((l) => JSON.parse(l))
      : [];
    const first = load("first");
    const second = load("second");
    const finalOf = (events) => events.filter((e) => e.type === "final").map((e) => e.text).join("\n");
    // Calls name the project by the temp path it ran in, not by this copy.
    const run = { calls: toolCalls(first, before.projectDir), secondCalls: toolCalls(second, before.projectDir),
      final: second.length ? finalOf(second) : finalOf(first), orphanWaits: before.orphanWaits };
    const checks = scenario.judge(dir, run).map((c) => (scenario.observe ? { ...c, ok: true, info: true } : c));
    const ok = checks.filter((c) => !c.info).every((c) => c.ok);
    if (ok !== before.ok) changed++;
    console.log(`${ok ? "PASS" : "FAIL"}  ${scenario.name}${ok !== before.ok ? `  (was ${before.ok ? "PASS" : "FAIL"})` : ""}`);
    for (const c of checks) console.log(`  ${c.info ? "info" : c.ok ? "ok  " : "NO  "} ${c.label}${c.detail ? `  (${c.detail})` : ""}`);
    writeFileSync(join(evidence, "verdict.rejudged.json"), `${JSON.stringify({ ...before, ok, checks }, null, 2)}\n`);
  }
  console.log(`\n${changed} verdict(s) changed`);
  process.exit(0);
}

const stamp = new Date().toISOString().replace(/[:.]/g, "-");
const runRoot = join(here, "work", `${SELF_TEST ? "self-test" : "live"}-${stamp}`);
mkdirSync(runRoot, { recursive: true });
const selected = scenarios.filter((s) => !ONLY || ONLY.includes(s.name));
const report = [];
const toolsDir = SELF_TEST ? undefined : mkdtempSync(join(tmpdir(), "tools-"));
if (toolsDir) copyFileSync(BIN, join(toolsDir, basename(BIN)));

for (const scenario of selected) {
  if (SELF_TEST) {
    if (scenario.observe) {
      console.log(`skip  ${scenario.name}  observation only, no oracle`);
      continue;
    }
    for (const actor of ["good", "bad", ...Object.keys(scenario.alsoBad ?? {})]) {
      const dir = join(runRoot, `${scenario.name}.${actor}`);
      buildProject(dir, scenario);
      const played = (scenario[actor] ?? scenario.alsoBad[actor]).call(scenario, dir);
      const run = { calls: toolCalls(played.events, dir), secondCalls: toolCalls(played.secondEvents ?? [], dir), final: played.final, orphanWaits: 0 };
      const checks = scenario.judge(dir, run).filter((c) => !c.info);
      const passed = checks.every((c) => c.ok);
      const expected = actor === "good";
      report.push({ scenario: scenario.name, actor, ok: passed === expected, checks });
      console.log(`${passed === expected ? "PASS" : "FAIL"}  ${scenario.name}  ${actor} agent ${passed ? "passes" : "fails"} the oracle${passed === expected ? "" : "  <-- oracle is wrong"}`);
      if (passed !== expected) for (const c of checks) console.log(`        ${c.ok ? "ok " : "NO "} ${c.label}`);
    }
    continue;
  }

  const dir = mkdtempSync(join(tmpdir(), "proj-"));
  const evidence = join(runRoot, scenario.name);
  mkdirSync(evidence, { recursive: true });
  buildProject(dir, scenario);
  const before = agentCoexistProcesses();
  const timer = scenario.during?.(dir);
  console.log(`\n=== ${scenario.name}: ${scenario.about}`);
  const first = await runDsh(dir, scenario.task, evidence, "first");
  clearTimeout(timer);
  if (/MISSING_CREDENTIAL/.test(first.stderr)) {
    console.log("BLOCKED  dsh has no DeepSeek API key: set DEEPSEEK_API_KEY or store it on dsh's web Models page");
    process.exit(2);
  }
  let second;
  if (scenario.second) {
    second = await runDsh(dir, scenario.second, evidence, "second", scenario.resume ? first.session : undefined);
  }
  const orphanWaits = Math.max(0, agentCoexistProcesses() - before);
  const run = {
    calls: toolCalls(first.events, dir),
    secondCalls: second ? toolCalls(second.events, dir) : [],
    final: second ? second.final : first.final,
    orphanWaits,
  };
  writeFileSync(join(evidence, "git-status.txt"), git(dir, "status", "--porcelain", "--untracked-files=all"));
  writeFileSync(join(evidence, "final.txt"), [first.final, second?.final].filter(Boolean).join("\n\n----- second -----\n\n"));
  const checks = scenario.judge(dir, run).map((c) => (scenario.observe ? { ...c, ok: true, info: true } : c));
  const ok = checks.filter((c) => !c.info).every((c) => c.ok);
  report.push({ scenario: scenario.name, about: scenario.about, ok, observe: scenario.observe === true, seconds: first.seconds + (second?.seconds ?? 0),
    tools: run.calls.length + run.secondCalls.length, orphanWaits, projectDir: dir, checks });
  writeFileSync(join(evidence, "verdict.json"), `${JSON.stringify(report[report.length - 1], null, 2)}\n`);
  cpSync(dir, join(evidence, "project"), { recursive: true });
  rmSync(dir, { recursive: true, force: true });
  for (const c of checks) console.log(`  ${c.info ? "info" : c.ok ? "PASS" : "FAIL"}  ${c.label}${c.detail ? `  (${c.detail})` : ""}`);
  console.log(`  ${scenario.observe ? "OBSERVED" : ok ? "PASSED" : "FAILED"} in ${report[report.length - 1].seconds}s, ${report[report.length - 1].tools} tool call(s)`);
}

const failed = report.filter((r) => !r.ok).length;
const header = SELF_TEST ? "oracle self-test" : "dsh live scenarios";
const summary = [
  `# ${header}, ${stamp}`,
  "",
  `agent-coexist: ${binaryVersion} (${BIN})`,
  `dsh: ${dshManifest.version}`,
  "",
  ...report.map((r) => `- ${r.observe ? "OBSERVED" : r.ok ? "PASS" : "FAIL"} ${r.scenario}${r.actor ? ` (${r.actor} agent)` : ""}`),
  "",
  `${report.length - failed} of ${report.length} as expected`,
  "",
].join("\n");
writeFileSync(join(runRoot, "summary.md"), summary);
console.log(`\n${report.length - failed} of ${report.length} as expected; evidence in ${runRoot}`);
if (toolsDir) rmSync(toolsDir, { recursive: true, force: true });
process.exit(failed === 0 ? 0 : 1);
