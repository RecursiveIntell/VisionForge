import { useState, useCallback } from "react";
import { runFullPipeline, type RunPipelineInput } from "../api/pipeline";
import type { PipelineResult } from "../types";

export type PipelinePhase = "idle" | "running" | "completed" | "error";

export function usePipeline() {
  const [result, setResult] = useState<PipelineResult | null>(null);
  const [phase, setPhase] = useState<PipelinePhase>("idle");
  const [error, setError] = useState<string | null>(null);

  const run = useCallback(async (input: RunPipelineInput) => {
    setPhase("running");
    setError(null);
    setResult(null);
    try {
      const res = await runFullPipeline(input);
      setResult(res);
      setPhase("completed");
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
      setPhase("error");
    }
  }, []);

  const reset = useCallback(() => {
    setResult(null);
    setPhase("idle");
    setError(null);
  }, []);

  return { result, phase, error, run, reset };
}
