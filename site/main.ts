// Landing page behavior: a live, sped-up demo of the tray badge through a real day's prayers.
import "@fontsource-variable/inter";
import "./style.css";
import { badgeFor, drawBadge } from "./badge.ts";

/** The element matching `sel`; index.html must have it. */
const $ = <T extends HTMLElement>(sel: string): T => {
  const el = document.querySelector<T>(sel);
  if (!el) throw new Error(`${sel} is missing from index.html`);
  return el;
};
const canvas = $<HTMLCanvasElement>("#badge");
const tip = $<HTMLElement>("#tip");
const clock = $<HTMLElement>(".clock");

// Tunis, 7 Nov 2026, Muslim World League: the same day as the menu screenshot.
const DAY: ReadonlyArray<[string, number]> = [["Fajr", 321], ["Dhuhr", 724], ["Asr", 894], ["Maghrib", 1036], ["Isha", 1120]];
/** Minutes since midnight as "HH:MM". */
const hhmm = (m: number): string => `${String(Math.floor(m / 60) % 24).padStart(2, "0")}:${String(m % 60).padStart(2, "0")}`;
/** Minutes left as "45m" or "2h 05m", like the app's tooltip. */
const left = (m: number): string => (m < 60 ? `${m}m` : `${Math.floor(m / 60)}h ${String(m % 60).padStart(2, "0")}m`);
let now = 894 - 18; // 18 minutes before Asr

/** Restarts the short roll-in animation on `el` (CSS class `roll`). */
function roll(el: HTMLElement): void {
  el.classList.remove("roll");
  void el.offsetWidth; // reflow so the animation plays again
  el.classList.add("roll");
}

/** Shows the badge and tooltip for the virtual clock, then moves it on (faster while far away). */
function tick(): void {
  const [name, at] = DAY.find(([, t]) => t > now) ?? ["Fajr", DAY[0]![1] + 1440];
  const mins = at - now;
  const { text, look } = badgeFor(mins);
  drawBadge(canvas, text, look);
  tip.textContent = `${name} in ${left(mins)} · ${hhmm(at)}`;
  roll(canvas);
  roll(tip);
  clock.textContent = hhmm(now);
  now = (now + (mins > 30 ? 5 : 1)) % 1440;
}

tick();
if (!matchMedia("(prefers-reduced-motion: reduce)").matches) setInterval(tick, 1100);
