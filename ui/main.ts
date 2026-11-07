// Settings window: fill the form, live-preview today's times, save through Rust.
import { type Config, getConfig, preview, saveConfig, stopAdhan, testAdhan } from "./api.ts";
import { type Madhab, METHODS, type Method, suggest } from "./methods.ts";
import { bindPlace } from "./place.ts";

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

const DEFAULTS: Config = {
  place: "", lat: Number.NaN, lon: Number.NaN, method: "MuslimWorldLeague", madhab: "Shafi",
  hijriOffset: 0, adhan: [true, true, true, true, true], volume: 80, autostart: true,
};
let place = "";

const radio = (name: string): string => (form.elements.namedItem(name) as RadioNodeList).value;
const setRadio = (name: string, value: string): void => {
  (form.elements.namedItem(name) as RadioNodeList).value = value;
};
const showError = (msg: string): void => {
  error.textContent = msg;
  error.hidden = !msg;
};

/** The form as a Config; a typed-in position is named by its coordinates. */
function read(): Config {
  const [a = true, b = true, c = true, d = true, e = true] = adhan.map((x) => x.checked);
  const la = lat.valueAsNumber;
  const lo = lon.valueAsNumber;
  return {
    place: place || `${la.toFixed(4)}, ${lo.toFixed(4)}`,
    lat: la, lon: lo,
    method: method.value as Method,
    madhab: radio("madhab") as Madhab,
    hijriOffset: Number(radio("hijri")),
    adhan: [a, b, c, d, e],
    volume: volume.valueAsNumber,
    autostart: autostart.checked,
  };
}

function write(cfg: Config): void {
  place = cfg.place;
  city.value = cfg.place;
  lat.value = Number.isFinite(cfg.lat) ? String(cfg.lat) : "";
  lon.value = Number.isFinite(cfg.lon) ? String(cfg.lon) : "";
  method.value = cfg.method;
  setRadio("madhab", cfg.madhab);
  setRadio("hijri", String(cfg.hijriOffset));
  adhan.forEach((x, i) => { x.checked = cfg.adhan[i] ?? true; });
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
  };
  if (!Number.isFinite(cfg.lat) || !Number.isFinite(cfg.lon)) return blank("Pick a city or enter coordinates.");
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
const later = (): void => {
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
for (const el of [lat, lon]) {
  el.addEventListener("input", () => {
    place = "";
    city.value = "";
  });
}
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
