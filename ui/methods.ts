// Calculation methods (ids match the Rust `salah::Method` names) and per-country defaults.

export type Method =
  | "MuslimWorldLeague" | "Egyptian" | "Karachi" | "UmmAlQura" | "Dubai" | "MoonsightingCommittee"
  | "NorthAmerica" | "Kuwait" | "Qatar" | "Singapore" | "Tehran" | "Turkey";
export type Madhab = "Shafi" | "Hanafi";

export const METHODS: ReadonlyArray<{ id: Method; label: string }> = [
  { id: "MuslimWorldLeague", label: "Muslim World League" },
  { id: "UmmAlQura", label: "Umm al-Qura, Makkah" },
  { id: "Egyptian", label: "Egyptian General Authority" },
  { id: "Karachi", label: "University of Islamic Sciences, Karachi" },
  { id: "NorthAmerica", label: "ISNA (North America)" },
  { id: "MoonsightingCommittee", label: "Moonsighting Committee" },
  { id: "Dubai", label: "Dubai" },
  { id: "Kuwait", label: "Kuwait" },
  { id: "Qatar", label: "Qatar" },
  { id: "Singapore", label: "Singapore, Malaysia, Indonesia" },
  { id: "Turkey", label: "Diyanet (Turkey)" },
  { id: "Tehran", label: "Institute of Geophysics, Tehran" },
];

const BY_COUNTRY: Readonly<Record<string, Method>> = {
  SA: "UmmAlQura", YE: "UmmAlQura",
  EG: "Egyptian", SD: "Egyptian", LY: "Egyptian", SY: "Egyptian", IQ: "Egyptian", LB: "Egyptian", JO: "Egyptian", PS: "Egyptian",
  PK: "Karachi", IN: "Karachi", BD: "Karachi", AF: "Karachi",
  AE: "Dubai", KW: "Kuwait", QA: "Qatar",
  SG: "Singapore", MY: "Singapore", BN: "Singapore", ID: "Singapore",
  TR: "Turkey", IR: "Tehran",
  US: "NorthAmerica", CA: "NorthAmerica", GB: "MoonsightingCommittee",
};

const HANAFI = new Set(["PK", "IN", "BD", "AF", "TR", "UZ", "TJ", "KZ", "KG", "TM", "AZ"]);

/** The usual method and Asr school for a country code. */
export function suggest(cc: string): { method: Method; madhab: Madhab } {
  return { method: BY_COUNTRY[cc] ?? "MuslimWorldLeague", madhab: HANAFI.has(cc) ? "Hanafi" : "Shafi" };
}
