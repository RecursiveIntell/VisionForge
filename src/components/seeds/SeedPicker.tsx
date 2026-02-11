import { useState, useEffect } from "react";
import { X, Hash, Shuffle } from "lucide-react";
import { listSeeds } from "../../api/seeds";
import type { SeedEntry } from "../../types";

interface SeedPickerProps {
  onSelect: (seed: number) => void;
  onClose: () => void;
}

export function SeedPicker({ onSelect, onClose }: SeedPickerProps) {
  const [seeds, setSeeds] = useState<SeedEntry[]>([]);
  const [customSeed, setCustomSeed] = useState("");

  useEffect(() => {
    listSeeds({}).then(setSeeds).catch(console.error);
  }, []);

  const handleRandom = () => {
    onSelect(Math.floor(Math.random() * 2147483647));
  };

  const handleCustom = () => {
    const val = parseInt(customSeed);
    if (!isNaN(val)) {
      onSelect(val);
    }
  };

  return (
    <div className="bg-zinc-800 border border-zinc-700 rounded-lg p-4 space-y-3">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-semibold text-zinc-200">Pick a Seed</h3>
        <button
          onClick={onClose}
          className="p-1 text-zinc-500 hover:text-zinc-300"
        >
          <X size={14} />
        </button>
      </div>

      <div className="flex gap-2">
        <input
          type="number"
          value={customSeed}
          onChange={(e) => setCustomSeed(e.target.value)}
          placeholder="Enter seed..."
          className="flex-1 bg-zinc-700 border border-zinc-600 rounded px-2 py-1.5 text-sm text-zinc-100 placeholder-zinc-500 focus:border-blue-500 focus:outline-none"
        />
        <button
          onClick={handleCustom}
          disabled={!customSeed}
          className="px-3 py-1.5 text-sm bg-blue-600 hover:bg-blue-500 disabled:bg-zinc-600 text-white rounded"
        >
          Use
        </button>
        <button
          onClick={handleRandom}
          className="flex items-center gap-1 px-3 py-1.5 text-sm bg-zinc-700 hover:bg-zinc-600 text-zinc-200 rounded"
        >
          <Shuffle size={14} />
          Random
        </button>
      </div>

      {seeds.length > 0 && (
        <div className="space-y-1 max-h-48 overflow-y-auto">
          <p className="text-xs text-zinc-500">Saved seeds:</p>
          {seeds.map((seed) => (
            <button
              key={seed.id}
              onClick={() => onSelect(seed.seedValue)}
              className="w-full flex items-center gap-2 px-2 py-1.5 text-left hover:bg-zinc-700 rounded"
            >
              <Hash size={12} className="text-blue-400 shrink-0" />
              <span className="text-sm font-mono text-zinc-200">
                {seed.seedValue}
              </span>
              {seed.comment && (
                <span className="text-xs text-zinc-500 truncate">
                  {seed.comment}
                </span>
              )}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
