interface ProgressBarProps {
  progress: number; // 0-100
  className?: string;
}

export function ProgressBar({ progress, className = "" }: ProgressBarProps) {
  const clamped = Math.min(100, Math.max(0, progress));

  return (
    <div
      className={`w-full h-1.5 bg-zinc-700 rounded-full overflow-hidden ${className}`}
    >
      <div
        className="h-full bg-blue-500 rounded-full transition-all duration-300"
        style={{ width: `${clamped}%` }}
      />
    </div>
  );
}
