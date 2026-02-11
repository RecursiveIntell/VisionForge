import { invoke } from "@tauri-apps/api/core";
import type { Comparison } from "../types";

export async function createComparison(comparison: Comparison): Promise<void> {
  return invoke("create_comparison", { comparison });
}

export async function getComparison(id: string): Promise<Comparison | null> {
  return invoke("get_comparison", { id });
}

export async function listComparisons(): Promise<Comparison[]> {
  return invoke("list_comparisons");
}

export async function listComparisonsForCheckpoint(
  checkpoint: string,
): Promise<Comparison[]> {
  return invoke("list_comparisons_for_checkpoint", { checkpoint });
}

export async function updateComparisonNote(
  id: string,
  note: string,
): Promise<void> {
  return invoke("update_comparison_note", { id, note });
}

export async function deleteComparison(id: string): Promise<void> {
  return invoke("delete_comparison", { id });
}
