import { CheckCircle, Clock, AlertCircle } from "lucide-react";

interface StageCardProps {
  name: string;
  enabled: boolean;
  status: "pending" | "running" | "completed" | "skipped" | "error";
  model?: string;
  durationMs?: number;
  children?: React.ReactNode;
}

const statusIcons = {
  pending: <Clock size={16} className="text-zinc-500" />,
  running: <Clock size={16} className="text-blue-400 animate-pulse" />,
  completed: <CheckCircle size={16} className="text-green-400" />,
  skipped: <Clock size={16} className="text-zinc-600" />,
  error: <AlertCircle size={16} className="text-red-400" />,
};

export function StageCard({
  name,
  enabled,
  status,
  model,
  durationMs,
  children,
}: StageCardProps) {
  return (
    <div
      className={`bg-zinc-800 border rounded-lg p-4 ${
        !enabled ? "opacity-50 border-zinc-700" : "border-zinc-700"
      }`}
    >
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
          {statusIcons[status]}
          <h4 className="text-sm font-medium text-zinc-200">{name}</h4>
          {!enabled && (
            <span className="text-xs text-zinc-500">(disabled)</span>
          )}
        </div>
        <div className="flex items-center gap-3 text-xs text-zinc-500">
          {model && <span>{model}</span>}
          {durationMs !== undefined && (
            <span>{(durationMs / 1000).toFixed(1)}s</span>
          )}
        </div>
      </div>
      {children && <div className="mt-2">{children}</div>}
    </div>
  );
}
