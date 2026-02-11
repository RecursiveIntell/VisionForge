import { Star } from "lucide-react";

interface StarRatingProps {
  value: number | null | undefined;
  onChange?: (rating: number | null) => void;
  size?: number;
  className?: string;
}

export function StarRating({
  value,
  onChange,
  size = 18,
  className = "",
}: StarRatingProps) {
  const rating = value ?? 0;

  return (
    <div className={`flex gap-0.5 ${className}`}>
      {[1, 2, 3, 4, 5].map((star) => (
        <button
          key={star}
          onClick={() => {
            if (!onChange) return;
            onChange(star === rating ? null : star);
          }}
          disabled={!onChange}
          className={`${onChange ? "cursor-pointer" : "cursor-default"}`}
        >
          <Star
            size={size}
            className={
              star <= rating
                ? "fill-amber-500 text-amber-500"
                : "text-zinc-600"
            }
          />
        </button>
      ))}
    </div>
  );
}
