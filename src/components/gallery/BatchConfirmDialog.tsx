import { useState, useEffect } from "react";
import { previewBatchJob } from "../../api/aiBatch";
import type { BatchOpKind, OverwritePolicy, BatchPreview } from "../../types";

interface BatchConfirmDialogProps {
  open: boolean;
  op: BatchOpKind;
  imageIds: string[];
  onConfirm: (overwritePolicy: OverwritePolicy) => void;
  onCancel: () => void;
}

export function BatchConfirmDialog({
  open,
  op,
  imageIds,
  onConfirm,
  onCancel,
}: BatchConfirmDialogProps) {
  const [policy, setPolicy] = useState<OverwritePolicy>("skip");
  const [preview, setPreview] = useState<BatchPreview | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!open || imageIds.length === 0) return;
    setLoading(true);
    previewBatchJob({ op, imageIds, overwritePolicy: policy })
      .then(setPreview)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, [open, op, imageIds, policy]);

  if (!open) return null;

  const opLabel = op === "tag" ? "Tag" : "Caption";

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="bg-zinc-800 border border-zinc-700 rounded-lg p-6 max-w-md w-full mx-4 shadow-xl">
        <h3 className="text-lg font-semibold text-zinc-100 mb-2">
          Batch {opLabel}: {imageIds.length} images
        </h3>

        {loading ? (
          <p className="text-zinc-400 text-sm mb-4">Loading preview...</p>
        ) : preview ? (
          <div className="space-y-2 mb-4">
            <p className="text-sm text-zinc-400">
              Model:{" "}
              <span className="text-zinc-200 font-medium">
                {preview.model}
              </span>
            </p>
            <p className="text-sm text-zinc-400">
              Will process:{" "}
              <span className="text-zinc-200">{preview.wouldProcess}</span>
              {preview.wouldSkip > 0 && (
                <span className="text-zinc-500">
                  {" "}
                  ({preview.wouldSkip} already have{" "}
                  {op === "tag" ? "tags" : "captions"})
                </span>
              )}
            </p>
          </div>
        ) : null}

        <div className="mb-6">
          <label className="flex items-center gap-2 text-sm text-zinc-400 cursor-pointer">
            <input
              type="checkbox"
              checked={policy === "overwrite"}
              onChange={(e) =>
                setPolicy(e.target.checked ? "overwrite" : "skip")
              }
              className="rounded"
            />
            Overwrite existing {op === "tag" ? "tags" : "captions"}
          </label>
        </div>

        <div className="flex justify-end gap-3">
          <button
            onClick={onCancel}
            className="px-4 py-2 text-sm rounded bg-zinc-700 text-zinc-300 hover:bg-zinc-600"
          >
            Cancel
          </button>
          <button
            onClick={() => onConfirm(policy)}
            disabled={loading || (preview?.wouldProcess ?? 0) === 0}
            className="px-4 py-2 text-sm rounded bg-purple-600 text-white hover:bg-purple-500 disabled:opacity-50"
          >
            Start {opLabel}
          </button>
        </div>
      </div>
    </div>
  );
}
