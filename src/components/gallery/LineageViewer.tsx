import { useState, useEffect } from "react";
import { GitBranch, ChevronDown, ChevronUp } from "lucide-react";
import { getImageLineage } from "../../api/gallery";
import type { PipelineResult } from "../../types";

interface LineageViewerProps {
  imageId: string;
}

export function LineageViewer({ imageId }: LineageViewerProps) {
  const [lineage, setLineage] = useState<PipelineResult | null>(null);
  const [loading, setLoading] = useState(true);
  const [expanded, setExpanded] = useState(false);

  useEffect(() => {
    setLoading(true);
    getImageLineage(imageId)
      .then((raw) => {
        if (raw) {
          try {
            setLineage(JSON.parse(raw));
          } catch {
            setLineage(null);
          }
        } else {
          setLineage(null);
        }
      })
      .catch(() => setLineage(null))
      .finally(() => setLoading(false));
  }, [imageId]);

  if (loading) {
    return (
      <div className="text-xs text-zinc-500 py-2">Loading lineage...</div>
    );
  }

  if (!lineage) {
    return (
      <div className="text-xs text-zinc-500 py-2">
        No pipeline lineage available.
      </div>
    );
  }

  return (
    <div className="border border-zinc-700 rounded-lg overflow-hidden">
      <button
        onClick={() => setExpanded(!expanded)}
        className="w-full flex items-center gap-2 px-3 py-2 bg-zinc-800 hover:bg-zinc-700 text-sm text-zinc-300"
      >
        <GitBranch size={14} className="text-zinc-500" />
        <span>Pipeline Lineage</span>
        <span className="ml-auto text-zinc-500">
          {expanded ? <ChevronUp size={14} /> : <ChevronDown size={14} />}
        </span>
      </button>

      {expanded && (
        <div className="p-3 bg-zinc-800/50 space-y-3 text-xs">
          <div>
            <span className="text-zinc-500">Original Idea:</span>
            <p className="text-zinc-300 mt-0.5">{lineage.originalIdea}</p>
          </div>

          {lineage.stages.ideator && (
            <StageSection
              title="Ideator"
              model={lineage.stages.ideator.model}
              duration={lineage.stages.ideator.durationMs}
            >
              <ol className="list-decimal list-inside space-y-0.5">
                {lineage.stages.ideator.output.map((c, i) => (
                  <li key={i} className="text-zinc-300">
                    {c}
                  </li>
                ))}
              </ol>
            </StageSection>
          )}

          {lineage.stages.composer && (
            <StageSection
              title="Composer"
              model={lineage.stages.composer.model}
              duration={lineage.stages.composer.durationMs}
            >
              <p className="text-zinc-300 whitespace-pre-wrap">
                {lineage.stages.composer.output}
              </p>
            </StageSection>
          )}

          {lineage.stages.judge && (
            <StageSection
              title="Judge"
              model={lineage.stages.judge.model}
              duration={lineage.stages.judge.durationMs}
            >
              {lineage.stages.judge.output.map((r, i) => (
                <div key={i} className="flex gap-2">
                  <span className="text-zinc-500">#{r.rank}</span>
                  <span className="text-zinc-300">
                    Score: {r.score} — {r.reasoning}
                  </span>
                </div>
              ))}
            </StageSection>
          )}

          {lineage.stages.promptEngineer && (
            <StageSection
              title="Prompt Engineer"
              model={lineage.stages.promptEngineer.model}
              duration={lineage.stages.promptEngineer.durationMs}
            >
              <div className="space-y-1">
                <div>
                  <span className="text-green-400">+</span>{" "}
                  <span className="text-zinc-300 font-mono">
                    {lineage.stages.promptEngineer.output.positive}
                  </span>
                </div>
                <div>
                  <span className="text-red-400">-</span>{" "}
                  <span className="text-zinc-300 font-mono">
                    {lineage.stages.promptEngineer.output.negative}
                  </span>
                </div>
              </div>
            </StageSection>
          )}

          {lineage.stages.reviewer && (
            <StageSection
              title="Reviewer"
              model={lineage.stages.reviewer.model}
              duration={lineage.stages.reviewer.durationMs}
            >
              <span
                className={
                  lineage.stages.reviewer.approved
                    ? "text-green-400"
                    : "text-amber-400"
                }
              >
                {lineage.stages.reviewer.approved
                  ? "Approved"
                  : "Issues found"}
              </span>
            </StageSection>
          )}

          {lineage.autoApproved && (
            <div className="text-amber-400 text-[10px]">
              Auto-approved (no manual review)
            </div>
          )}
        </div>
      )}
    </div>
  );
}

function StageSection({
  title,
  model,
  duration,
  children,
}: {
  title: string;
  model: string;
  duration: number;
  children: React.ReactNode;
}) {
  return (
    <div className="border-l-2 border-zinc-600 pl-2">
      <div className="flex items-center gap-2 mb-1">
        <span className="font-medium text-zinc-200">{title}</span>
        <span className="text-zinc-500">{model}</span>
        <span className="text-zinc-600">
          {(duration / 1000).toFixed(1)}s
        </span>
      </div>
      {children}
    </div>
  );
}
