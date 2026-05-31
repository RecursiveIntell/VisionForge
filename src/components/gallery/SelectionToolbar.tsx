import { Tag, Type, CheckSquare, XSquare } from "lucide-react";

interface SelectionToolbarProps {
  selectedCount: number;
  totalCount: number;
  onSelectAll: () => void;
  onDeselectAll: () => void;
  onBatchTag: () => void;
  onBatchCaption: () => void;
}

export function SelectionToolbar({
  selectedCount,
  totalCount,
  onSelectAll,
  onDeselectAll,
  onBatchTag,
  onBatchCaption,
}: SelectionToolbarProps) {
  if (selectedCount < 2) return null;

  return (
    <div className="flex items-center gap-3 bg-blue-600/10 border border-blue-500/20 rounded-lg px-3 py-2">
      <span className="text-xs text-blue-400 font-medium">
        {selectedCount} selected
      </span>

      <div className="flex items-center gap-1.5 ml-auto">
        <button
          onClick={onSelectAll}
          disabled={selectedCount === totalCount}
          className="flex items-center gap-1 px-2 py-1 text-xs text-zinc-400 hover:text-zinc-200 disabled:opacity-30"
          title="Select all visible (Ctrl+A)"
        >
          <CheckSquare size={12} />
          Select All
        </button>
        <button
          onClick={onDeselectAll}
          className="flex items-center gap-1 px-2 py-1 text-xs text-zinc-400 hover:text-zinc-200"
          title="Deselect all (Esc)"
        >
          <XSquare size={12} />
          Deselect
        </button>

        <div className="w-px h-4 bg-zinc-700 mx-1" />

        <button
          onClick={onBatchTag}
          className="flex items-center gap-1.5 px-3 py-1.5 text-xs bg-purple-600/20 text-purple-300 border border-purple-500/30 rounded hover:bg-purple-600/30"
        >
          <Tag size={12} />
          Tag Selected
        </button>
        <button
          onClick={onBatchCaption}
          className="flex items-center gap-1.5 px-3 py-1.5 text-xs bg-purple-600/20 text-purple-300 border border-purple-500/30 rounded hover:bg-purple-600/30"
        >
          <Type size={12} />
          Caption Selected
        </button>
      </div>
    </div>
  );
}
