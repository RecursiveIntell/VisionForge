import { useState, useEffect, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import {
  getQueue,
  pauseQueue,
  resumeQueue,
  isQueuePaused,
  cancelQueueJob,
  reorderQueue,
} from "../api/queue";
import type { QueueJob, QueuePriority } from "../types";

interface JobEvent {
  jobId: string;
}

interface JobProgressEvent {
  jobId: string;
  currentStep: number;
  totalSteps: number;
  progress: number;
}

export interface JobProgress {
  currentStep: number;
  totalSteps: number;
  progress: number;
}

export function useQueue() {
  const [jobs, setJobs] = useState<QueueJob[]>([]);
  const [paused, setPaused] = useState(false);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [progressMap, setProgressMap] = useState<Record<string, JobProgress>>({});

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [queue, pauseState] = await Promise.all([
        getQueue(),
        isQueuePaused(),
      ]);
      setJobs(queue);
      setPaused(pauseState);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load queue");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  // Subscribe to Tauri events for live updates
  useEffect(() => {
    const unlisteners: (() => void)[] = [];

    const setup = async () => {
      const u1 = await listen<JobEvent>("queue:job_started", () => refresh());
      const u2 = await listen<JobEvent>("queue:job_completed", (e) => {
        setProgressMap((prev) => {
          const next = { ...prev };
          delete next[e.payload.jobId];
          return next;
        });
        refresh();
      });
      const u3 = await listen<JobEvent>("queue:job_failed", (e) => {
        setProgressMap((prev) => {
          const next = { ...prev };
          delete next[e.payload.jobId];
          return next;
        });
        refresh();
      });
      const u5 = await listen<JobEvent>("queue:job_cancelled", (e) => {
        setProgressMap((prev) => {
          const next = { ...prev };
          delete next[e.payload.jobId];
          return next;
        });
        refresh();
      });
      const u4 = await listen<JobProgressEvent>("queue:job_progress", (e) => {
        setProgressMap((prev) => ({
          ...prev,
          [e.payload.jobId]: {
            currentStep: e.payload.currentStep,
            totalSteps: e.payload.totalSteps,
            progress: e.payload.progress,
          },
        }));
      });
      unlisteners.push(u1, u2, u3, u4, u5);
    };

    setup();
    return () => unlisteners.forEach((u) => u());
  }, [refresh]);

  const togglePause = useCallback(async () => {
    try {
      if (paused) {
        await resumeQueue();
      } else {
        await pauseQueue();
      }
      setPaused(!paused);
    } catch (e) {
      console.error("Failed to toggle pause:", e);
    }
  }, [paused]);

  const cancel = useCallback(
    async (jobId: string) => {
      try {
        await cancelQueueJob(jobId);
        refresh();
      } catch (e) {
        console.error("Failed to cancel job:", e);
      }
    },
    [refresh],
  );

  const reorder = useCallback(
    async (jobId: string, newPriority: QueuePriority) => {
      try {
        await reorderQueue(jobId, newPriority);
        refresh();
      } catch (e) {
        console.error("Failed to reorder:", e);
      }
    },
    [refresh],
  );

  return { jobs, paused, loading, error, refresh, togglePause, cancel, reorder, progressMap };
}
