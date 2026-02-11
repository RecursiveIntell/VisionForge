import { useState } from "react";
import { ChevronDown, ChevronUp } from "lucide-react";
import { StageCard } from "./StageCard";
import { JudgeRanking } from "./JudgeRanking";
import type { PipelineResult, PipelineConfig } from "../../types";
import type { PipelinePhase } from "../../hooks/usePipeline";

interface PipelineStepperProps {
  config: PipelineConfig;
  result: PipelineResult | null;
  phase: PipelinePhase;
  selectedConcept: number;
  onSelectConcept: (index: number) => void;
}

type StageName = "ideator" | "composer" | "judge" | "promptEngineer" | "reviewer";

const stageLabels: Record<StageName, string> = {
  ideator: "1. Ideator",
  composer: "2. Composer",
  judge: "3. Judge",
  promptEngineer: "4. Prompt Engineer",
  reviewer: "5. Reviewer",
};

const stageConfigKeys: StageName[] = [
  "ideator",
  "composer",
  "judge",
  "promptEngineer",
  "reviewer",
];

function getStageEnabled(config: PipelineConfig, stage: StageName): boolean {
  const idx = stageConfigKeys.indexOf(stage);
  return config.stagesEnabled[idx] ?? true;
}

function getStageStatus(
  stage: StageName,
  result: PipelineResult | null,
  phase: PipelinePhase,
  enabled: boolean,
): "pending" | "running" | "completed" | "skipped" | "error" {
  if (!enabled) return "skipped";
  if (phase === "idle") return "pending";
  if (phase === "error") {
    // If we have output for this stage, it completed before the error
    const stageData = result?.stages?.[stage];
    if (stageData) return "completed";
    return "error";
  }
  if (phase === "completed") {
    const stageData = result?.stages?.[stage];
    return stageData ? "completed" : "skipped";
  }
  // running — check if this stage has output yet
  const stageData = result?.stages?.[stage];
  if (stageData) return "completed";
  return "running";
}

export function PipelineStepper({
  config,
  result,
  phase,
  selectedConcept,
  onSelectConcept,
}: PipelineStepperProps) {
  return (
    <div className="space-y-3">
      {stageConfigKeys.map((stage) => {
        const enabled = getStageEnabled(config, stage);
        const status = getStageStatus(stage, result, phase, enabled);
        const stageData = result?.stages?.[stage];
        const model = stageData
          ? "model" in stageData
            ? (stageData as { model?: string }).model
            : undefined
          : config.modelsUsed[stage];
        const durationMs = stageData
          ? "durationMs" in stageData
            ? (stageData as { durationMs?: number }).durationMs
            : undefined
          : undefined;

        return (
          <StageCard
            key={stage}
            name={stageLabels[stage]}
            enabled={enabled}
            status={status}
            model={model}
            durationMs={durationMs}
          >
            {stageData && (
              <StageOutput
                stage={stage}
                result={result!}
                selectedConcept={selectedConcept}
                onSelectConcept={onSelectConcept}
              />
            )}
          </StageCard>
        );
      })}
    </div>
  );
}

function StageOutput({
  stage,
  result,
  selectedConcept,
  onSelectConcept,
}: {
  stage: StageName;
  result: PipelineResult;
  selectedConcept: number;
  onSelectConcept: (index: number) => void;
}) {
  const [expanded, setExpanded] = useState(false);
  const stages = result.stages;

  if (stage === "ideator" && stages.ideator) {
    const concepts = stages.ideator.output;
    return (
      <ExpandableOutput expanded={expanded} onToggle={setExpanded}>
        <ol className="list-decimal list-inside space-y-1">
          {concepts.map((c, i) => (
            <li key={i} className="text-xs text-zinc-300">
              {c}
            </li>
          ))}
        </ol>
      </ExpandableOutput>
    );
  }

  if (stage === "composer" && stages.composer) {
    return (
      <ExpandableOutput expanded={expanded} onToggle={setExpanded}>
        <p className="text-xs text-zinc-300 whitespace-pre-wrap">
          {stages.composer.output}
        </p>
      </ExpandableOutput>
    );
  }

  if (stage === "judge" && stages.judge) {
    const concepts = stages.ideator?.output ?? stages.judge.input;
    return (
      <JudgeRanking
        rankings={stages.judge.output}
        concepts={concepts}
        selectedIndex={selectedConcept}
        onSelect={onSelectConcept}
      />
    );
  }

  if (stage === "promptEngineer" && stages.promptEngineer) {
    const { positive, negative } = stages.promptEngineer.output;
    return (
      <ExpandableOutput expanded={expanded} onToggle={setExpanded}>
        <div className="space-y-2">
          <div>
            <span className="text-xs font-medium text-green-400">
              Positive:
            </span>
            <p className="text-xs text-zinc-300 font-mono mt-0.5">
              {positive}
            </p>
          </div>
          <div>
            <span className="text-xs font-medium text-red-400">Negative:</span>
            <p className="text-xs text-zinc-300 font-mono mt-0.5">
              {negative}
            </p>
          </div>
        </div>
      </ExpandableOutput>
    );
  }

  if (stage === "reviewer" && stages.reviewer) {
    const r = stages.reviewer;
    return (
      <div
        className={`text-xs rounded px-2 py-1.5 ${
          r.approved
            ? "text-green-400 bg-green-400/10"
            : "text-amber-400 bg-amber-400/10"
        }`}
      >
        {r.approved ? "Approved" : "Issues found"}
        {r.issues && r.issues.length > 0 && (
          <ul className="list-disc list-inside mt-1">
            {r.issues.map((issue, i) => (
              <li key={i}>{issue}</li>
            ))}
          </ul>
        )}
      </div>
    );
  }

  return null;
}

function ExpandableOutput({
  expanded,
  onToggle,
  children,
}: {
  expanded: boolean;
  onToggle: (v: boolean) => void;
  children: React.ReactNode;
}) {
  return (
    <div>
      <button
        onClick={() => onToggle(!expanded)}
        className="flex items-center gap-1 text-xs text-zinc-500 hover:text-zinc-300 mb-1"
      >
        {expanded ? <ChevronUp size={12} /> : <ChevronDown size={12} />}
        {expanded ? "Collapse" : "Expand output"}
      </button>
      {expanded && <div className="mt-1">{children}</div>}
    </div>
  );
}
