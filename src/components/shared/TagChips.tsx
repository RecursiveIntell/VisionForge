import { X } from "lucide-react";
import type { TagEntry } from "../../types";

interface TagChipsProps {
  tags: TagEntry[];
  onRemove?: (tagId: number) => void;
  className?: string;
}

export function TagChips({ tags, onRemove, className = "" }: TagChipsProps) {
  return (
    <div className={`flex flex-wrap gap-1.5 ${className}`}>
      {tags.map((tag) => (
        <span
          key={tag.id}
          className={`inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs ${
            tag.source === "ai"
              ? "bg-blue-500/20 text-blue-300 border border-blue-500/30"
              : "bg-zinc-700 text-zinc-300 border border-zinc-600"
          }`}
        >
          {tag.name}
          {onRemove && (
            <button
              onClick={() => onRemove(tag.id)}
              className="hover:text-red-400"
            >
              <X size={12} />
            </button>
          )}
        </span>
      ))}
    </div>
  );
}
