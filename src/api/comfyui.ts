import { invoke } from "@tauri-apps/api/core";
import type { GenerationRequest, GenerationStatus } from "../types";

export async function checkComfyuiHealth(): Promise<boolean> {
  return invoke("check_comfyui_health");
}

export async function getComfyuiCheckpoints(): Promise<string[]> {
  return invoke("get_comfyui_checkpoints");
}

export async function getComfyuiSamplers(): Promise<string[]> {
  return invoke("get_comfyui_samplers");
}

export async function getComfyuiSchedulers(): Promise<string[]> {
  return invoke("get_comfyui_schedulers");
}

export async function queueGeneration(
  request: GenerationRequest,
): Promise<GenerationStatus> {
  return invoke("queue_generation", { request });
}

export async function getGenerationStatus(
  promptId: string,
): Promise<GenerationStatus> {
  return invoke("get_generation_status", { promptId });
}

export interface QueueStatus {
  running: number;
  pending: number;
}

export async function getComfyuiQueueStatus(): Promise<QueueStatus> {
  return invoke("get_comfyui_queue_status");
}

export async function freeComfyuiMemory(
  unloadModels: boolean,
): Promise<void> {
  return invoke("free_comfyui_memory", { unloadModels });
}

export async function interruptComfyui(): Promise<void> {
  return invoke("interrupt_comfyui");
}
