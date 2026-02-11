import { AlertTriangle, Play, RefreshCw, Layers } from "lucide-react";
import { PromptEditor } from "./PromptEditor";

interface ApprovalGateProps {
  positive: string;
  negative: string;
  onPositiveChange: (value: string) => void;
  onNegativeChange: (value: string) => void;
  autoApprove: boolean;
  onAutoApproveChange: (value: boolean) => void;
  onGenerate: () => void;
  onRegenerate: () => void;
  onBatch?: () => void;
  disabled?: boolean;
  reviewerApproved?: boolean;
  reviewerIssues?: string[];
}

export function ApprovalGate({
  positive,
  negative,
  onPositiveChange,
  onNegativeChange,
  autoApprove,
  onAutoApproveChange,
  onGenerate,
  onRegenerate,
  onBatch,
  disabled,
  reviewerApproved,
  reviewerIssues,
}: ApprovalGateProps) {
  return (
    <div className="bg-zinc-800 border border-zinc-700 rounded-lg p-4 space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-semibold text-zinc-200">Approval Gate</h3>
        {autoApprove && (
          <div className="flex items-center gap-1.5 text-amber-400 text-xs">
            <AlertTriangle size={14} />
            <span>Auto-approve is ON</span>
          </div>
        )}
      </div>

      {reviewerApproved !== undefined && (
        <ReviewerStatus
          approved={reviewerApproved}
          issues={reviewerIssues}
        />
      )}

      <PromptEditor
        positive={positive}
        negative={negative}
        onPositiveChange={onPositiveChange}
        onNegativeChange={onNegativeChange}
        disabled={disabled}
      />

      <div className="flex items-center justify-between pt-2 border-t border-zinc-700">
        <label className="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={autoApprove}
            onChange={(e) => onAutoApproveChange(e.target.checked)}
            className="rounded border-zinc-600 bg-zinc-700 text-blue-500 focus:ring-blue-500 focus:ring-offset-0"
          />
          <span className="text-sm text-zinc-400">Auto-approve</span>
        </label>
        <div className="flex items-center gap-2">
          <button
            onClick={onRegenerate}
            disabled={disabled}
            className="flex items-center gap-1.5 px-3 py-1.5 text-sm text-zinc-300 bg-zinc-700 hover:bg-zinc-600 disabled:opacity-50 disabled:cursor-not-allowed rounded"
          >
            <RefreshCw size={14} />
            Regenerate
          </button>
          {onBatch && (
            <button
              onClick={onBatch}
              disabled={disabled}
              className="flex items-center gap-1.5 px-3 py-1.5 text-sm text-zinc-300 bg-zinc-700 hover:bg-zinc-600 disabled:opacity-50 disabled:cursor-not-allowed rounded"
            >
              <Layers size={14} />
              Batch
            </button>
          )}
          <button
            onClick={onGenerate}
            disabled={disabled || !positive.trim()}
            className="flex items-center gap-1.5 px-4 py-1.5 text-sm text-white bg-blue-600 hover:bg-blue-500 disabled:bg-zinc-600 disabled:cursor-not-allowed rounded"
          >
            <Play size={14} />
            Generate
          </button>
        </div>
      </div>
    </div>
  );
}

function ReviewerStatus({
  approved,
  issues,
}: {
  approved: boolean;
  issues?: string[];
}) {
  if (approved) {
    return (
      <div className="text-xs text-green-400 bg-green-400/10 border border-green-400/20 rounded px-3 py-2">
        Reviewer approved this prompt.
      </div>
    );
  }

  return (
    <div className="text-xs text-amber-400 bg-amber-400/10 border border-amber-400/20 rounded px-3 py-2 space-y-1">
      <p className="font-medium">Reviewer flagged issues:</p>
      {issues && issues.length > 0 ? (
        <ul className="list-disc list-inside space-y-0.5">
          {issues.map((issue, i) => (
            <li key={i}>{issue}</li>
          ))}
        </ul>
      ) : (
        <p>Review did not pass. Consider editing the prompts.</p>
      )}
    </div>
  );
}
