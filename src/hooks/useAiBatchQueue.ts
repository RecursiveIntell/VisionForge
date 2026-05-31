import { useState, useEffect, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import {
  getBatchJobs,
  cancelBatchJob,
  cancelBatchItem,
  retryBatchFailed,
} from "../api/aiBatch";
import type {
  BatchJob,
  BatchCompletionSummary,
  BatchItemStatus,
  BatchOpKind,
} from "../types";

interface BatchItemProgressPayload {
  jobId: string;
  imageId: string;
  status: BatchItemStatus;
  completed: number;
  total: number;
  error?: string;
  durationMs?: number;
  etaRemainingMs?: number;
}

interface BatchJobStartedPayload {
  jobId: string;
  op: BatchOpKind;
  model: string;
  totalItems: number;
}

interface BatchJobCompletedPayload {
  summary: BatchCompletionSummary;
}

export interface BatchQueueState {
  jobs: BatchJob[];
  activeJobId: string | null;
  activeProgress: { completed: number; total: number } | null;
  activeEtaMs: number | null;
  lastCompletion: BatchCompletionSummary | null;
  loading: boolean;

  refresh: () => Promise<void>;
  cancelJob: (jobId: string) => Promise<void>;
  cancelItem: (jobId: string, imageId: string) => Promise<void>;
  retryFailed: (jobId: string) => Promise<void>;
}

export function useAiBatchQueue(): BatchQueueState {
  const [jobs, setJobs] = useState<BatchJob[]>([]);
  const [activeJobId, setActiveJobId] = useState<string | null>(null);
  const [activeProgress, setActiveProgress] = useState<{
    completed: number;
    total: number;
  } | null>(null);
  const [activeEtaMs, setActiveEtaMs] = useState<number | null>(null);
  const [lastCompletion, setLastCompletion] =
    useState<BatchCompletionSummary | null>(null);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    try {
      const allJobs = await getBatchJobs();
      setJobs(allJobs);
      const running = allJobs.find((j) => j.status === "running");
      setActiveJobId(running?.id ?? null);
      if (!running) {
        setActiveProgress(null);
        setActiveEtaMs(null);
      }
    } catch (e) {
      console.error("Failed to fetch batch jobs:", e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  useEffect(() => {
    let cancelled = false;
    const unlisteners: (() => void)[] = [];

    const setup = async () => {
      const u1 = await listen<BatchJobStartedPayload>(
        "ai_batch:job_started",
        (e) => {
          if (cancelled) return;
          setActiveJobId(e.payload.jobId);
          setActiveProgress({ completed: 0, total: e.payload.totalItems });
          setActiveEtaMs(null);
          refresh();
        }
      );

      const u2 = await listen<BatchItemProgressPayload>(
        "ai_batch:item_progress",
        (e) => {
          if (cancelled) return;
          setActiveProgress({
            completed: e.payload.completed,
            total: e.payload.total,
          });
          setActiveEtaMs(e.payload.etaRemainingMs ?? null);
        }
      );

      const u3 = await listen<BatchJobCompletedPayload>(
        "ai_batch:job_completed",
        (e) => {
          if (cancelled) return;
          setLastCompletion(e.payload.summary);
          setActiveJobId(null);
          setActiveProgress(null);
          setActiveEtaMs(null);
          refresh();
        }
      );

      const u4 = await listen<{ jobId: string }>(
        "ai_batch:job_cancelled",
        () => {
          if (cancelled) return;
          setActiveJobId(null);
          setActiveProgress(null);
          setActiveEtaMs(null);
          refresh();
        }
      );

      if (cancelled) {
        [u1, u2, u3, u4].forEach((u) => u());
      } else {
        unlisteners.push(u1, u2, u3, u4);
      }
    };

    setup();
    return () => {
      cancelled = true;
      unlisteners.forEach((u) => u());
    };
  }, [refresh]);

  const handleCancelJob = useCallback(
    async (jobId: string) => {
      await cancelBatchJob(jobId);
      refresh();
    },
    [refresh]
  );

  const handleCancelItem = useCallback(
    async (jobId: string, imageId: string) => {
      await cancelBatchItem(jobId, imageId);
      refresh();
    },
    [refresh]
  );

  const handleRetryFailed = useCallback(
    async (jobId: string) => {
      await retryBatchFailed(jobId);
      refresh();
    },
    [refresh]
  );

  return {
    jobs,
    activeJobId,
    activeProgress,
    activeEtaMs,
    lastCompletion,
    loading,
    refresh,
    cancelJob: handleCancelJob,
    cancelItem: handleCancelItem,
    retryFailed: handleRetryFailed,
  };
}
