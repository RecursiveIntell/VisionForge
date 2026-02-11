import { useState } from "react";
import { Trophy, ChevronDown, ChevronUp } from "lucide-react";
import type { JudgeRanking as JudgeRankingType } from "../../types";

interface JudgeRankingProps {
  rankings: JudgeRankingType[];
  concepts: string[];
  selectedIndex: number;
  onSelect: (conceptIndex: number) => void;
}

export function JudgeRanking({
  rankings,
  concepts,
  selectedIndex,
  onSelect,
}: JudgeRankingProps) {
  const [expandedRank, setExpandedRank] = useState<number | null>(null);

  const toggleExpand = (rank: number) => {
    setExpandedRank(expandedRank === rank ? null : rank);
  };

  return (
    <div className="space-y-2">
      <p className="text-xs text-zinc-500 mb-2">
        Click a concept to select it for prompt engineering.
      </p>
      {rankings.map((r) => {
        const isSelected = r.conceptIndex === selectedIndex;
        const isExpanded = expandedRank === r.rank;
        const concept = concepts[r.conceptIndex] ?? "Unknown concept";

        return (
          <div
            key={r.rank}
            className={`border rounded-lg overflow-hidden cursor-pointer transition-colors ${
              isSelected
                ? "border-blue-500 bg-zinc-700/50"
                : "border-zinc-700 bg-zinc-800 hover:border-zinc-600"
            }`}
          >
            <div
              className="flex items-center gap-3 p-3"
              onClick={() => onSelect(r.conceptIndex)}
            >
              <div className="flex items-center gap-1.5 shrink-0">
                {r.rank === 1 && (
                  <Trophy size={14} className="text-amber-400" />
                )}
                <span className="text-xs font-medium text-zinc-400">
                  #{r.rank}
                </span>
              </div>
              <div className="flex-1 min-w-0">
                <p className="text-sm text-zinc-200 truncate">{concept}</p>
              </div>
              <div className="flex items-center gap-2 shrink-0">
                <ScoreBadge score={r.score} />
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    toggleExpand(r.rank);
                  }}
                  className="p-1 text-zinc-500 hover:text-zinc-300"
                >
                  {isExpanded ? (
                    <ChevronUp size={14} />
                  ) : (
                    <ChevronDown size={14} />
                  )}
                </button>
              </div>
            </div>
            {isExpanded && (
              <div className="px-3 pb-3 border-t border-zinc-700">
                <p className="text-xs text-zinc-400 mt-2">{r.reasoning}</p>
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}

function ScoreBadge({ score }: { score: number }) {
  let color = "text-zinc-400";
  if (score >= 80) color = "text-green-400";
  else if (score >= 60) color = "text-amber-400";
  else if (score >= 40) color = "text-zinc-300";
  else color = "text-red-400";

  return (
    <span className={`text-sm font-mono font-medium ${color}`}>{score}</span>
  );
}
