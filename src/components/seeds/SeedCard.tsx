import { Hash, Trash2 } from "lucide-react";
import type { SeedEntry } from "../../types";

interface SeedCardProps {
  seed: SeedEntry;
  onSelect: () => void;
  onDelete: () => void;
}

export function SeedCard({ seed, onSelect, onDelete }: SeedCardProps) {
  return (
    <div
      onClick={onSelect}
      className="bg-zinc-800 border border-zinc-700 rounded-lg p-3 cursor-pointer hover:border-zinc-600 group"
    >
      <div className="flex items-start justify-between">
        <div className="flex items-center gap-2">
          <Hash size={14} className="text-blue-400 shrink-0" />
          <span className="text-sm font-mono text-zinc-200">
            {seed.seedValue}
          </span>
        </div>
        <button
          onClick={(e) => {
            e.stopPropagation();
            onDelete();
          }}
          className="p-1 text-zinc-600 hover:text-red-400 opacity-0 group-hover:opacity-100"
        >
          <Trash2 size={12} />
        </button>
      </div>

      {seed.comment && (
        <p className="text-xs text-zinc-400 mt-1.5 line-clamp-2">
          {seed.comment}
        </p>
      )}

      <div className="flex items-center gap-2 mt-2 flex-wrap">
        {seed.checkpoint && (
          <span className="text-[10px] text-zinc-500 bg-zinc-700 px-1.5 py-0.5 rounded">
            {seed.checkpoint}
          </span>
        )}
        {seed.tags?.map((tag, i) => (
          <span
            key={i}
            className="text-[10px] text-blue-400 bg-blue-400/10 px-1.5 py-0.5 rounded"
          >
            {tag}
          </span>
        ))}
      </div>
    </div>
  );
}
