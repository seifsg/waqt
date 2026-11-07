// Settings window: fill the form, live-preview today's times, save through Rust.
import { type Config, getConfig, preview, saveConfig, stopAdhan, testAdhan } from "./api.ts";
import { type Madhab, METHODS, type Method, suggest } from "./methods.ts";
import { preload } from "./cities.ts";
import { bindPlace } from "./place.ts";

/** The element with `id`; index.html must have it. */
const $ = <T extends HTMLElement>(id: string): T => {
  const el = document.getElementById(id);
  if (!el) throw new Error(`#${id} is missing from index.html`);
  return el as T;
};
const form = $<HTMLFormElement>("form");
const city = $<HTMLInputElement>("city");
const lat = $<HTMLInputElement>("lat");
const lon = $<HTMLInputElement>("lon");
const method = $<HTMLSelectElement>("method");
const volume = $<HTMLInputElement>("volume");
const autostart = $<HTMLInputElement>("autostart");
const error = $<HTMLParagraphElement>("error");
const save = $<HTMLButtonElement>("save");
const adhan = [...form.querySelectorAll<HTMLInputElement>('input[name="adhan"]')];
const PRAYERS = ["Fajr", "Dhuhr", "Asr", "Maghrib", "Isha"];
if (adhan.length !== PRAYERS.length) throw new Error("index.html needs one adhan checkbox per prayer");

const DEFAULTS: Config = {
  place: "", lat: Number.NaN, lon: Number.NaN, method: "MuslimWorldLeague", madhab: "Shafi",
  hijriOffset: 0, adhan: [true, true, true, true, true], volume: 80, autostart: true,
};
let place = "";

/** Value of the checked radio in group `name`. */
const radio = (name: string): string => (form.elements.namedItem(name) as RadioNodeList).value;
/** Checks the radio in group `name` with `value`. */
const setRadio = (name: string, value: string): void => {
  (form.elements.namedItem(name) as RadioNodeList).value = value;
};
/** Error-message prefix → the fields it is about, marked invalid while it shows. */
const FIELDS: ReadonlyArray<[string, HTMLElement[]]> = [
  ["Latitude", [lat]], ["Longitude", [lon]], ["Name the place", [city]], ["Pick a city", [city, lat, lon]],
];
/** Shows `msg` (empty hides it) and marks the fields it is about. */
const showError = (msg: string): void => {
  error.textContent = msg;
  error.hidden = !msg;
  const bad = new Set(FIELDS.filter(([prefix]) => msg.startsWith(prefix)).flatMap(([, els]) => els));
  for (const el of [city, lat, lon]) el.toggleAttribute("aria-invalid", bad.has(el));
};

/** The form as a Config; a typed-in position is named by its coordinates. */
function read(): Config {
  const la = lat.valueAsNumber;
  const lo = lon.valueAsNumber;
  return {
    place: place || `${la.toFixed(4)}, ${lo.toFixed(4)}`,
    lat: la, lon: lo,
    method: method.value as Method,
    madhab: radio("madhab") as Madhab,
    hijriOffset: Number(radio("hijri")),
    adhan: adhan.map((x) => x.checked) as Config["adhan"], // length checked at load
    volume: volume.valueAsNumber,
    autostart: autostart.checked,
  };
}

/** Fills the form from a saved config. */
function write(cfg: Config): void {
  place = cfg.place;
  city.value = cfg.place;
  lat.value = Number.isFinite(cfg.lat) ? String(cfg.lat) : "";
  lon.value = Number.isFinite(cfg.lon) ? String(cfg.lon) : "";
  method.value = cfg.method;
  setRadio("madhab", cfg.madhab);
  setRadio("hijri", String(cfg.hijriOffset));
  adhan.forEach((x, i) => { x.checked = cfg.adhan[i]!; }); // Rust validated: five flags
  volume.value = String(cfg.volume);
  autostart.checked = cfg.autostart;
}

/** Shows the times the current form would give, or why it cannot. */
async function refresh(): Promise<void> {
  const cfg = read();
  const list = $<HTMLOListElement>("times");
  const blank = (hint: string): void => {
    list.replaceChildren(...PRAYERS.map((p) => row(p, "--:--", false)));
    $("hijri").textContent = hint;
    $("polar").hidden = true;
  };
  if (!Number.isFinite(cfg.lat) || !Number.isFinite(cfg.lon)) {
    showError("");
    return blank("Pick a city or enter coordinates.");
  }
  try {
    const p = await preview(cfg);
    list.replaceChildren(...p.times.map((t, i) => row(t.label, t.time, i === p.next)));
    $("hijri").textContent = p.hijri;
    $("polar").hidden = !p.polar;
    showError("");
  } catch (e) {
    blank("");
    showError(String(e));
  }
}

/** One prayer line of the preview; the next prayer is highlighted. */
function row(label: string, time: string, next: boolean): HTMLLIElement {
  const li = document.createElement("li");
  li.className = next ? "next" : "";
  const name = document.createElement("span");
  name.textContent = next ? `${label} · next` : label;
  const t = document.createElement("time");
  t.textContent = time;
  li.append(name, t);
  return li;
}

let timer = 0;
/** Refreshes the preview once typing pauses; city typing alone changes no times. */
const later = (e: Event): void => {
  if (e.target === city) return;
  clearTimeout(timer);
  timer = window.setTimeout(refresh, 200);
};

method.append(...METHODS.map((m) => new Option(m.label, m.id)));
bindPlace(city, $<HTMLUListElement>("city-list"), (c) => {
  place = city.value;
  lat.value = String(c.lat);
  lon.value = String(c.lon);
  const s = suggest(c.cc);
  method.value = s.method;
  setRadio("madhab", s.madhab);
  void refresh();
}, showError);
// Typing coordinates names the place by them; typing a new city drops the old pick.
for (const el of [lat, lon]) {
  el.addEventListener("input", () => {
    place = "";
    city.value = "";
  });
}
city.addEventListener("input", () => {
  if (!place) return;
  place = "";
  lat.value = "";
  lon.value = "";
  void refresh();
});
$("test").addEventListener("click", () => void testAdhan(volume.valueAsNumber).catch((e) => showError(String(e))));
$("stop").addEventListener("click", () => void stopAdhan().catch((e) => showError(String(e))));
form.addEventListener("input", later);
form.addEventListener("change", later);
form.addEventListener("submit", async (e) => {
  e.preventDefault();
  const cfg = read();
  if (!Number.isFinite(cfg.lat) || !Number.isFinite(cfg.lon)) return showError("Pick a city or enter latitude and longitude.");
  save.disabled = true;
  try {
    await saveConfig(cfg);
  } catch (err) {
    showError(String(err));
  } finally {
    save.disabled = false;
  }
});

write((await getConfig()) ?? DEFAULTS);
await refresh();
city.focus();
void preload().catch((e: unknown) => showError(String(e instanceof Error ? e.message : e)));
