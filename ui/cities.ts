// Offline place search over the bundled GeoNames list (most populous first).

export interface City {
  name: string;
  region: string;
  cc: string;
  lat: number;
  lon: number;
}

const country = new Intl.DisplayNames(["en"], { type: "region" });
let rows: { key: string; line: string }[] | undefined;

/** Lower-case, accent-free form used for matching ("São Paulo" → "sao paulo"). */
const fold = (s: string): string => s.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase();

/** Loads the list once, on first search. */
async function load(): Promise<NonNullable<typeof rows>> {
  if (!rows) {
    const res = await fetch("/cities.tsv");
    if (!res.ok) throw new Error(`Place list failed to load (${res.status}).`);
    rows = (await res.text()).split("\n").filter(Boolean).map((line) => ({ key: fold(line.slice(0, line.indexOf("\t"))), line }));
  }
  return rows;
}

function parse(line: string): City {
  const [name = "", region = "", cc = "", lat = "", lon = ""] = line.split("\t");
  return { name, region, cc, lat: Number(lat), lon: Number(lon) };
}

/** Country name for a code, e.g. "TN" → "Tunisia". */
export const countryName = (cc: string): string => country.of(cc) ?? cc;

/** Up to `limit` places: name prefix matches first, then names containing the query. */
export async function search(query: string, limit = 8): Promise<City[]> {
  const q = fold(query.trim());
  if (q.length < 2) return [];
  const prefix: string[] = [];
  const inner: string[] = [];
  for (const r of await load()) {
    if (r.key.startsWith(q)) {
      if (prefix.push(r.line) === limit) break;
    } else if (inner.length < limit && r.key.includes(q)) {
      inner.push(r.line);
    }
  }
  return [...prefix, ...inner].slice(0, limit).map(parse);
}
