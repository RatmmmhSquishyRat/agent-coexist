// Deterministic check of how dsh (DeepSeek harness) discovers the bundled
// agent-coexist skill. Drives dsh's own FileSystemSkillProvider — the code dsh
// runs at session start — against a fresh test project, so no model call and
// no API key are involved. The result is exactly the catalog entry the dsh
// model is shown.
//
//   node tests/dsh/check-discovery.mjs
//
// Exits 0 when every check passes, 1 otherwise. See README.md.

import { execFileSync, execSync } from "node:child_process";
import { cpSync, mkdirSync, readFileSync, rmSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const productRoot = resolve(here, "..", "..");
const shippedSkill = join(productRoot, ".agents", "skills", "agent-coexist");

// dsh is installed globally by npm; resolve its bundled plugins from there.
const npmRoot = execSync("npm root -g", { encoding: "utf8" }).trim();
const dshRoot = join(npmRoot, "@deepseek-ai", "dsh");
const dshPlugin = (name) => join(dshRoot, "node_modules", "@deepseek-ai", name);
const dshVersion = JSON.parse(readFileSync(join(dshRoot, "package.json"), "utf8")).version;

const { FileSystemSkillProvider } = await import(
  pathToFileURL(join(dshPlugin("dsh-skill-filesystem"), "lib", "index.js")).href
);

// The catalog cap is read from the installed dsh rather than copied here, so
// this check follows dsh if the limit ever changes.
const toolSkillSource = readFileSync(join(dshPlugin("dsh-tool-skill"), "lib", "index.js"), "utf8");
const capMatch = toolSkillSource.match(/DEFAULT_CATALOG_DESCRIPTION_MAX_LENGTH\s*=\s*(\d+)/);
if (!capMatch) throw new Error("could not find DEFAULT_CATALOG_DESCRIPTION_MAX_LENGTH in dsh-tool-skill");
const CAP = Number(capMatch[1]);

// dsh-tool-skill's catalogDescription(): collapse whitespace runs, then keep
// the first CAP-3 characters and append "..." when over the cap.
function asShownToModel(description) {
  const normalized = description.replaceAll(/\s+/g, " ").trim();
  return normalized.length <= CAP ? normalized : `${normalized.slice(0, CAP - 3)}...`;
}

// A fresh project with its own git root. dsh resolves the project root to the
// nearest ancestor containing .git; without one here it would climb to the
// product repository and find the product's own copy of the skill, so the
// check would pass while testing nothing.
const work = join(here, "work", `discovery-${Date.now()}`);
rmSync(work, { recursive: true, force: true });
mkdirSync(work, { recursive: true });
execFileSync("git", ["init", "-q"], { cwd: work });
cpSync(shippedSkill, join(work, ".agents", "skills", "agent-coexist"), { recursive: true });

// The provider asks its context for an optional virtual filesystem service
// (ctx.get("fs")); none registered means it reads the real disk, which is the
// local case being checked. Watching is off: one scan is all this needs.
const provider = new FileSystemSkillProvider(
  { get: () => undefined },
  { invalidate() {}, signal: new AbortController().signal },
  { watch: false },
);
const listed = await provider.list({ cwd: work });
const candidates = Array.isArray(listed) ? listed : (listed.candidates ?? listed.skills ?? []);

const results = [];
const check = (ok, label, detail = "") => results.push({ ok, label, detail });

const ours = candidates.filter((c) => c.name === "agent-coexist");
check(ours.length >= 1, "skill discovered by dsh", `${candidates.length} skill(s) listed in total`);

if (ours.length >= 1) {
  const skill = ours[0];
  const shipped = readFileSync(join(shippedSkill, "SKILL.md"), "utf8");
  const raw = skill.description ?? "";
  const shown = asShownToModel(raw);
  const location = JSON.stringify(skill.source ?? skill.provider ?? skill.root ?? skill.locator ?? "");

  check(
    ours.length === 1,
    "exactly one agent-coexist skill visible",
    ours.length > 1 ? "a user-level copy may be shadowing or duplicating the project one" : "",
  );
  check(/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(skill.name), "name matches dsh's skill-name grammar");
  check(
    raw.length > 0 && shipped.includes(raw.slice(0, 40)),
    "description is the shipped one, not a stale copy",
  );
  check(
    raw.replaceAll(/\s+/g, " ").trim().length <= CAP,
    `description fits the ${CAP}-character catalog cap`,
    `${raw.replaceAll(/\s+/g, " ").trim().length} characters`,
  );
  check(
    /mention/i.test(shown) && /not a reason/i.test(shown),
    "over-trigger guard is visible to the model",
    "the sentence telling the agent not to create files merely because the word appears",
  );
  for (const cue of ["harness", "session", "subagent"]) {
    check(new RegExp(cue, "i").test(shown), `trigger cue "${cue}" is visible to the model`);
  }

  console.log(`dsh ${dshVersion}   catalog cap ${CAP}`);
  console.log(`project: ${work}`);
  console.log(`source:  ${location}`);
  console.log(`\nwhat the dsh model sees for agent-coexist (${shown.length} chars):`);
  console.log(`  ${shown}\n`);
}

let failed = 0;
for (const r of results) {
  if (!r.ok) failed++;
  console.log(`${r.ok ? "PASS" : "FAIL"}  ${r.label}${r.detail ? `  (${r.detail})` : ""}`);
}
console.log(`\n${results.length - failed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
