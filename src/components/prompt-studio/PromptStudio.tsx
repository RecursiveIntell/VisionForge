import { useState, useEffect } from "react";
import { IdeaInput } from "./IdeaInput";
import { StreamingStepper } from "./StreamingStepper";
import { ApprovalGate } from "./ApprovalGate";
import { GenerationControls, getDefaultSettings } from "./GenerationControls";
import { usePipelineStream } from "../../hooks/usePipelineStream";
import { useConfig } from "../../hooks/useConfig";
import { addToQueue } from "../../api/queue";
import { useToast } from "../shared/Toast";
import type { PipelineConfig, QueueJob, GenSettings } from "../../types";

export function PromptStudio() {
  const { config, update: updateConfig } = useConfig();
  const { result, phase, error, streams, activeStage, run, cancel, reset } = usePipelineStream();
  const { addToast } = useToast();

  const [selectedConcept, setSelectedConcept] = useState(0);
  const [editedPositive, setEditedPositive] = useState("");
  const [editedNegative, setEditedNegative] = useState("");
  const [genSettings, setGenSettings] = useState<GenSettings>(() => getDefaultSettings(config));

  // Re-initialize genSettings when config first loads
  useEffect(() => {
    if (config && !genSettings.checkpoint) {
      setGenSettings(getDefaultSettings(config));
    }
  }, [config]); // eslint-disable-line react-hooks/exhaustive-deps

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
      checkpointContext: genSettings.checkpoint || undefined,
    });
  };

  const handleGenerate = async () => {
    if (!editedPositive.trim()) return;

    const count = Math.max(1, genSettings.batchCount);

    for (let i = 0; i < count; i++) {
      let seed = genSettings.seed;
      if (seed !== -1) {
        seed = seed + i;
      }

      const job: QueueJob = {
        id: "",
        priority: "normal",
        status: "pending",
        positivePrompt: editedPositive,
        negativePrompt: editedNegative,
        settingsJson: JSON.stringify({
          checkpoint: genSettings.checkpoint,
          seed,
          steps: genSettings.steps,
          cfgScale: genSettings.cfg,
          width: genSettings.width,
          height: genSettings.height,
          sampler: genSettings.sampler,
          scheduler: genSettings.scheduler,
          batchSize: 1,
        }),
        pipelineLog: result ? JSON.stringify(result) : undefined,
        originalIdea: result?.originalIdea,
      };

      try {
        await addToQueue(job);
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        addToast("error", `Failed to queue: ${msg}`);
        return;
      }
    }

    addToast(
      "success",
      count > 1
        ? `Added ${count} jobs to generation queue`
        : "Added to generation queue",
    );
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

      {isRunning && (
        <div className="flex justify-end">
          <button
            onClick={cancel}
            className="px-4 py-2 text-sm bg-red-500/20 hover:bg-red-500/30 text-red-400 border border-red-500/30 rounded-lg transition-colors"
          >
            Cancel Pipeline
          </button>
        </div>
      )}

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

      {phase === "cancelled" && (
        <div className="bg-amber-400/10 border border-amber-400/20 rounded-lg p-3 text-sm text-amber-400 flex items-center justify-between">
          <span>Pipeline was cancelled.</span>
          <button
            onClick={reset}
            className="px-3 py-1 text-xs bg-zinc-700 hover:bg-zinc-600 rounded transition-colors"
          >
            Start Over
          </button>
        </div>
      )}

      {error && phase === "error" && (
        <div className="bg-red-400/10 border border-red-400/20 rounded-lg p-3 text-sm text-red-400">
          {error}
        </div>
      )}

      {showApproval && (
        <GenerationControls
          config={config}
          settings={genSettings}
          onChange={setGenSettings}
          disabled={isRunning}
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
