import { useState, useEffect, useRef } from "react";
import {
  CheckCircle,
  AlertCircle,
  ChevronDown,
  Loader2,
  SkipForward,
  Cpu,
} from "lucide-react";
import { JudgeRanking } from "./JudgeRanking";
import type { PipelineResult, PipelineConfig } from "../../types";
import type {
  StageName,
  StageStreams,
} from "../../hooks/usePipelineStream";

interface StreamingStepperProps {
  config: PipelineConfig;
  result: PipelineResult | null;
  streams: StageStreams;
  activeStage: StageName | null;
  selectedConcept: number;
  onSelectConcept: (index: number) => void;
}

const STAGES: { key: StageName; label: string; num: number }[] = [
  { key: "ideator", label: "Ideator", num: 1 },
  { key: "composer", label: "Composer", num: 2 },
  { key: "judge", label: "Judge", num: 3 },
  { key: "promptEngineer", label: "Prompt Engineer", num: 4 },
  { key: "reviewer", label: "Reviewer", num: 5 },
];

function isStageEnabled(config: PipelineConfig, stage: StageName): boolean {
  const idx = STAGES.findIndex((s) => s.key === stage);
  return config.stagesEnabled[idx] ?? true;
}

export function StreamingStepper({
  config,
  result,
  streams,
  activeStage,
  selectedConcept,
  onSelectConcept,
}: StreamingStepperProps) {
  const [manualOpen, setManualOpen] = useState<Record<string, boolean>>({});
  const prevActiveRef = useRef<StageName | null>(null);

  // Auto-open the active stage and auto-close the previous one
  useEffect(() => {
    if (activeStage) {
      setManualOpen((prev) => {
        const next = { ...prev, [activeStage]: true };
        if (prevActiveRef.current && prevActiveRef.current !== activeStage) {
          next[prevActiveRef.current] = false;
        }
        return next;
      });
      prevActiveRef.current = activeStage;
    } else if (prevActiveRef.current) {
      // Pipeline finished — collapse the last active stage
      const last = prevActiveRef.current;
      setManualOpen((prev) => ({ ...prev, [last]: false }));
      prevActiveRef.current = null;
    }
  }, [activeStage]);

  return (
    <div className="space-y-3">
      {STAGES.map(({ key, label, num }) => {
        const enabled = isStageEnabled(config, key);
        if (!enabled) {
          return (
            <div
              key={key}
              className="flex items-center gap-3 px-4 py-2 bg-zinc-800 border border-zinc-800 rounded-lg opacity-40"
            >
              <StageNumber status="pending" num={num} />
              <span className="text-sm text-zinc-400">{label}</span>
              <SkipForward size={14} className="text-zinc-500 ml-auto" />
            </div>
          );
        }

        const stream = streams[key];
        const isOpen = manualOpen[key] ?? false;
        const hasContent =
          stream.tokens.length > 0 || stream.status === "completed";
        const canToggle = hasContent;

        return (
          <StreamingStageCard
            key={key}
            stageName={key}
            label={label}
            num={num}
            stream={stream}
            isOpen={isOpen}
            canToggle={canToggle}
            onToggle={() =>
              setManualOpen((prev) => ({ ...prev, [key]: !prev[key] }))
            }
            result={result}
            selectedConcept={selectedConcept}
            onSelectConcept={onSelectConcept}
          />
        );
      })}
    </div>
  );
}

function StreamingStageCard({
  stageName,
  label,
  num,
  stream,
  isOpen,
  canToggle,
  onToggle,
  result,
  selectedConcept,
  onSelectConcept,
}: {
  stageName: StageName;
  label: string;
  num: number;
  stream: StageStreams[StageName];
  isOpen: boolean;
  canToggle: boolean;
  onToggle: () => void;
  result: PipelineResult | null;
  selectedConcept: number;
  onSelectConcept: (index: number) => void;
}) {
  const borderColor =
    stream.status === "streaming"
      ? "border-blue-500/40"
      : stream.status === "completed"
        ? "border-zinc-700"
        : stream.status === "error"
          ? "border-red-500/40"
          : "border-zinc-800";

  const statusText =
    stream.status === "streaming"
      ? "Streaming..."
      : stream.status === "completed"
        ? stream.durationMs
          ? `${(stream.durationMs / 1000).toFixed(1)}s`
          : "Done"
        : stream.status === "error"
          ? "Error"
          : "";

  return (
    <div
      className={`bg-zinc-800 border ${borderColor} rounded-lg overflow-hidden`}
    >
      <button
        onClick={canToggle ? onToggle : undefined}
        className={`w-full flex items-center gap-3 px-4 py-3 text-left ${canToggle ? "cursor-pointer hover:bg-zinc-750" : "cursor-default"}`}
      >
        <StageNumber status={stream.status} num={num} />
        <span className="text-sm font-medium text-zinc-200 flex-1">
          {label}
        </span>
        {statusText && (
          <span className="text-xs text-zinc-400">{statusText}</span>
        )}
        {stream.model && (
          <span className="flex items-center gap-1 text-xs text-zinc-500">
            <Cpu size={12} />
            {stream.model}
          </span>
        )}
        {canToggle && (
          <ChevronDown
            size={16}
            className={`text-zinc-400 transition-transform duration-200 ${isOpen ? "rotate-180" : ""}`}
          />
        )}
      </button>
      <div
        className={`transition-all duration-300 overflow-hidden ${isOpen ? "max-h-[400px]" : "max-h-0"}`}
      >
        <div className="px-4 pb-3 border-t border-zinc-700/50">
          {stream.status === "completed" && result?.stages?.[stageName] ? (
            <ParsedStageOutput
              stageName={stageName}
              result={result}
              selectedConcept={selectedConcept}
              onSelectConcept={onSelectConcept}
            />
          ) : (
            <StreamOutput
              tokens={stream.tokens}
              isStreaming={stream.status === "streaming"}
            />
          )}
        </div>
      </div>
    </div>
  );
}

