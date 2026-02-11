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

export function useQueue() {
  const [jobs, setJobs] = useState<QueueJob[]>([]);
  const [paused, setPaused] = useState(false);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

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
      const u1 = await listen<JobEvent>("queue:job-started", () => refresh());
      const u2 = await listen<JobEvent>("queue:job-completed", () => refresh());
      const u3 = await listen<JobEvent>("queue:job-failed", () => refresh());
      unlisteners.push(u1, u2, u3);
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

  return { jobs, paused, loading, error, refresh, togglePause, cancel, reorder };
}
