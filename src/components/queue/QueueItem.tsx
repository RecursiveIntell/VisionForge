import {
  Clock,
  Loader2,
  CheckCircle,
  XCircle,
  Ban,
  ChevronUp,
  ChevronDown,
  X,
} from "lucide-react";
import { ProgressBar } from "./ProgressBar";
import type { QueueJob, QueuePriority } from "../../types";

interface QueueItemProps {
  job: QueueJob;
  onCancel: () => void;
  onReorder: (priority: QueuePriority) => void;
}

const statusConfig = {
  pending: { icon: Clock, color: "text-zinc-400", label: "Pending" },
  generating: { icon: Loader2, color: "text-blue-400", label: "Generating" },
  completed: { icon: CheckCircle, color: "text-green-400", label: "Completed" },
  failed: { icon: XCircle, color: "text-red-400", label: "Failed" },
  cancelled: { icon: Ban, color: "text-zinc-500", label: "Cancelled" },
};

const priorityColors: Record<QueuePriority, string> = {
  high: "text-red-400 bg-red-400/10",
  normal: "text-zinc-400 bg-zinc-700",
  low: "text-zinc-500 bg-zinc-800",
};

export function QueueItem({ job, onCancel, onReorder }: QueueItemProps) {
  const status = statusConfig[job.status];
  const StatusIcon = status.icon;
  const isActive = job.status === "pending" || job.status === "generating";

  return (
    <div className="bg-zinc-800 border border-zinc-700 rounded-lg p-3">
      <div className="flex items-start gap-3">
        <StatusIcon
          size={16}
          className={`mt-0.5 ${status.color} ${
            job.status === "generating" ? "animate-spin" : ""
          }`}
        />

        <div className="flex-1 min-w-0">
          <p className="text-sm text-zinc-200 truncate">
            {job.positivePrompt}
          </p>
          {job.originalIdea && (
            <p className="text-xs text-zinc-500 truncate mt-0.5">
              Idea: {job.originalIdea}
            </p>
          )}

          <div className="flex items-center gap-2 mt-1.5">
            <span className={`text-[10px] px-1.5 py-0.5 rounded ${priorityColors[job.priority]}`}>
              {job.priority}
            </span>
            <span className="text-[10px] text-zinc-500">{status.label}</span>
            {job.createdAt && (
              <span className="text-[10px] text-zinc-600">
                {new Date(job.createdAt).toLocaleTimeString()}
              </span>
            )}
          </div>

          {job.status === "generating" && (
            <ProgressBar progress={50} className="mt-2" />
          )}
        </div>

        {isActive && (
          <div className="flex items-center gap-1 shrink-0">
            {job.status === "pending" && (
              <>
                <button
                  onClick={() => onReorder("high")}
                  className="p-1 text-zinc-500 hover:text-zinc-300"
                  title="Move to high priority"
                >
                  <ChevronUp size={14} />
                </button>
                <button
                  onClick={() => onReorder("low")}
                  className="p-1 text-zinc-500 hover:text-zinc-300"
                  title="Move to low priority"
                >
                  <ChevronDown size={14} />
                </button>
              </>
            )}
            <button
              onClick={onCancel}
              className="p-1 text-zinc-500 hover:text-red-400"
              title="Cancel job"
            >
              <X size={14} />
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
