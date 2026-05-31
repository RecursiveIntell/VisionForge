import { Tag, Type, ChevronUp } from "lucide-react";
import type { BatchQueueState } from "../../hooks/useAiBatchQueue";

interface BatchStatusBarProps {
  batchState: BatchQueueState;
  onExpand: () => void;
}

export function BatchStatusBar({ batchState, onExpand }: BatchStatusBarProps) {
  const { activeJobId, activeProgress, activeEtaMs, jobs } = batchState;

  const activeJob = activeJobId
    ? jobs.find((j) => j.id === activeJobId)
    : null;

  const queuedCount = jobs.filter((j) => j.status === "queued").length;

  if (!activeJob && queuedCount === 0) return null;

  const icon =
    activeJob?.op === "caption" ? <Type size={12} /> : <Tag size={12} />;

  const formatEta = (ms: number) => {
    const seconds = Math.round(ms / 1000);
    if (seconds < 60) return `~${seconds}s`;
    const minutes = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `~${minutes}m ${secs}s`;
  };

  return (
    <div
      onClick={onExpand}
      className="flex items-center gap-3 px-4 py-1.5 bg-zinc-800 border-t border-zinc-700 cursor-pointer hover:bg-zinc-750 text-xs"
    >
      {activeJob && activeProgress ? (
        <>
          <span className="text-purple-400 flex items-center gap-1">
            {icon}
            {activeJob.op === "caption" ? "Captioning" : "Tagging"}
          </span>
          <span className="text-zinc-300">
            {activeProgress.completed}/{activeProgress.total}
          </span>
          <div className="flex-1 max-w-48 h-1.5 bg-zinc-700 rounded-full overflow-hidden">
            <div
              className="h-full bg-purple-500 rounded-full transition-all duration-300"
              style={{
                width: `${(activeProgress.completed / activeProgress.total) * 100}%`,
              }}
            />
          </div>
          {activeEtaMs != null && activeEtaMs > 0 && (
            <span className="text-zinc-500">{formatEta(activeEtaMs)}</span>
          )}
          {queuedCount > 0 && (
            <span className="text-zinc-600">+{queuedCount} queued</span>
          )}
        </>
      ) : queuedCount > 0 ? (
        <span className="text-zinc-400">
          {queuedCount} batch job{queuedCount !== 1 ? "s" : ""} queued
        </span>
      ) : null}

      <ChevronUp size={12} className="text-zinc-500 ml-auto" />
    </div>
  );
}
