import { invoke } from "@tauri-apps/api/core";
import type { SeedEntry, SeedCheckpointNote, SeedFilter } from "../types";

export async function createSeed(seed: SeedEntry): Promise<number> {
  return invoke("create_seed", { seed });
}

export async function getSeed(id: number): Promise<SeedEntry | null> {
  return invoke("get_seed", { id });
}

export async function listSeeds(filter: SeedFilter): Promise<SeedEntry[]> {
  return invoke("list_seeds", { filter });
}

export async function deleteSeed(id: number): Promise<void> {
  return invoke("delete_seed", { id });
}

export async function addSeedTag(
  seedId: number,
  tagName: string,
): Promise<void> {
  return invoke("add_seed_tag", { seedId, tagName });
}

export async function removeSeedTag(
  seedId: number,
  tagId: number,
): Promise<void> {
  return invoke("remove_seed_tag", { seedId, tagId });
}

export async function addSeedCheckpointNote(
  note: SeedCheckpointNote,
): Promise<void> {
  return invoke("add_seed_checkpoint_note", { note });
}

export async function getSeedCheckpointNotes(
  seedId: number,
): Promise<SeedCheckpointNote[]> {
  return invoke("get_seed_checkpoint_notes", { seedId });
}
