import { useState } from "react";
import { Sparkles } from "lucide-react";

interface IdeaInputProps {
  onSubmit: (idea: string, numConcepts: number) => void;
  disabled?: boolean;
}

export function IdeaInput({ onSubmit, disabled }: IdeaInputProps) {
  const [idea, setIdea] = useState("");
  const [numConcepts, setNumConcepts] = useState(3);

  const handleSubmit = () => {
    if (idea.trim()) {
      onSubmit(idea.trim(), numConcepts);
    }
  };

  return (
    <div className="bg-zinc-800 border border-zinc-700 rounded-lg p-4 space-y-4">
      <label className="block">
        <span className="text-sm font-medium text-zinc-300">Your Idea</span>
        <textarea
          value={idea}
          onChange={(e) => setIdea(e.target.value)}
          placeholder="Describe your image idea... e.g. 'A mystical forest with glowing mushrooms at twilight'"
          rows={3}
          disabled={disabled}
          className="mt-1 block w-full bg-zinc-700 border border-zinc-600 rounded px-3 py-2 text-sm text-zinc-100 placeholder-zinc-500 resize-none focus:border-blue-500 focus:outline-none disabled:opacity-50"
          onKeyDown={(e) => {
            if (e.key === "Enter" && e.ctrlKey) handleSubmit();
          }}
        />
      </label>
      <div className="flex items-center justify-between">
        <label className="flex items-center gap-2">
          <span className="text-sm text-zinc-400">Concepts to generate:</span>
          <input
            type="number"
            min={1}
            max={5}
            value={numConcepts}
            onChange={(e) => setNumConcepts(parseInt(e.target.value) || 3)}
            disabled={disabled}
            className="w-16 bg-zinc-700 border border-zinc-600 rounded px-2 py-1 text-sm text-zinc-100 focus:border-blue-500 focus:outline-none disabled:opacity-50"
          />
        </label>
        <button
          onClick={handleSubmit}
          disabled={disabled || !idea.trim()}
          className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-500 disabled:bg-zinc-600 disabled:cursor-not-allowed text-white text-sm rounded"
        >
          <Sparkles size={16} />
          Run Pipeline
        </button>
      </div>
    </div>
  );
}
