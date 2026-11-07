// Builds ui/public/cities.tsv (name, region, country code, lat, lon; most populous first)
// from GeoNames dumps (CC BY 4.0): https://download.geonames.org/export/dump/
// Usage: npm run cities -- <folder holding cities1000.txt and admin1CodesASCII.txt>
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const dir = process.argv[2];
if (!dir) throw new Error("usage: npm run cities -- <folder with cities1000.txt and admin1CodesASCII.txt>");

/** Tab-separated rows of a dump file; each row must have at least `min` columns. */
function rows(file: string, min: number): string[][] {
  return readFileSync(join(dir!, file), "utf8")
    .split("\n")
    .filter((line) => line.trim())
    .map((line, i) => {
      const r = line.split("\t");
      if (r.length < min) throw new Error(`${file}:${i + 1} has ${r.length} columns, expected ${min}`);
      return r;
    });
}

const regions = new Map(rows("admin1CodesASCII.txt", 2).map((r) => [r[0]!, r[1]!]));
const places = rows("cities1000.txt", 15)
  .map((r) => ({
    pop: Number(r[14]),
    line: [r[1], regions.get(`${r[8]}.${r[10]}`) ?? "", r[8], Number(r[4]).toFixed(2), Number(r[5]).toFixed(2)].join("\t"),
  }))
  .sort((a, b) => b.pop - a.pop)
  .map((p) => p.line);

writeFileSync(new URL("../ui/public/cities.tsv", import.meta.url), `${places.join("\n")}\n`);
console.log(`${places.length} places written`);
