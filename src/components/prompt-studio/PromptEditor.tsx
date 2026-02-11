interface PromptEditorProps {
  positive: string;
  negative: string;
  onPositiveChange: (value: string) => void;
  onNegativeChange: (value: string) => void;
  disabled?: boolean;
}

export function PromptEditor({
  positive,
  negative,
  onPositiveChange,
  onNegativeChange,
  disabled,
}: PromptEditorProps) {
  return (
    <div className="space-y-3">
      <label className="block">
        <span className="text-sm font-medium text-green-400">
          Positive Prompt
        </span>
        <textarea
          value={positive}
          onChange={(e) => onPositiveChange(e.target.value)}
          disabled={disabled}
          rows={4}
          className="mt-1 block w-full bg-zinc-700 border border-zinc-600 rounded px-3 py-2 text-sm text-zinc-100 placeholder-zinc-500 resize-y focus:border-blue-500 focus:outline-none disabled:opacity-50 font-mono"
          placeholder="Positive prompt will appear here..."
        />
      </label>
      <label className="block">
        <span className="text-sm font-medium text-red-400">
          Negative Prompt
        </span>
        <textarea
          value={negative}
          onChange={(e) => onNegativeChange(e.target.value)}
          disabled={disabled}
          rows={3}
          className="mt-1 block w-full bg-zinc-700 border border-zinc-600 rounded px-3 py-2 text-sm text-zinc-100 placeholder-zinc-500 resize-y focus:border-blue-500 focus:outline-none disabled:opacity-50 font-mono"
          placeholder="Negative prompt will appear here..."
        />
      </label>
    </div>
  );
}
