import { useState, useEffect } from "react";
import { Heart, Star, Trash2, Maximize2 } from "lucide-react";
import { getThumbnailFilePath } from "../../api/gallery";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { ImageEntry } from "../../types";

interface ImageCardProps {
  image: ImageEntry;
  selected?: boolean;
  compareSelected?: boolean;
  onClick: () => void;
  onEnlarge?: () => void;
  onFavoriteToggle?: () => void;
}

export function ImageCard({
  image,
  selected,
  compareSelected,
  onClick,
  onEnlarge,
  onFavoriteToggle,
}: ImageCardProps) {
  const [thumbnailSrc, setThumbnailSrc] = useState<string | null>(null);

  useEffect(() => {
    getThumbnailFilePath(image.filename)
      .then((path) => setThumbnailSrc(convertFileSrc(path)))
      .catch(() => setThumbnailSrc(null));
  }, [image.filename]);

  return (
    <div
      onClick={onClick}
      className={`group relative bg-zinc-800 rounded-lg overflow-hidden cursor-pointer border-2 transition-colors ${
        compareSelected
          ? "border-green-500 ring-2 ring-green-500/30"
          : selected
            ? "border-blue-500"
            : "border-transparent hover:border-zinc-600"
      } ${image.deleted ? "opacity-50" : ""}`}
    >
      <div className="aspect-square bg-zinc-700">
        {thumbnailSrc ? (
          <img
            src={thumbnailSrc}
            alt={image.caption ?? "Generated image"}
            className="w-full h-full object-cover"
          />
        ) : (
          <div className="w-full h-full flex items-center justify-center text-zinc-600 text-xs">
            Loading...
          </div>
        )}
      </div>

      {/* Overlay on hover */}
      <div className="absolute inset-0 bg-gradient-to-t from-black/60 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity">
        {/* Top-left: enlarge button */}
        {onEnlarge && (
          <button
            onClick={(e) => {
              e.stopPropagation();
              onEnlarge();
            }}
            className="absolute top-1.5 left-1.5 p-1.5 rounded bg-black/50 text-zinc-300 hover:text-white hover:bg-black/70"
            title="View full size"
          >
            <Maximize2 size={14} />
          </button>
        )}

        <div className="absolute bottom-0 left-0 right-0 p-2 flex items-center justify-between">
          <div className="flex items-center gap-1">
            {image.rating !== undefined && image.rating !== null && image.rating > 0 && (
              <div className="flex items-center gap-0.5">
                <Star size={12} className="text-amber-400 fill-amber-400" />
                <span className="text-xs text-zinc-200">{image.rating}</span>
              </div>
            )}
            {image.deleted && (
              <Trash2 size={12} className="text-red-400" />
            )}
          </div>
          {onFavoriteToggle && (
            <button
              onClick={(e) => {
                e.stopPropagation();
                onFavoriteToggle();
              }}
              className="p-1 rounded hover:bg-black/30"
            >
              <Heart
                size={14}
                className={
                  image.favorite
                    ? "text-red-400 fill-red-400"
                    : "text-zinc-300"
                }
              />
            </button>
          )}
        </div>
      </div>

      {image.autoApproved && (
        <div className="absolute top-1 right-1 px-1.5 py-0.5 bg-amber-500/80 rounded text-[10px] font-medium text-black">
          AUTO
        </div>
      )}
    </div>
  );
}
