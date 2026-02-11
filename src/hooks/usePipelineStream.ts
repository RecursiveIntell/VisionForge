import { useState, useCallback, useRef, useEffect } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { runFullPipeline, type RunPipelineInput } from "../api/pipeline";
import type { PipelineResult } from "../types";

export type PipelinePhase = "idle" | "running" | "completed" | "error";
export type StageName =
  | "ideator"
  | "composer"
  | "judge"
  | "promptEngineer"
  | "reviewer";

export interface StageStream {
  status: "pending" | "streaming" | "completed" | "skipped" | "error";
  model: string;
  tokens: string;
  durationMs?: number;
}

export type StageStreams = Record<StageName, StageStream>;

const STAGE_NAMES: StageName[] = [
  "ideator",
  "composer",
  "judge",
  "promptEngineer",
  "reviewer",
];

function createInitialStreams(): StageStreams {
  const streams = {} as StageStreams;
  for (const name of STAGE_NAMES) {
    streams[name] = { status: "pending", model: "", tokens: "" };
  }
  return streams;
}

export function usePipelineStream() {
  const [result, setResult] = useState<PipelineResult | null>(null);
  const [phase, setPhase] = useState<PipelinePhase>("idle");
  const [error, setError] = useState<string | null>(null);
  const [streams, setStreams] = useState<StageStreams>(createInitialStreams);
  const [activeStage, setActiveStage] = useState<StageName | null>(null);

  const tokenBufferRef = useRef<Record<string, string>>({});
  const flushTimerRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const unlistenersRef = useRef<UnlistenFn[]>([]);

  const startFlushing = useCallback(() => {
    if (flushTimerRef.current) return;
    flushTimerRef.current = setInterval(() => {
      const buffer = tokenBufferRef.current;
      const stages = Object.keys(buffer);
      if (stages.length === 0) return;

      const snapshot: Record<string, string> = {};
      for (const stage of stages) {
        if (buffer[stage]) {
          snapshot[stage] = buffer[stage];
          buffer[stage] = "";
        }
      }

      if (Object.keys(snapshot).length > 0) {
        setStreams((prev) => {
          const next = { ...prev };
          for (const [stage, newTokens] of Object.entries(snapshot)) {
            const s = stage as StageName;
            next[s] = {
              ...next[s],
              tokens: next[s].tokens + newTokens,
            };
          }
          return next;
        });
      }
    }, 33); // ~30fps
  }, []);

  const stopFlushing = useCallback(() => {
    if (flushTimerRef.current) {
      clearInterval(flushTimerRef.current);
      flushTimerRef.current = null;
    }
    // Final flush of any remaining buffer (entries already deleted by
    // stage_complete won't appear here, so no double-flush)
    const buffer = tokenBufferRef.current;
    const snapshot: Record<string, string> = {};
    for (const [stage, tokens] of Object.entries(buffer)) {
      if (tokens) {
        snapshot[stage] = tokens;
      }
    }
    tokenBufferRef.current = {};
    if (Object.keys(snapshot).length > 0) {
      setStreams((prev) => {
        const next = { ...prev };
        for (const [stage, newTokens] of Object.entries(snapshot)) {
          const s = stage as StageName;
          next[s] = { ...next[s], tokens: next[s].tokens + newTokens };
        }
        return next;
      });
    }
  }, []);

  const cleanup = useCallback(async () => {
    stopFlushing();
    for (const unlisten of unlistenersRef.current) {
      unlisten();
    }
    unlistenersRef.current = [];
  }, [stopFlushing]);

  useEffect(() => {
    return () => {
      cleanup();
    };
  }, [cleanup]);

  const run = useCallback(
    async (input: RunPipelineInput) => {
      setPhase("running");
      setError(null);
      setResult(null);
      setStreams(createInitialStreams());
      setActiveStage(null);
      tokenBufferRef.current = {};

      await cleanup();

      // Set up event listeners
      const unlistenStart = await listen<{ stage: string; model: string }>(
        "pipeline:stage_start",
        (event) => {
          const { stage, model } = event.payload;
          const s = stage as StageName;
          setActiveStage(s);
          setStreams((prev) => ({
            ...prev,
            [s]: { ...prev[s], status: "streaming", model },
          }));
        },
      );

      const unlistenToken = await listen<{ stage: string; token: string }>(
        "pipeline:stage_token",
        (event) => {
          const { stage, token } = event.payload;
          if (!tokenBufferRef.current[stage]) {
            tokenBufferRef.current[stage] = "";
          }
          tokenBufferRef.current[stage] += token;
        },
      );

      const unlistenComplete = await listen<{
        stage: string;
        durationMs: number;
      }>("pipeline:stage_complete", (event) => {
        const { stage, durationMs } = event.payload;
        const s = stage as StageName;
        // Atomically capture and clear remaining buffered tokens for this stage.
        // Safe in JS single-threaded model: interval cannot fire mid-handler.
        const remaining = tokenBufferRef.current[s] || "";
        delete tokenBufferRef.current[s];
        setStreams((prev) => ({
          ...prev,
          [s]: {
            ...prev[s],
            status: "completed",
            durationMs,
            tokens: prev[s].tokens + remaining,
          },
        }));
      });

      unlistenersRef.current = [unlistenStart, unlistenToken, unlistenComplete];
      startFlushing();

      try {
        const res = await runFullPipeline(input);
        stopFlushing();
        setResult(res);
        setPhase("completed");
        setActiveStage(null);
      } catch (e) {
        stopFlushing();
        const msg = e instanceof Error ? e.message : String(e);
        setError(msg);
        setPhase("error");
        setActiveStage(null);
      } finally {
        await cleanup();
      }
    },
    [cleanup, startFlushing, stopFlushing],
  );

  const reset = useCallback(() => {
    setResult(null);
    setPhase("idle");
    setError(null);
    setStreams(createInitialStreams());
    setActiveStage(null);
    tokenBufferRef.current = {};
  }, []);

  return { result, phase, error, streams, activeStage, run, reset };
}
