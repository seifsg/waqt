// Builds THIRD-PARTY-NOTICES.txt for the installer: the license files of every crate linked into
// Waqt's Windows binary and of the npm code inside its settings window, each distinct text once.
// Runs before every Tauri build (tauri.conf.json), so it never goes stale.
import { execFileSync } from "node:child_process";
import { readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";

type Pkg = { id: string; name: string; version: string; license: string | null; manifest_path: string };
type Dep = { pkg: string; dep_kinds: { kind: string | null }[] };
type Meta = { packages: Pkg[]; resolve: { root: string; nodes: { id: string; deps: Dep[] }[] } };
type Part = { name: string; license: string; dir: string };

const LICENSE_FILE = /^(licen[cs]e|copying|notice)/i;
const NPM = ["@tauri-apps/api"]; // the only npm package bundled into the settings window

/** A map value that must exist; a miss means cargo metadata changed shape. */
function must<K, V>(map: Map<K, V>, key: K): V {
  const v = map.get(key);
  if (v === undefined) throw new Error(`cargo metadata: unknown id ${String(key)}`);
  return v;
}

/** Crates the binary links: normal dependencies reachable from Waqt, Waqt itself excluded. */
function crates(): Part[] {
  const args = ["metadata", "--format-version", "1", "--locked", "--manifest-path", "src-tauri/Cargo.toml", "--filter-platform", "x86_64-pc-windows-msvc"];
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
  return [...seen].map((id) => must(pkgs, id)).map((p) => ({ name: `${p.name} ${p.version}`, license: p.license ?? "see its license file", dir: dirname(p.manifest_path) }));
}

/** npm packages bundled into the settings window. */
function npm(): Part[] {
  return NPM.map((name) => {
    const dir = join("node_modules", name);
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

const byText = new Map<string, string[]>();
const bare: string[] = [];
const parts = [...crates(), ...npm()].sort((a, b) => a.name.localeCompare(b.name));
for (const p of parts) {
  const found = texts(p.dir);
  if (!found.length) bare.push(`${p.name} (${p.license})`);
  for (const t of found) byText.set(t, [...(byText.get(t) ?? []), `${p.name} (${p.license})`]);
}

const rule = "=".repeat(78);
const out = [
  `Waqt includes the ${parts.length} third-party packages below. Each license text follows the packages it covers.`,
  "Source code of each package, at the version shown: https://crates.io/crates/<name>/<version> (Rust) or https://www.npmjs.com/package/<name> (npm).",
  "Bundled data and media (adhan recording, GeoNames places): see CREDITS.md.",
  ...[...byText].map(([text, names]) => `${rule}\n${names.join("\n")}\n${"-".repeat(78)}\n${text}`),
  ...(bare.length ? [`${rule}\nThese ship no license file; the standard text of each declared license appears above:\n${bare.join("\n")}`] : []),
];
writeFileSync(new URL("../THIRD-PARTY-NOTICES.txt", import.meta.url), `${out.join("\n\n")}\n`);
console.log(`${parts.length} packages, ${byText.size} license texts, ${bare.length} without a file`);
