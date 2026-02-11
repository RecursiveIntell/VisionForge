import { invoke } from "@tauri-apps/api/core";
import type { QueueJob, QueuePriority } from "../types";

export async function addToQueue(job: QueueJob): Promise<string> {
  return invoke("add_to_queue", { job });
}

export async function getQueue(): Promise<QueueJob[]> {
  return invoke("get_queue");
}

export async function reorderQueue(
  jobId: string,
  newPriority: QueuePriority,
): Promise<void> {
  return invoke("reorder_queue", { jobId, newPriority });
}

export async function cancelQueueJob(jobId: string): Promise<void> {
  return invoke("cancel_queue_job", { jobId });
}

export async function pauseQueue(): Promise<void> {
  return invoke("pause_queue");
}

export async function resumeQueue(): Promise<void> {
  return invoke("resume_queue");
}

export async function isQueuePaused(): Promise<boolean> {
  return invoke("is_queue_paused");
}
