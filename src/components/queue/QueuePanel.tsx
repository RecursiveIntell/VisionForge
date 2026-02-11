import { Pause, Play, RefreshCw } from "lucide-react";
import { QueueItem } from "./QueueItem";
import { useQueue } from "../../hooks/useQueue";
import { LoadingSpinner } from "../shared/LoadingSpinner";

export function QueuePanel() {
  const { jobs, paused, loading, error, refresh, togglePause, cancel, reorder } =
    useQueue();

  const pendingCount = jobs.filter((j) => j.status === "pending").length;
  const activeCount = jobs.filter((j) => j.status === "generating").length;

  return (
    <div className="p-6 max-w-3xl mx-auto space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <h2 className="text-lg font-semibold text-zinc-200">
            Generation Queue
          </h2>
          <span className="text-xs text-zinc-500">
            {activeCount > 0 && `${activeCount} active, `}
            {pendingCount} pending
          </span>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={refresh}
            className="p-1.5 text-zinc-400 hover:text-zinc-200 bg-zinc-800 border border-zinc-700 rounded"
            title="Refresh"
          >
            <RefreshCw size={14} />
          </button>
          <button
            onClick={togglePause}
            className={`flex items-center gap-1.5 px-3 py-1.5 text-sm rounded ${
              paused
                ? "bg-green-600 hover:bg-green-500 text-white"
                : "bg-amber-600 hover:bg-amber-500 text-white"
            }`}
          >
            {paused ? (
              <>
                <Play size={14} />
                Resume
              </>
            ) : (
              <>
                <Pause size={14} />
                Pause
              </>
            )}
          </button>
        </div>
      </div>

      {paused && (
        <div className="bg-amber-400/10 border border-amber-400/20 rounded-lg px-3 py-2 text-sm text-amber-400">
          Queue is paused. New jobs will wait until resumed.
        </div>
      )}

      {error && (
        <div className="bg-red-400/10 border border-red-400/20 rounded-lg px-3 py-2 text-sm text-red-400">
          {error}
        </div>
      )}

      {loading ? (
        <div className="flex items-center justify-center py-12">
          <LoadingSpinner size={24} />
        </div>
      ) : jobs.length === 0 ? (
        <div className="flex items-center justify-center py-12 text-zinc-500 text-sm">
          Queue is empty. Generate images from the Prompt Studio!
        </div>
      ) : (
        <div className="space-y-2">
          {jobs.map((job) => (
            <QueueItem
              key={job.id}
              job={job}
              onCancel={() => cancel(job.id)}
              onReorder={(priority) => reorder(job.id, priority)}
            />
          ))}
        </div>
      )}
    </div>
  );
}
