import { invoke } from "@tauri-apps/api/core";
import type { PipelineResult } from "../types";

export interface RunPipelineInput {
  idea: string;
  numConcepts: number;
  autoApprove: boolean;
  checkpointContext?: string;
}

export async function runFullPipeline(
  input: RunPipelineInput,
): Promise<PipelineResult> {
  return invoke("run_full_pipeline", {
    idea: input.idea,
    numConcepts: input.numConcepts,
    autoApprove: input.autoApprove,
    checkpoint: input.checkpointContext,
  });
}

export async function runPipelineStage(
  stage: string,
  input: string,
  model: string,
  checkpointContext?: string,
): Promise<string> {
  return invoke("run_pipeline_stage", { stage, input, model, checkpointContext });
}

export async function getAvailableModels(): Promise<string[]> {
  return invoke("get_available_models");
}

export async function checkOllamaHealth(): Promise<boolean> {
  return invoke("check_ollama_health");
}
