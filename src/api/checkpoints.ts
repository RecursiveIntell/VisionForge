import { invoke } from "@tauri-apps/api/core";
import type {
  CheckpointProfile,
  PromptTerm,
  CheckpointObservation,
} from "../types";

export async function upsertCheckpoint(
  profile: CheckpointProfile,
): Promise<number> {
  return invoke("upsert_checkpoint", { profile });
}

export async function getCheckpoint(
  filename: string,
): Promise<CheckpointProfile | null> {
  return invoke("get_checkpoint", { filename });
}

export async function listCheckpointProfiles(): Promise<CheckpointProfile[]> {
  return invoke("list_checkpoint_profiles");
}

export async function addPromptTerm(term: PromptTerm): Promise<number> {
  return invoke("add_prompt_term", { term });
}

export async function getPromptTerms(
  checkpointId: number,
): Promise<PromptTerm[]> {
  return invoke("get_prompt_terms", { checkpointId });
}

export async function addCheckpointObservation(
  observation: CheckpointObservation,
): Promise<number> {
  return invoke("add_checkpoint_observation", { observation });
}

export async function getCheckpointObservations(
  checkpointId: number,
): Promise<CheckpointObservation[]> {
  return invoke("get_checkpoint_observations", { checkpointId });
}

export async function getCheckpointContext(filename: string): Promise<string> {
  return invoke("get_checkpoint_context", { filename });
}
