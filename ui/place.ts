// Accessible place combobox: type, arrow through matches, Enter or click to pick.
import { type City, countryName, search } from "./cities.ts";

/** "Tunis, Tunis Governorate, Tunisia" (region skipped when it repeats the name). */
export const cityLabel = (c: City): string =>
  [c.name, c.region && c.region !== c.name ? c.region : "", countryName(c.cc)].filter(Boolean).join(", ");

/** Wires `input` + `list` as a combobox; `onPick` gets the chosen city, `onError` load failures. */
export function bindPlace(input: HTMLInputElement, list: HTMLUListElement, onPick: (c: City) => void, onError: (msg: string) => void): void {
  let items: City[] = [];
  let active = -1;
  let seq = 0;

  /** Redraws the option list and its ARIA state. */
  const render = (): void => {
    list.replaceChildren(
      ...items.map((c, i) => {
        const li = document.createElement("li");
        li.id = `place-${i}`;
        li.role = "option";
        li.dataset.i = String(i);
        li.ariaSelected = String(i === active);
        const small = document.createElement("small");
        small.textContent = [c.region, countryName(c.cc)].filter(Boolean).join(", ");
        li.append(c.name, small);
        return li;
      }),
    );
    list.hidden = items.length === 0;
    input.ariaExpanded = String(!list.hidden);
    input.setAttribute("aria-activedescendant", active >= 0 ? `place-${active}` : "");
    list.querySelector('[aria-selected="true"]')?.scrollIntoView({ block: "nearest" });
  };
  /** Empties and hides the list. */
  const close = (): void => {
    items = [];
    active = -1;
    render();
  };
  /** Fills the field with option `i` and reports it. */
  const pick = (i: number): void => {
    const c = items[i];
    if (!c) return;
    input.value = cityLabel(c);
    close();
    onPick(c);
  };

  input.addEventListener("input", async () => {
    const mine = ++seq;
    try {
      const found = await search(input.value);
      if (mine !== seq) return; // a newer keystroke owns the list
      items = found;
      active = found.length ? 0 : -1;
      render();
    } catch (e) {
      onError(String(e instanceof Error ? e.message : e));
    }
  });
  input.addEventListener("keydown", (e) => {
    if (list.hidden) return;
    if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      active = (active + (e.key === "ArrowDown" ? 1 : items.length - 1)) % items.length;
      render();
    } else if (e.key === "Enter") {
      e.preventDefault();
      pick(active);
    } else if (e.key === "Escape") {
      close();
    }
  });
  list.addEventListener("mousedown", (e) => {
    const li = (e.target as Element).closest("li");
    if (li) {
      e.preventDefault();
      pick(Number(li.dataset.i));
    }
  });
  input.addEventListener("blur", close);
}
