// Typed calls into the Rust core. Rust validates everything; errors come back as user-facing text.
import { invoke } from "@tauri-apps/api/core";
import type { Madhab, Method } from "./methods.ts";

export interface Config {
  place: string;
  lat: number;
  lon: number;
  method: Method;
  madhab: Madhab;
  hijriOffset: number;
  adhan: [boolean, boolean, boolean, boolean, boolean];
  volume: number;
  autostart: boolean;
}

export interface Preview {
  times: { label: string; time: string }[];
  next: number;
  hijri: string;
  polar: boolean;
}

export const getConfig = (): Promise<Config | null> => invoke("get_config");
export const preview = (cfg: Config): Promise<Preview> => invoke("preview", { cfg });
export const saveConfig = (cfg: Config): Promise<void> => invoke("save_config", { cfg });
