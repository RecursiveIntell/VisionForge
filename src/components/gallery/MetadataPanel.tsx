import { useState } from "react";
import { X, Tag, MessageSquare, StickyNote, Sparkles } from "lucide-react";
import { StarRating } from "../shared/StarRating";
import { TagChips } from "../shared/TagChips";
import { tagImage, captionImage } from "../../api/ai";
import { useToast } from "../shared/Toast";
import type { ImageEntry } from "../../types";

interface MetadataPanelProps {
  image: ImageEntry;
  onClose: () => void;
  onRatingChange: (rating: number | null) => void;
  onFavoriteToggle: () => void;
  onCaptionSave: (caption: string) => void;
  onNoteSave: (note: string) => void;
  onAddTag: (tag: string) => void;
  onRemoveTag: (tagId: number) => void;
  onDelete: () => void;
  onRefresh?: () => void;
}

export function MetadataPanel({
  image,
  onClose,
  onRatingChange,
  onFavoriteToggle,
  onCaptionSave,
  onNoteSave,
  onAddTag,
  onRemoveTag,
  onDelete,
  onRefresh,
}: MetadataPanelProps) {
  const [editingCaption, setEditingCaption] = useState(false);
  const [captionDraft, setCaptionDraft] = useState(image.caption ?? "");
  const [editingNote, setEditingNote] = useState(false);
  const [noteDraft, setNoteDraft] = useState(image.userNote ?? "");
  const [newTag, setNewTag] = useState("");
  const [aiLoading, setAiLoading] = useState<"tag" | "caption" | null>(null);
  const { addToast } = useToast();

  const handleCaptionSave = () => {
    onCaptionSave(captionDraft);
    setEditingCaption(false);
  };

  const handleNoteSave = () => {
    onNoteSave(noteDraft);
    setEditingNote(false);
  };

  const handleAddTag = () => {
    if (newTag.trim()) {
      onAddTag(newTag.trim());
      setNewTag("");
    }
  };

  const handleAiTag = async () => {
    setAiLoading("tag");
    try {
      const tags = await tagImage(image.id);
      addToast("success", `Added ${tags.length} AI tags`);
      onRefresh?.();
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      addToast("error", `AI tagging failed: ${msg}`);
    } finally {
      setAiLoading(null);
    }
  };

  const handleAiCaption = async () => {
    setAiLoading("caption");
    try {
      const caption = await captionImage(image.id);
      setCaptionDraft(caption);
      onCaptionSave(caption);
      addToast("success", "AI caption generated");
      onRefresh?.();
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      addToast("error", `AI captioning failed: ${msg}`);
    } finally {
      setAiLoading(null);
    }
  };

  return (
    <div className="w-80 bg-zinc-800 border-l border-zinc-700 overflow-y-auto shrink-0">
      <div className="flex items-center justify-between p-4 border-b border-zinc-700">
        <h3 className="text-sm font-semibold text-zinc-200">Details</h3>
        <button
          onClick={onClose}
          className="p-1 text-zinc-500 hover:text-zinc-300"
        >
          <X size={16} />
        </button>
      </div>

      <div className="p-4 space-y-4">
        {/* Rating */}
        <div>
          <label className="text-xs text-zinc-500 block mb-1">Rating</label>
          <StarRating
            value={image.rating ?? 0}
            onChange={(v) => onRatingChange(v === image.rating ? null : v)}
          />
        </div>

        {/* Caption */}
        <div>
          <div className="flex items-center gap-1 mb-1">
            <MessageSquare size={12} className="text-zinc-500" />
            <label className="text-xs text-zinc-500">Caption</label>
            <button
              onClick={handleAiCaption}
              disabled={aiLoading !== null}
              className="ml-auto flex items-center gap-1 px-1.5 py-0.5 text-[10px] bg-purple-600/20 text-purple-400 hover:bg-purple-600/30 disabled:opacity-50 rounded"
              title="Auto-caption with AI"
            >
              <Sparkles size={10} />
              {aiLoading === "caption" ? "Working..." : "AI Caption"}
            </button>
          </div>
          {editingCaption ? (
            <div className="space-y-1">
              <textarea
                value={captionDraft}
                onChange={(e) => setCaptionDraft(e.target.value)}
                rows={2}
                className="w-full bg-zinc-700 border border-zinc-600 rounded px-2 py-1 text-xs text-zinc-100 resize-none focus:border-blue-500 focus:outline-none"
              />
              <div className="flex gap-1">
                <button
                  onClick={handleCaptionSave}
                  className="px-2 py-0.5 text-xs bg-blue-600 text-white rounded"
                >
                  Save
                </button>
                <button
                  onClick={() => setEditingCaption(false)}
                  className="px-2 py-0.5 text-xs text-zinc-400 hover:text-zinc-200"
                >
                  Cancel
                </button>
              </div>
            </div>
          ) : (
            <p
              onClick={() => {
                setCaptionDraft(image.caption ?? "");
                setEditingCaption(true);
              }}
              className="text-xs text-zinc-300 cursor-pointer hover:bg-zinc-700 rounded px-1 py-0.5"
            >
              {image.caption || "Click to add caption..."}
            </p>
          )}
        </div>

        {/* Tags */}
        <div>
          <div className="flex items-center gap-1 mb-1">
            <Tag size={12} className="text-zinc-500" />
            <label className="text-xs text-zinc-500">Tags</label>
            <button
              onClick={handleAiTag}
              disabled={aiLoading !== null}
              className="ml-auto flex items-center gap-1 px-1.5 py-0.5 text-[10px] bg-purple-600/20 text-purple-400 hover:bg-purple-600/30 disabled:opacity-50 rounded"
              title="Auto-tag with AI"
            >
              <Sparkles size={10} />
              {aiLoading === "tag" ? "Working..." : "AI Tag"}
            </button>
          </div>
          <TagChips
            tags={(image.tags ?? []).map((t) => ({
              id: t.id,
              name: t.name,
              source: t.source as "ai" | "user" | undefined,
            }))}
            onRemove={(id) => onRemoveTag(id)}
          />
          <div className="flex gap-1 mt-1">
            <input
              type="text"
              value={newTag}
              onChange={(e) => setNewTag(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && handleAddTag()}
              placeholder="Add tag..."
              className="flex-1 bg-zinc-700 border border-zinc-600 rounded px-2 py-1 text-xs text-zinc-100 placeholder-zinc-500 focus:border-blue-500 focus:outline-none"
            />
            <button
              onClick={handleAddTag}
              disabled={!newTag.trim()}
              className="px-2 py-1 text-xs bg-zinc-600 text-zinc-200 rounded hover:bg-zinc-500 disabled:opacity-50"
            >
              Add
            </button>
          </div>
        </div>

        {/* Notes */}
        <div>
          <div className="flex items-center gap-1 mb-1">
            <StickyNote size={12} className="text-zinc-500" />
            <label className="text-xs text-zinc-500">Note</label>
          </div>
          {editingNote ? (
            <div className="space-y-1">
              <textarea
                value={noteDraft}
                onChange={(e) => setNoteDraft(e.target.value)}
                rows={3}
                className="w-full bg-zinc-700 border border-zinc-600 rounded px-2 py-1 text-xs text-zinc-100 resize-none focus:border-blue-500 focus:outline-none"
              />
              <div className="flex gap-1">
                <button
                  onClick={handleNoteSave}
                  className="px-2 py-0.5 text-xs bg-blue-600 text-white rounded"
                >
                  Save
                </button>
                <button
                  onClick={() => setEditingNote(false)}
                  className="px-2 py-0.5 text-xs text-zinc-400 hover:text-zinc-200"
                >
                  Cancel
                </button>
              </div>
            </div>
          ) : (
            <p
              onClick={() => {
                setNoteDraft(image.userNote ?? "");
                setEditingNote(true);
              }}
              className="text-xs text-zinc-300 cursor-pointer hover:bg-zinc-700 rounded px-1 py-0.5"
            >
              {image.userNote || "Click to add note..."}
            </p>
          )}
        </div>

        {/* Generation Info */}
        <div className="border-t border-zinc-700 pt-3">
          <label className="text-xs text-zinc-500 block mb-2">
            Generation Info
          </label>
          <div className="space-y-1 text-xs">
            <InfoRow label="Checkpoint" value={image.checkpoint} />
            <InfoRow label="Seed" value={image.seed?.toString()} />
            <InfoRow label="Steps" value={image.steps?.toString()} />
            <InfoRow label="CFG" value={image.cfgScale?.toString()} />
            <InfoRow label="Sampler" value={image.sampler} />
            <InfoRow label="Scheduler" value={image.scheduler} />
            <InfoRow
              label="Resolution"
              value={
                image.width && image.height
                  ? `${image.width}x${image.height}`
                  : undefined
              }
            />
            <InfoRow label="Created" value={image.createdAt} />
          </div>
        </div>

        {/* Actions */}
        <div className="border-t border-zinc-700 pt-3 flex gap-2">
          <button
            onClick={onFavoriteToggle}
            className={`flex-1 py-1.5 text-xs rounded ${
              image.favorite
                ? "bg-red-400/10 text-red-400 border border-red-400/20"
                : "bg-zinc-700 text-zinc-300 hover:bg-zinc-600"
            }`}
          >
            {image.favorite ? "Unfavorite" : "Favorite"}
          </button>
          <button
            onClick={onDelete}
            className="flex-1 py-1.5 text-xs bg-zinc-700 text-red-400 hover:bg-red-400/10 rounded"
          >
            {image.deleted ? "Restore" : "Delete"}
          </button>
        </div>
      </div>
    </div>
  );
}

function InfoRow({ label, value }: { label: string; value?: string }) {
  if (!value) return null;
  return (
    <div className="flex justify-between">
      <span className="text-zinc-500">{label}</span>
      <span className="text-zinc-300 truncate ml-2 max-w-[160px]">
        {value}
      </span>
    </div>
  );
}
