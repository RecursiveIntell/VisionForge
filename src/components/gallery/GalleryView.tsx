import { useState, useCallback, useEffect, useRef } from "react";
import { save } from "@tauri-apps/plugin-dialog";
import { FilterBar } from "./FilterBar";
import { ImageGrid } from "./ImageGrid";
import { MetadataPanel } from "./MetadataPanel";
import { Lightbox } from "./Lightbox";
import { LineageViewer } from "./LineageViewer";
import { SelectionToolbar } from "./SelectionToolbar";
import { BatchConfirmDialog } from "./BatchConfirmDialog";
import { useGallery } from "../../hooks/useGallery";
import {
  updateImageRating,
  updateImageFavorite,
  updateCaption,
  updateImageNote,
  addTag,
  removeTag,
  deleteImage,
  restoreImage,
} from "../../api/gallery";
import { submitBatchJob } from "../../api/aiBatch";
import { createComparison } from "../../api/comparison";
import { exportImages } from "../../api/export";
import { LoadingSpinner } from "../shared/LoadingSpinner";
import { useToast } from "../shared/Toast";
import type {
  ImageEntry,
  GalleryFilter,
  Comparison,
  BatchOpKind,
  OverwritePolicy,
} from "../../types";

export function GalleryView() {
  const { images, loading, error, filter, updateFilter, refresh } =
    useGallery();
  const [selectedImage, setSelectedImage] = useState<ImageEntry | null>(null);
  const [lightboxIndex, setLightboxIndex] = useState<number | null>(null);
  const [compareMode, setCompareMode] = useState(false);
  const [compareSelection, setCompareSelection] = useState<string[]>([]);
  const { addToast } = useToast();

  // Multi-select state
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const lastClickedRef = useRef<string | null>(null);

  // Batch dialog state
  const [batchOp, setBatchOp] = useState<BatchOpKind | null>(null);

  // Keep selectedImage in sync with refreshed data
  useEffect(() => {
    if (selectedImage) {
      const updated = images.find((i) => i.id === selectedImage.id);
      if (updated) {
        setSelectedImage(updated);
      }
    }
  }, [images]);

  // Keyboard shortcuts for multi-select
  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      // Ctrl+A to select all (only when not in an input)
      if (
        (e.ctrlKey || e.metaKey) &&
        e.key === "a" &&
        !compareMode &&
        !(e.target instanceof HTMLInputElement) &&
        !(e.target instanceof HTMLTextAreaElement)
      ) {
        e.preventDefault();
        setSelectedIds(new Set(images.map((i) => i.id)));
      }
      // Escape to deselect
      if (e.key === "Escape" && selectedIds.size > 0) {
        e.preventDefault();
        setSelectedIds(new Set());
        lastClickedRef.current = null;
      }
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [images, selectedIds.size, compareMode]);

  const handleSelect = useCallback((image: ImageEntry) => {
    setSelectedImage(image);
  }, []);

  const handleMultiSelect = useCallback(
    (image: ImageEntry, event: React.MouseEvent) => {
      // Only handle shift/ctrl clicks for multi-select
      if (!event.shiftKey && !event.ctrlKey && !event.metaKey) {
        return; // Let normal onClick handle it
      }
      event.preventDefault();
      event.stopPropagation();

      if (event.shiftKey && lastClickedRef.current) {
        const lastIdx = images.findIndex(
          (i) => i.id === lastClickedRef.current
        );
        const currentIdx = images.findIndex((i) => i.id === image.id);
        if (lastIdx >= 0 && currentIdx >= 0) {
          const start = Math.min(lastIdx, currentIdx);
          const end = Math.max(lastIdx, currentIdx);
          const rangeIds = images.slice(start, end + 1).map((i) => i.id);
          setSelectedIds((prev) => {
            const next = new Set(prev);
            rangeIds.forEach((id) => next.add(id));
            return next;
          });
        }
      } else if (event.ctrlKey || event.metaKey) {
        setSelectedIds((prev) => {
          const next = new Set(prev);
          if (next.has(image.id)) {
            next.delete(image.id);
          } else {
            next.add(image.id);
          }
          return next;
        });
      }
      lastClickedRef.current = image.id;
    },
    [images]
  );

  const selectAll = useCallback(() => {
    setSelectedIds(new Set(images.map((i) => i.id)));
  }, [images]);

  const deselectAll = useCallback(() => {
    setSelectedIds(new Set());
    lastClickedRef.current = null;
  }, []);

  const handleFavoriteToggle = useCallback(
    async (image: ImageEntry) => {
      try {
        await updateImageFavorite(image.id, !image.favorite);
        refresh();
      } catch (e) {
        console.error("Failed to toggle favorite:", e);
      }
    },
    [refresh]
  );

  const handleRatingChange = useCallback(
    async (rating: number | null) => {
      if (!selectedImage) return;
      try {
        await updateImageRating(selectedImage.id, rating);
        refresh();
      } catch (e) {
        console.error("Failed to update rating:", e);
      }
    },
    [selectedImage, refresh]
  );

  const handleCaptionSave = useCallback(
    async (caption: string) => {
      if (!selectedImage) return;
      try {
        await updateCaption(selectedImage.id, caption);
        refresh();
      } catch (e) {
        console.error("Failed to update caption:", e);
      }
    },
    [selectedImage, refresh]
  );

  const handleNoteSave = useCallback(
    async (note: string) => {
      if (!selectedImage) return;
      try {
        await updateImageNote(selectedImage.id, note);
        refresh();
      } catch (e) {
        console.error("Failed to update note:", e);
      }
    },
    [selectedImage, refresh]
  );

  const handleAddTag = useCallback(
    async (tag: string) => {
      if (!selectedImage) return;
      try {
        await addTag(selectedImage.id, tag, "user");
        refresh();
      } catch (e) {
        console.error("Failed to add tag:", e);
      }
    },
    [selectedImage, refresh]
  );

  const handleRemoveTag = useCallback(
    async (tagId: number) => {
      if (!selectedImage) return;
      try {
        await removeTag(selectedImage.id, tagId);
        refresh();
      } catch (e) {
        console.error("Failed to remove tag:", e);
      }
    },
    [selectedImage, refresh]
  );

  const handleDelete = useCallback(async () => {
    if (!selectedImage) return;
    try {
      if (selectedImage.deleted) {
        await restoreImage(selectedImage.id);
      } else {
        await deleteImage(selectedImage.id);
      }
      setSelectedImage(null);
      refresh();
    } catch (e) {
      console.error("Failed to delete/restore:", e);
    }
  }, [selectedImage, refresh]);

  const handleCompareToggle = useCallback((imageId: string) => {
    setCompareSelection((prev) => {
      if (prev.includes(imageId)) return prev.filter((id) => id !== imageId);
      if (prev.length >= 2) return [prev[1], imageId];
      return [...prev, imageId];
    });
  }, []);

  const handleCreateComparison = useCallback(async () => {
    if (compareSelection.length !== 2) return;
    const variable = prompt("What variable changed between these images?");
    if (!variable) return;
    const comparison: Comparison = {
      id: crypto.randomUUID(),
      imageAId: compareSelection[0],
      imageBId: compareSelection[1],
      variableChanged: variable,
    };
    try {
      await createComparison(comparison);
      addToast("success", "Comparison created");
      setCompareMode(false);
      setCompareSelection([]);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      addToast("error", `Failed to create comparison: ${msg}`);
    }
  }, [compareSelection]);

  const handleExport = useCallback(async () => {
    const ids = images.map((i) => i.id);
    if (ids.length === 0) return;
    try {
      const outputPath = await save({
        defaultPath: `visionforge-export-${Date.now()}.zip`,
        filters: [{ name: "ZIP Archive", extensions: ["zip"] }],
      });
      if (!outputPath) return;
      await exportImages(ids, outputPath);
      addToast("success", `Exported ${ids.length} images`);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      addToast("error", `Export failed: ${msg}`);
    }
  }, [images, addToast]);

  const openLightbox = useCallback(
    (image: ImageEntry) => {
      const idx = images.findIndex((i) => i.id === image.id);
      if (idx >= 0) setLightboxIndex(idx);
    },
    [images]
  );

  const handleBatchConfirm = useCallback(
    async (overwritePolicy: OverwritePolicy) => {
      if (!batchOp || selectedIds.size === 0) return;
      try {
        await submitBatchJob({
          op: batchOp,
          imageIds: Array.from(selectedIds),
          overwritePolicy,
        });
        const opLabel = batchOp === "tag" ? "tagging" : "captioning";
        addToast("success", `Batch ${opLabel} job submitted for ${selectedIds.size} images`);
        setBatchOp(null);
        deselectAll();
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        addToast("error", `Failed to submit batch job: ${msg}`);
      }
    },
    [batchOp, selectedIds, addToast, deselectAll]
  );

  return (
    <div className="flex h-full">
      <div className="flex-1 flex flex-col overflow-hidden p-6 gap-4">
        <div className="flex items-center gap-2">
          <div className="flex-1">
            <FilterBar
              filter={filter}
              onFilterChange={updateFilter as (u: Partial<GalleryFilter>) => void}
            />
          </div>
          <button
            onClick={() => {
              setCompareMode(!compareMode);
              setCompareSelection([]);
            }}
            className={`shrink-0 px-3 py-1.5 text-xs rounded ${
              compareMode
                ? "bg-blue-600 text-white"
                : "bg-zinc-800 border border-zinc-700 text-zinc-400 hover:text-zinc-200"
            }`}
          >
            {compareMode ? "Cancel Compare" : "Compare"}
          </button>
          <button
            onClick={handleExport}
            disabled={images.length === 0}
            className="shrink-0 px-3 py-1.5 text-xs bg-zinc-800 border border-zinc-700 text-zinc-400 hover:text-zinc-200 disabled:opacity-50 rounded"
          >
            Export
          </button>
        </div>

        {compareMode && (
          <div className="flex items-center gap-2 bg-blue-600/10 border border-blue-500/20 rounded-lg px-3 py-2">
            <span className="text-xs text-blue-400">
              Select 2 images to compare ({compareSelection.length}/2)
            </span>
            {compareSelection.length === 2 && (
              <button
                onClick={handleCreateComparison}
                className="ml-auto px-3 py-1 text-xs bg-blue-600 hover:bg-blue-500 text-white rounded"
              >
                Create Comparison
              </button>
            )}
          </div>
        )}

        {!compareMode && (
          <SelectionToolbar
            selectedCount={selectedIds.size}
            totalCount={images.length}
            onSelectAll={selectAll}
            onDeselectAll={deselectAll}
            onBatchTag={() => setBatchOp("tag")}
            onBatchCaption={() => setBatchOp("caption")}
          />
        )}

        {error && (
          <div className="bg-red-400/10 border border-red-400/20 rounded p-2 text-sm text-red-400">
            {error}
          </div>
        )}

        {loading ? (
          <div className="flex-1 flex items-center justify-center">
            <LoadingSpinner size={32} />
          </div>
        ) : (
          <div className="flex-1 overflow-auto">
            <ImageGrid
              images={images}
              selectedId={compareMode ? undefined : selectedImage?.id}
              selectedIds={!compareMode ? selectedIds : undefined}
              compareSelection={compareMode ? compareSelection : undefined}
              onSelect={(img) => {
                if (compareMode) {
                  handleCompareToggle(img.id);
                } else {
                  handleSelect(img);
                }
              }}
              onMultiSelect={!compareMode ? handleMultiSelect : undefined}
              onEnlarge={openLightbox}
              onFavoriteToggle={handleFavoriteToggle}
            />
          </div>
        )}
      </div>

      {selectedImage && (
        <div className="flex flex-col">
          <MetadataPanel
            image={selectedImage}
            onClose={() => setSelectedImage(null)}
            onRatingChange={handleRatingChange}
            onFavoriteToggle={() => handleFavoriteToggle(selectedImage)}
            onCaptionSave={handleCaptionSave}
            onNoteSave={handleNoteSave}
            onAddTag={handleAddTag}
            onRemoveTag={handleRemoveTag}
            onDelete={handleDelete}
            onRefresh={refresh}
            onViewFullSize={() => openLightbox(selectedImage)}
          />
          <div className="px-4 pb-4">
            <LineageViewer imageId={selectedImage.id} />
          </div>
        </div>
      )}

      {lightboxIndex !== null && (
        <Lightbox
          images={images}
          currentIndex={lightboxIndex}
          onClose={() => setLightboxIndex(null)}
          onNavigate={setLightboxIndex}
        />
      )}

      {batchOp && (
        <BatchConfirmDialog
          open={!!batchOp}
          op={batchOp}
          imageIds={Array.from(selectedIds)}
          onConfirm={handleBatchConfirm}
          onCancel={() => setBatchOp(null)}
        />
      )}
    </div>
  );
}