function StageNumber({
  status,
  num,
}: {
  status: string;
  num: number;
}) {
  if (status === "streaming") {
    return (
      <div className="w-7 h-7 rounded-full bg-blue-500/20 flex items-center justify-center shrink-0">
        <Loader2 size={16} className="text-blue-400 animate-spin" />
      </div>
    );
  }
  if (status === "completed") {
    return (
      <div className="w-7 h-7 rounded-full bg-green-500/20 flex items-center justify-center shrink-0">
        <CheckCircle size={16} className="text-green-400" />
      </div>
    );
  }
  if (status === "error") {
    return (
      <div className="w-7 h-7 rounded-full bg-red-500/20 flex items-center justify-center shrink-0">
        <AlertCircle size={16} className="text-red-400" />
      </div>
    );
  }
  return (
    <div className="w-7 h-7 rounded-full bg-zinc-700 flex items-center justify-center shrink-0">
      <span className="text-xs font-medium text-zinc-400">{num}</span>
    </div>
  );
}

function StreamOutput({
  tokens,
  isStreaming,
}: {
  tokens: string;
  isStreaming: boolean;
}) {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (containerRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight;
    }
  }, [tokens]);

  return (
    <div
      ref={containerRef}
      className="mt-2 max-h-[350px] overflow-auto font-mono text-xs text-zinc-300 whitespace-pre-wrap break-words"
    >
      {tokens}
      {isStreaming && (
        <span className="inline-block w-1.5 h-3.5 bg-blue-400 ml-0.5 animate-pulse rounded-sm" />
      )}
    </div>
  );
}

function ParsedStageOutput({
  stageName,
  result,
  selectedConcept,
  onSelectConcept,
}: {
  stageName: StageName;
  result: PipelineResult;
  selectedConcept: number;
  onSelectConcept: (index: number) => void;
}) {
  const stages = result.stages;

  if (stageName === "ideator" && stages.ideator) {
    return (
      <ol className="list-decimal list-inside space-y-1 mt-2">
        {stages.ideator.output.map((c, i) => (
          <li key={i} className="text-xs text-zinc-300">
            {c}
          </li>
        ))}
      </ol>
    );
  }

  if (stageName === "composer" && stages.composer) {
    return (
      <p className="text-xs text-zinc-300 whitespace-pre-wrap mt-2">
        {stages.composer.output}
      </p>
    );
  }

  if (stageName === "judge" && stages.judge) {
    const concepts = stages.ideator?.output ?? stages.judge.input;
    return (
      <div className="mt-2">
        <JudgeRanking
          rankings={stages.judge.output}
          concepts={concepts}
          selectedIndex={selectedConcept}
          onSelect={onSelectConcept}
        />
      </div>
    );
  }

  if (stageName === "promptEngineer" && stages.promptEngineer) {
    const { positive, negative } = stages.promptEngineer.output;
    return (
      <div className="space-y-2 mt-2">
        <div>
          <span className="text-xs font-medium text-green-400">Positive:</span>
          <p className="text-xs text-zinc-300 font-mono mt-0.5">{positive}</p>
        </div>
        <div>
          <span className="text-xs font-medium text-red-400">Negative:</span>
          <p className="text-xs text-zinc-300 font-mono mt-0.5">{negative}</p>
        </div>
      </div>
    );
  }

  if (stageName === "reviewer" && stages.reviewer) {
    const r = stages.reviewer;
    return (
      <div
        className={`mt-2 text-xs rounded px-2 py-1.5 ${
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
