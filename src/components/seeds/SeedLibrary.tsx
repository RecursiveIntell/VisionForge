import { useState, useEffect, useCallback } from "react";
import { Plus, Search } from "lucide-react";
import { SeedCard } from "./SeedCard";
import { listSeeds, createSeed, deleteSeed } from "../../api/seeds";
import { LoadingSpinner } from "../shared/LoadingSpinner";
import { ConfirmDialog } from "../shared/ConfirmDialog";
import type { SeedEntry, SeedFilter } from "../../types";

export function SeedLibrary() {
  const [seeds, setSeeds] = useState<SeedEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [search, setSearch] = useState("");
  const [showAddForm, setShowAddForm] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<SeedEntry | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const filter: SeedFilter = search ? { search } : {};
      const result = await listSeeds(filter);
      setSeeds(result);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load seeds");
    } finally {
      setLoading(false);
    }
  }, [search]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const handleCreate = async (seedValue: number, comment: string) => {
    try {
      await createSeed({ seedValue, comment });
      setShowAddForm(false);
      refresh();
    } catch (e) {
      console.error("Failed to create seed:", e);
    }
  };

  const handleDelete = async () => {
    if (!deleteTarget?.id) return;
    try {
      await deleteSeed(deleteTarget.id);
      setDeleteTarget(null);
      refresh();
    } catch (e) {
      console.error("Failed to delete seed:", e);
    }
  };

  return (
    <div className="p-6 max-w-3xl mx-auto space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold text-zinc-200">Seed Library</h2>
        <button
          onClick={() => setShowAddForm(true)}
          className="flex items-center gap-1.5 px-3 py-1.5 text-sm bg-blue-600 hover:bg-blue-500 text-white rounded"
        >
          <Plus size={14} />
          Add Seed
        </button>
      </div>

      <div className="relative">
        <Search
          size={14}
          className="absolute left-2.5 top-1/2 -translate-y-1/2 text-zinc-500"
        />
        <input
          type="text"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder="Search seeds..."
          className="w-full bg-zinc-800 border border-zinc-700 rounded-lg pl-8 pr-3 py-2 text-sm text-zinc-100 placeholder-zinc-500 focus:border-blue-500 focus:outline-none"
        />
      </div>

      {error && (
        <div className="bg-red-400/10 border border-red-400/20 rounded p-2 text-sm text-red-400">
          {error}
        </div>
      )}

      {showAddForm && (
        <AddSeedForm
          onSubmit={handleCreate}
          onCancel={() => setShowAddForm(false)}
        />
      )}

      {loading ? (
        <div className="flex items-center justify-center py-12">
          <LoadingSpinner size={24} />
        </div>
      ) : seeds.length === 0 ? (
        <div className="flex items-center justify-center py-12 text-zinc-500 text-sm">
          No seeds saved yet.
        </div>
      ) : (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
          {seeds.map((seed) => (
            <SeedCard
              key={seed.id}
              seed={seed}
              onSelect={() => {}}
              onDelete={() => setDeleteTarget(seed)}
            />
          ))}
        </div>
      )}

      {deleteTarget && (
        <ConfirmDialog
          open={true}
          title="Delete Seed"
          message={`Delete seed ${deleteTarget.seedValue}? This cannot be undone.`}
          onConfirm={handleDelete}
          onCancel={() => setDeleteTarget(null)}
          destructive
        />
      )}
    </div>
  );
}

function AddSeedForm({
  onSubmit,
  onCancel,
}: {
  onSubmit: (seedValue: number, comment: string) => void;
  onCancel: () => void;
}) {
  const [seedValue, setSeedValue] = useState("");
  const [comment, setComment] = useState("");

  return (
    <div className="bg-zinc-800 border border-zinc-700 rounded-lg p-4 space-y-3">
      <div className="flex gap-3">
        <label className="block flex-1">
          <span className="text-xs text-zinc-400">Seed Value</span>
          <input
            type="number"
            value={seedValue}
            onChange={(e) => setSeedValue(e.target.value)}
            placeholder="e.g. 42"
            className="mt-1 w-full bg-zinc-700 border border-zinc-600 rounded px-2 py-1.5 text-sm text-zinc-100 focus:border-blue-500 focus:outline-none"
          />
        </label>
        <label className="block flex-[2]">
          <span className="text-xs text-zinc-400">Comment</span>
          <input
            type="text"
            value={comment}
            onChange={(e) => setComment(e.target.value)}
            placeholder="Description..."
            className="mt-1 w-full bg-zinc-700 border border-zinc-600 rounded px-2 py-1.5 text-sm text-zinc-100 focus:border-blue-500 focus:outline-none"
          />
        </label>
      </div>
      <div className="flex justify-end gap-2">
        <button
          onClick={onCancel}
          className="px-3 py-1.5 text-sm text-zinc-400 hover:text-zinc-200"
        >
          Cancel
        </button>
        <button
          onClick={() => {
            const val = parseInt(seedValue);
            if (!isNaN(val)) onSubmit(val, comment);
          }}
          disabled={!seedValue}
          className="px-3 py-1.5 text-sm bg-blue-600 hover:bg-blue-500 disabled:bg-zinc-600 text-white rounded"
        >
          Save Seed
        </button>
      </div>
    </div>
  );
}
