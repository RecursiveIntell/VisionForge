import { useState, useEffect } from "react";
import { Trash2 } from "lucide-react";
import { SliderOverlay } from "./SliderOverlay";
import { DiffTable } from "./DiffTable";
import { useComparison } from "../../hooks/useComparison";
import { getImage, getImageFilePath } from "../../api/gallery";
import { convertFileSrc } from "@tauri-apps/api/core";
import { LoadingSpinner } from "../shared/LoadingSpinner";
import { ConfirmDialog } from "../shared/ConfirmDialog";
import type { Comparison, ImageEntry } from "../../types";

export function ComparisonView() {
  const { comparisons, loading, error, remove, updateNote } = useComparison();
  const [selected, setSelected] = useState<Comparison | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<string | null>(null);

  return (
    <div className="p-6 max-w-4xl mx-auto space-y-4">
      <h2 className="text-lg font-semibold text-zinc-200">A/B Comparisons</h2>

      {error && (
        <div className="bg-red-400/10 border border-red-400/20 rounded p-2 text-sm text-red-400">
          {error}
        </div>
      )}

      {loading ? (
        <div className="flex items-center justify-center py-12">
          <LoadingSpinner size={24} />
        </div>
      ) : comparisons.length === 0 ? (
        <div className="flex items-center justify-center py-12 text-zinc-500 text-sm">
          No comparisons yet. Create one from the gallery by selecting two
          images.
        </div>
      ) : (
        <div className="space-y-3">
          {comparisons.map((c) => (
            <ComparisonCard
              key={c.id}
              comparison={c}
              isSelected={selected?.id === c.id}
              onSelect={() =>
                setSelected(selected?.id === c.id ? null : c)
              }
              onDelete={() => setDeleteTarget(c.id)}
              onNoteChange={(note) => updateNote(c.id, note)}
            />
          ))}
        </div>
      )}

      {selected && <ComparisonDetail comparison={selected} />}

      {deleteTarget && (
        <ConfirmDialog
          open={true}
          title="Delete Comparison"
          message="Delete this comparison? This cannot be undone."
          onConfirm={() => {
            remove(deleteTarget);
            setDeleteTarget(null);
            if (selected?.id === deleteTarget) setSelected(null);
          }}
          onCancel={() => setDeleteTarget(null)}
          destructive
        />
      )}
    </div>
  );
}

function ComparisonCard({
  comparison,
  isSelected,
  onSelect,
  onDelete,
  onNoteChange,
}: {
  comparison: Comparison;
  isSelected: boolean;
  onSelect: () => void;
  onDelete: () => void;
  onNoteChange: (note: string) => void;
}) {
  const [editingNote, setEditingNote] = useState(false);
  const [noteDraft, setNoteDraft] = useState(comparison.note ?? "");

  return (
    <div
      className={`bg-zinc-800 border rounded-lg p-3 cursor-pointer ${
        isSelected ? "border-blue-500" : "border-zinc-700 hover:border-zinc-600"
      }`}
      onClick={onSelect}
    >
      <div className="flex items-center justify-between">
        <div>
          <span className="text-sm text-zinc-200">
            {comparison.variableChanged}
          </span>
          <span className="text-xs text-zinc-500 ml-2">
            {comparison.createdAt &&
              new Date(comparison.createdAt).toLocaleDateString()}
          </span>
        </div>
        <button
          onClick={(e) => {
            e.stopPropagation();
            onDelete();
          }}
          className="p-1 text-zinc-600 hover:text-red-400"
        >
          <Trash2 size={14} />
        </button>
      </div>
      {editingNote ? (
        <div className="mt-2 flex gap-1" onClick={(e) => e.stopPropagation()}>
          <input
            type="text"
            value={noteDraft}
            onChange={(e) => setNoteDraft(e.target.value)}
            className="flex-1 bg-zinc-700 border border-zinc-600 rounded px-2 py-1 text-xs text-zinc-100 focus:border-blue-500 focus:outline-none"
          />
          <button
            onClick={() => {
              onNoteChange(noteDraft);
              setEditingNote(false);
            }}
            className="px-2 py-1 text-xs bg-blue-600 text-white rounded"
          >
            Save
          </button>
        </div>
      ) : (
        <p
          className="text-xs text-zinc-400 mt-1 cursor-text"
          onClick={(e) => {
            e.stopPropagation();
            setNoteDraft(comparison.note ?? "");
            setEditingNote(true);
          }}
        >
          {comparison.note || "Click to add note..."}
        </p>
      )}
    </div>
  );
}

function ComparisonDetail({ comparison }: { comparison: Comparison }) {
  const [imageA, setImageA] = useState<ImageEntry | null>(null);
  const [imageB, setImageB] = useState<ImageEntry | null>(null);
  const [srcA, setSrcA] = useState<string | null>(null);
  const [srcB, setSrcB] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    setLoading(true);
    Promise.all([
      getImage(comparison.imageAId),
      getImage(comparison.imageBId),
    ])
      .then(async ([a, b]) => {
        setImageA(a);
        setImageB(b);
        if (a) {
          const path = await getImageFilePath(a.filename);
          setSrcA(convertFileSrc(path));
        }
        if (b) {
          const path = await getImageFilePath(b.filename);
          setSrcB(convertFileSrc(path));
        }
      })
      .catch(console.error)
      .finally(() => setLoading(false));
  }, [comparison]);

  if (loading) {
    return (
      <div className="flex items-center justify-center py-8">
        <LoadingSpinner size={24} />
      </div>
    );
  }

  if (!imageA || !imageB) {
    return (
      <div className="text-sm text-zinc-500 py-4">
        One or both images could not be loaded.
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {srcA && srcB && (
        <SliderOverlay
          imageASrc={srcA}
          imageBSrc={srcB}
          labelA="Image A"
          labelB="Image B"
        />
      )}
      <DiffTable
        imageA={imageA}
        imageB={imageB}
        variableChanged={comparison.variableChanged}
      />
    </div>
  );
}
