// Builds THIRD-PARTY-NOTICES.txt for the installer: the license files of every crate linked into
// Waqt's Windows binary and of the npm code inside its settings window, each distinct text once.
// Committed so plain cargo builds find it; every Tauri build regenerates it and CI fails if stale.
import { execFileSync } from "node:child_process";
import { readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, relative } from "node:path";

type Pkg = { id: string; name: string; version: string; license: string | null; license_file: string | null; manifest_path: string; source: string | null };
type Dep = { pkg: string; dep_kinds: { kind: string | null }[] };
type Meta = { packages: Pkg[]; resolve: { root: string; nodes: { id: string; deps: Dep[] }[] } };
type Part = { name: string; license: string; dir: string };

const LICENSE_FILE = /^(licen[cs]e|copying|notice)/i;
const NPM = ["@tauri-apps/api"]; // the only npm package bundled into the settings window
const ROOT = join(import.meta.dirname, "..");
/** A phrase every copy of each license's standard text contains. */
const SIGNATURE: Record<string, RegExp> = {
  "0BSD": /Permission to use, copy, modify, and\/or distribute this software/,
  "Apache-2.0": /Apache License/,
  "BSD-3-Clause": /Neither the name of/,
  "CC0-1.0": /CC0 1\.0/,
  MIT: /Permission is hereby granted, free of charge/,
  "MIT-0": /Permission is hereby granted, free of charge/,
  "MPL-2.0": /Mozilla Public License,? [Vv]ersion 2\.0/,
  "Unicode-3.0": /UNICODE LICENSE V3/i,
  Unlicense: /free and unencumbered software released into the public domain/,
  Zlib: /provided ['‘]as-is['’]/,
};

/** A map value that must exist; a miss means cargo metadata changed shape. */
function must<K, V>(map: Map<K, V>, key: K): V {
  const v = map.get(key);
  if (v === undefined) throw new Error(`cargo metadata: unknown id ${String(key)}`);
  return v;
}

/** One crate's entry; vendored crates point at their folder in this repository. */
function part(p: Pkg): Part {
  if (!p.license && !p.license_file) throw new Error(`${p.name} ${p.version} declares no license`);
  const dir = dirname(p.manifest_path);
  const where = p.source === null ? `, vendored with patches in Waqt's ${relative(ROOT, dir).replaceAll("\\", "/")}` : "";
  return { name: `${p.name} ${p.version}${where}`, license: p.license ?? "license in its file", dir };
}

/** Crates the binary links: normal dependencies reachable from Waqt, Waqt itself excluded. */
function crates(): Part[] {
  const args = ["metadata", "--format-version", "1", "--locked", "--manifest-path", join(ROOT, "src-tauri/Cargo.toml"), "--filter-platform", "x86_64-pc-windows-msvc"];
  const meta = JSON.parse(execFileSync("cargo", args, { encoding: "utf8", maxBuffer: 1 << 28 })) as Meta;
  const pkgs = new Map(meta.packages.map((p) => [p.id, p]));
  const deps = new Map(meta.resolve.nodes.map((n) => [n.id, n.deps]));
  const seen = new Set<string>();
  const todo = [meta.resolve.root];
  for (let id = todo.pop(); id !== undefined; id = todo.pop()) {
    for (const d of must(deps, id)) {
      if (d.dep_kinds.some((k) => k.kind === null) && !seen.has(d.pkg)) {
        seen.add(d.pkg);
        todo.push(d.pkg);
      }
    }
  }
  return [...seen].map((id) => part(must(pkgs, id)));
}

/** npm packages bundled into the settings window. */
function npm(): Part[] {
  return NPM.map((name) => {
    const dir = join(ROOT, "node_modules", name);
    const pkg = JSON.parse(readFileSync(join(dir, "package.json"), "utf8")) as { version: string; license: string };
    return { name: `${name} ${pkg.version} (npm)`, license: pkg.license, dir };
  });
}

/** The package's license texts, read from its folder. */
function texts(dir: string): string[] {
  return readdirSync(dir)
    .filter((f) => LICENSE_FILE.test(f) && statSync(join(dir, f)).isFile())
    .map((f) => readFileSync(join(dir, f), "utf8").replace(/\r\n/g, "\n").trim());
}

/** Whether `all` holds a standard text for `expr` (every id of an AND, any of an OR); unknown ids throw. */
function covered(expr: string, all: string): boolean {
  const ids = expr.split(/ OR | AND |\/|[()]/).map((s) => s.replace(/ WITH .*/, "").trim()).filter(Boolean);
  const found = ids.map((id) => {
    const sig = SIGNATURE[id];
    if (!sig) throw new Error(`no SIGNATURE for license ${id} in scripts/notices.ts`);
    return sig.test(all);
  });
  return expr.includes(" AND ") ? found.every(Boolean) : found.some(Boolean);
}

const byText = new Map<string, string[]>();
const bare: Part[] = [];
const parts = [...crates(), ...npm()].sort((a, b) => a.name.localeCompare(b.name));
for (const p of parts) {
  const found = texts(p.dir);
  if (!found.length) bare.push(p);
  for (const t of found) byText.set(t, [...(byText.get(t) ?? []), `${p.name} (${p.license})`]);
}
const all = [...byText.keys()].join("\n");
const missing = bare.filter((p) => !covered(p.license, all));
if (missing.length) throw new Error(`no license text for: ${missing.map((p) => `${p.name} (${p.license})`).join(", ")}`);

const rule = "=".repeat(78);
const out = [
  `Waqt includes the ${parts.length} third-party packages below. Each license text follows the packages it covers.`,
  "Source code of each package, at the version shown: https://crates.io/crates/<name>/<version> (Rust), https://www.npmjs.com/package/<name> (npm), or the folder named for a vendored crate in Waqt's repository (https://github.com/seifsg/waqt).",
  "Bundled data and media (adhan recording, GeoNames places): see CREDITS.md.",
  ...[...byText].map(([text, names]) => `${rule}\n${names.join("\n")}\n${"-".repeat(78)}\n${text}`),
  ...(bare.length ? [`${rule}\nThese ship no license file; the standard text of each declared license appears above:\n${bare.map((p) => `${p.name} (${p.license})`).join("\n")}`] : []),
];
writeFileSync(join(ROOT, "THIRD-PARTY-NOTICES.txt"), `${out.join("\n\n")}\n`);
console.log(`${parts.length} packages, ${byText.size} license texts, ${bare.length} without a file`);
