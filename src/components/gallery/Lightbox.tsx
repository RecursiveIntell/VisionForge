import { useState, useEffect, useCallback } from "react";
import { X, ChevronLeft, ChevronRight, ZoomIn, ZoomOut } from "lucide-react";
import { getImageFilePath } from "../../api/gallery";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { ImageEntry } from "../../types";

interface LightboxProps {
  images: ImageEntry[];
  currentIndex: number;
  onClose: () => void;
  onNavigate: (index: number) => void;
}

export function Lightbox({
  images,
  currentIndex,
  onClose,
  onNavigate,
}: LightboxProps) {
  const [imageSrc, setImageSrc] = useState<string | null>(null);
  const [zoomed, setZoomed] = useState(false);
  const current = images[currentIndex];

  useEffect(() => {
    if (!current) return;
    setImageSrc(null);
    getImageFilePath(current.filename)
      .then((path) => setImageSrc(convertFileSrc(path)))
      .catch(() => setImageSrc(null));
  }, [current]);

  const goNext = useCallback(() => {
    if (currentIndex < images.length - 1) {
      onNavigate(currentIndex + 1);
      setZoomed(false);
    }
  }, [currentIndex, images.length, onNavigate]);

  const goPrev = useCallback(() => {
    if (currentIndex > 0) {
      onNavigate(currentIndex - 1);
      setZoomed(false);
    }
  }, [currentIndex, onNavigate]);

  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      switch (e.key) {
        case "Escape":
          onClose();
          break;
        case "ArrowRight":
          goNext();
          break;
        case "ArrowLeft":
          goPrev();
          break;
      }
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [onClose, goNext, goPrev]);

  if (!current) return null;

  return (
    <div className="fixed inset-0 z-50 bg-black/90 flex items-center justify-center">
      {/* Controls */}
      <div className="absolute top-4 right-4 flex items-center gap-2 z-10">
        <span className="text-xs text-zinc-400">
          {currentIndex + 1} / {images.length}
        </span>
        <button
          onClick={() => setZoomed(!zoomed)}
          className="p-2 text-zinc-400 hover:text-white bg-zinc-800/80 rounded"
        >
          {zoomed ? <ZoomOut size={16} /> : <ZoomIn size={16} />}
        </button>
        <button
          onClick={onClose}
          className="p-2 text-zinc-400 hover:text-white bg-zinc-800/80 rounded"
        >
          <X size={16} />
        </button>
      </div>

      {/* Nav arrows */}
      {currentIndex > 0 && (
        <button
          onClick={goPrev}
          className="absolute left-4 top-1/2 -translate-y-1/2 p-3 text-zinc-400 hover:text-white bg-zinc-800/60 rounded-full z-10"
        >
          <ChevronLeft size={24} />
        </button>
      )}
      {currentIndex < images.length - 1 && (
        <button
          onClick={goNext}
          className="absolute right-4 top-1/2 -translate-y-1/2 p-3 text-zinc-400 hover:text-white bg-zinc-800/60 rounded-full z-10"
        >
          <ChevronRight size={24} />
        </button>
      )}

      {/* Image */}
      <div
        className={`flex items-center justify-center ${
          zoomed ? "overflow-auto cursor-zoom-out" : "cursor-zoom-in"
        }`}
        onClick={() => setZoomed(!zoomed)}
        style={{ maxWidth: "90vw", maxHeight: "90vh" }}
      >
        {imageSrc ? (
          <img
            src={imageSrc}
            alt={current.caption ?? "Image"}
            className={zoomed ? "max-w-none" : "max-w-full max-h-[90vh] object-contain"}
          />
        ) : (
          <div className="text-zinc-500">Loading...</div>
        )}
      </div>

      {/* Caption bar */}
      {current.positivePrompt && (
        <div className="absolute bottom-0 left-0 right-0 bg-gradient-to-t from-black/80 to-transparent p-4 pt-12">
          <p className="text-xs text-zinc-300 max-w-3xl mx-auto truncate">
            {current.positivePrompt}
          </p>
        </div>
      )}
    </div>
  );
}
