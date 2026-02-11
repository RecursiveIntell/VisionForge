import { useState, useEffect } from "react";
import { IdeaInput } from "./IdeaInput";
import { StreamingStepper } from "./StreamingStepper";
import { ApprovalGate } from "./ApprovalGate";
import { SeedPicker } from "../seeds/SeedPicker";
import { usePipelineStream } from "../../hooks/usePipelineStream";
import { useConfig } from "../../hooks/useConfig";
import { addToQueue } from "../../api/queue";
import { getComfyuiCheckpoints } from "../../api/comfyui";
import { useToast } from "../shared/Toast";
import type { PipelineConfig, QueueJob } from "../../types";

export function PromptStudio() {
  const { config, update: updateConfig } = useConfig();
  const { result, phase, error, streams, activeStage, run, reset } = usePipelineStream();
  const { addToast } = useToast();

  const [selectedConcept, setSelectedConcept] = useState(0);
  const [editedPositive, setEditedPositive] = useState("");
  const [editedNegative, setEditedNegative] = useState("");
  const [showSeedPicker, setShowSeedPicker] = useState(false);
  const [selectedSeed, setSelectedSeed] = useState<number>(-1);
  const [checkpoints, setCheckpoints] = useState<string[]>([]);
  const [selectedCheckpoint, setSelectedCheckpoint] = useState("");

  // Load available checkpoints (runs once on mount)
  useEffect(() => {
    let cancelled = false;
    getComfyuiCheckpoints()
      .then((cp) => {
        if (cancelled) return;
        setCheckpoints(cp);
        setSelectedCheckpoint((prev) => (prev || cp[0] || ""));
      })
      .catch(() => {});
    return () => { cancelled = true; };
  }, []);

  // Sync prompt editor when pipeline produces output
  useEffect(() => {
    if (result?.stages?.promptEngineer) {
      setEditedPositive(result.stages.promptEngineer.output.positive);
      setEditedNegative(result.stages.promptEngineer.output.negative);
    } else if (result?.stages?.reviewer) {
      if (result.stages.reviewer.suggestedPositive) {
        setEditedPositive(result.stages.reviewer.suggestedPositive);
      }
      if (result.stages.reviewer.suggestedNegative) {
        setEditedNegative(result.stages.reviewer.suggestedNegative);
      }
    }
  }, [result]);

  const handleSubmit = async (idea: string, numConcepts: number) => {
    setSelectedConcept(0);
    setEditedPositive("");
    setEditedNegative("");
    await run({
      idea,
      numConcepts,
      autoApprove: config?.pipeline.autoApprove ?? false,
      checkpointContext: selectedCheckpoint || undefined,
    });
  };

  const handleGenerate = async () => {
    if (!editedPositive.trim()) return;

    const settings = config?.presets?.["quality"] ?? {
      steps: 20,
      cfg: 7,
      width: 512,
      height: 512,
      sampler: "euler",
      scheduler: "normal",
    };

    const job: QueueJob = {
      id: "",
      priority: "normal",
      status: "pending",
      positivePrompt: editedPositive,
      negativePrompt: editedNegative,
      settingsJson: JSON.stringify({
        checkpoint: selectedCheckpoint,
        seed: selectedSeed,
        ...settings,
      }),
      pipelineLog: result ? JSON.stringify(result) : undefined,
      originalIdea: result?.originalIdea,
    };

    try {
      await addToQueue(job);
      addToast("success", "Added to generation queue");
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      addToast("error", `Failed to queue: ${msg}`);
    }
  };

  const handleRegenerate = () => {
    reset();
  };

  const handleAutoApproveChange = (value: boolean) => {
    if (config) {
      updateConfig({
        ...config,
        pipeline: { ...config.pipeline, autoApprove: value },
      });
    }
  };

  const pipelineConfig: PipelineConfig = config
    ? {
        stagesEnabled: [
          config.pipeline.enableIdeator,
          config.pipeline.enableComposer,
          config.pipeline.enableJudge,
          config.pipeline.enablePromptEngineer,
          config.pipeline.enableReviewer,
        ],
        modelsUsed: config.models,
      }
    : {
        stagesEnabled: [true, true, true, true, false],
        modelsUsed: {},
      };

  const isRunning = phase === "running";
  const showApproval =
    phase === "completed" && editedPositive.trim().length > 0;

  return (
    <div className="p-6 max-w-3xl mx-auto space-y-6">
      <IdeaInput onSubmit={handleSubmit} disabled={isRunning} />

      {phase !== "idle" && (
        <StreamingStepper
          config={pipelineConfig}
          result={result}
          streams={streams}
          activeStage={activeStage}
          selectedConcept={selectedConcept}
          onSelectConcept={setSelectedConcept}
        />
      )}

      {error && (
        <div className="bg-red-400/10 border border-red-400/20 rounded-lg p-3 text-sm text-red-400">
          {error}
        </div>
      )}

      {showApproval && (
        <div className="flex gap-3 items-end">
          {checkpoints.length > 0 && (
            <label className="flex-1">
              <span className="text-xs text-zinc-500 block mb-1">Checkpoint</span>
              <select
                value={selectedCheckpoint}
                onChange={(e) => setSelectedCheckpoint(e.target.value)}
                className="w-full bg-zinc-800 border border-zinc-700 rounded px-2 py-1.5 text-sm text-zinc-100 focus:border-blue-500 focus:outline-none"
              >
                {checkpoints.map((cp) => (
                  <option key={cp} value={cp}>{cp}</option>
                ))}
              </select>
            </label>
          )}
          <div>
            <span className="text-xs text-zinc-500 block mb-1">Seed</span>
            <div className="flex items-center gap-2">
              <span className="text-sm font-mono text-zinc-300">
                {selectedSeed === -1 ? "Random" : selectedSeed}
              </span>
              <button
                onClick={() => setShowSeedPicker(!showSeedPicker)}
                className="px-2 py-1 text-xs bg-zinc-700 hover:bg-zinc-600 text-zinc-300 rounded"
              >
                Pick Seed
              </button>
            </div>
          </div>
        </div>
      )}

      {showSeedPicker && (
        <SeedPicker
          onSelect={(seed) => {
            setSelectedSeed(seed);
            setShowSeedPicker(false);
          }}
          onClose={() => setShowSeedPicker(false)}
        />
      )}

      {showApproval && (
        <ApprovalGate
          positive={editedPositive}
          negative={editedNegative}
          onPositiveChange={setEditedPositive}
          onNegativeChange={setEditedNegative}
          autoApprove={config?.pipeline.autoApprove ?? false}
          onAutoApproveChange={handleAutoApproveChange}
          onGenerate={handleGenerate}
          onRegenerate={handleRegenerate}
          disabled={isRunning}
          reviewerApproved={result?.stages?.reviewer?.approved}
          reviewerIssues={result?.stages?.reviewer?.issues}
        />
      )}
    </div>
  );
}
