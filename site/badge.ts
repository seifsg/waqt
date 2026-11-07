// Draws Waqt's tray badge on a canvas with the app's own 3x5 pixel font, for the hero demo.

const FONT: Record<string, readonly number[]> = {
  "0": [7, 5, 5, 5, 7], "1": [2, 6, 2, 2, 7], "2": [7, 1, 7, 4, 7], "3": [7, 1, 7, 1, 7], "4": [5, 5, 7, 1, 1],
  "5": [7, 4, 7, 1, 7], "6": [7, 4, 7, 5, 7], "7": [7, 1, 2, 2, 2], "8": [7, 5, 7, 5, 7], "9": [7, 5, 7, 1, 7],
  h: [4, 4, 7, 5, 5],
};

type Look = { bg: string; fg: string };
const TEAL: Look = { bg: "#2bb3a3", fg: "#0b1626" };
const GOLD: Look = { bg: "#e8b85c", fg: "#0b1626" };
const NIGHT: Look = { bg: "#1e3550", fg: "#f7f3ea" };

/** Same rule as the app: minutes under 100 (gold in the last 15), else hours on night blue. */
export function badgeFor(mins: number): { text: string; look: Look } {
  if (mins <= 15) return { text: String(mins), look: GOLD };
  if (mins < 100) return { text: String(mins), look: TEAL };
  const h = Math.floor(mins / 60);
  return { text: h < 10 ? `${h}h` : String(h), look: NIGHT };
}

/** Renders `text` (digits or 'h', max 2) as a 16 px badge scaled up crisply to the canvas size. */
export function drawBadge(canvas: HTMLCanvasElement, text: string, look: Look): void {
  const ctx = canvas.getContext("2d");
  if (!ctx) throw new Error("2D canvas unavailable");
  const unit = canvas.width / 16;
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  ctx.fillStyle = look.bg;
  ctx.beginPath();
  ctx.roundRect(0, 0, canvas.width, canvas.height, 4 * unit);
  ctx.fill();
  ctx.fillStyle = look.fg;
  const k = 2; // glyph scale at 16 px, as in the app
  const x0 = (16 - (text.length * 4 * k - k)) / 2;
  [...text].forEach((ch, g) => {
    (FONT[ch] ?? []).forEach((bits, row) => {
      for (let col = 0; col < 3; col++) {
        if ((bits >> (2 - col)) & 1) ctx.fillRect((x0 + (g * 4 + col) * k) * unit, (3 + row * k) * unit, k * unit, k * unit);
      }
    });
  });
}
