import { useState, useRef, useCallback, useEffect } from "react";

interface SliderOverlayProps {
  imageASrc: string;
  imageBSrc: string;
  labelA?: string;
  labelB?: string;
}

export function SliderOverlay({
  imageASrc,
  imageBSrc,
  labelA = "A",
  labelB = "B",
}: SliderOverlayProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [position, setPosition] = useState(50);
  const [dragging, setDragging] = useState(false);

  const updatePosition = useCallback(
    (clientX: number) => {
      const container = containerRef.current;
      if (!container) return;
      const rect = container.getBoundingClientRect();
      const pct = ((clientX - rect.left) / rect.width) * 100;
      setPosition(Math.min(100, Math.max(0, pct)));
    },
    [],
  );

  useEffect(() => {
    if (!dragging) return;

    const handleMove = (e: MouseEvent) => updatePosition(e.clientX);
    const handleUp = () => setDragging(false);

    window.addEventListener("mousemove", handleMove);
    window.addEventListener("mouseup", handleUp);
    return () => {
      window.removeEventListener("mousemove", handleMove);
      window.removeEventListener("mouseup", handleUp);
    };
  }, [dragging, updatePosition]);

  return (
    <div
      ref={containerRef}
      className="relative select-none cursor-col-resize overflow-hidden rounded-lg"
      onMouseDown={(e) => {
        setDragging(true);
        updatePosition(e.clientX);
      }}
    >
      {/* Image B (full) */}
      <img src={imageBSrc} alt={labelB} className="w-full block" />

      {/* Image A (clipped) */}
      <div
        className="absolute inset-0 overflow-hidden"
        style={{ width: `${position}%` }}
      >
        <img src={imageASrc} alt={labelA} className="w-full block" style={{ minWidth: containerRef.current?.offsetWidth }} />
      </div>

      {/* Slider line */}
      <div
        className="absolute top-0 bottom-0 w-0.5 bg-white/80 z-10"
        style={{ left: `${position}%` }}
      >
        <div className="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 w-6 h-6 bg-white rounded-full border-2 border-zinc-800 flex items-center justify-center">
          <div className="w-0.5 h-3 bg-zinc-800 rounded" />
        </div>
      </div>

      {/* Labels */}
      <div className="absolute top-2 left-2 px-1.5 py-0.5 bg-black/60 rounded text-xs text-white">
        {labelA}
      </div>
      <div className="absolute top-2 right-2 px-1.5 py-0.5 bg-black/60 rounded text-xs text-white">
        {labelB}
      </div>
    </div>
  );
}
