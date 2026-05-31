import { invoke } from "@tauri-apps/api/core";
import type { BatchJob, BatchRequest, BatchPreview } from "../types";

export async function submitBatchJob(request: BatchRequest): Promise<string> {
  return invoke("submit_batch_job", { request });
}

export async function getBatchJobs(): Promise<BatchJob[]> {
  return invoke("get_batch_jobs");
}

export async function getBatchJob(jobId: string): Promise<BatchJob | null> {
  return invoke("get_batch_job", { jobId });
}

export async function cancelBatchItem(
  jobId: string,
  imageId: string
): Promise<void> {
  return invoke("cancel_batch_item", { jobId, imageId });
}

export async function cancelBatchJob(jobId: string): Promise<void> {
  return invoke("cancel_batch_job", { jobId });
}

export async function retryBatchFailed(jobId: string): Promise<void> {
  return invoke("retry_batch_failed", { jobId });
}

export async function getBatchEta(jobId: string): Promise<number | null> {
  return invoke("get_batch_eta", { jobId });
}

export async function previewBatchJob(
  request: BatchRequest
): Promise<BatchPreview> {
  return invoke("preview_batch_job", { request });
}
