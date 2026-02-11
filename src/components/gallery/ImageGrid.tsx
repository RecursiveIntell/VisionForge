import { ImageCard } from "./ImageCard";
import type { ImageEntry } from "../../types";

interface ImageGridProps {
  images: ImageEntry[];
  selectedId?: string;
  compareSelection?: string[];
  onSelect: (image: ImageEntry) => void;
  onEnlarge?: (image: ImageEntry) => void;
  onFavoriteToggle: (image: ImageEntry) => void;
}

export function ImageGrid({
  images,
  selectedId,
  compareSelection,
  onSelect,
  onEnlarge,
  onFavoriteToggle,
}: ImageGridProps) {
  if (images.length === 0) {
    return (
      <div className="flex items-center justify-center h-64 text-zinc-500 text-sm">
        No images found. Generate some in the Prompt Studio!
      </div>
    );
  }

  return (
    <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-3">
      {images.map((image) => (
        <ImageCard
          key={image.id}
          image={image}
          selected={image.id === selectedId}
          compareSelected={compareSelection?.includes(image.id)}
          onClick={() => onSelect(image)}
          onEnlarge={onEnlarge ? () => onEnlarge(image) : undefined}
          onFavoriteToggle={() => onFavoriteToggle(image)}
        />
      ))}
    </div>
  );
}
