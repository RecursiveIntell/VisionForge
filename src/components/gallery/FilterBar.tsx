import { useState, useEffect, useRef } from "react";
import { Search, SortAsc, SortDesc, Heart, Trash2, Tag, Type } from "lucide-react";
import type { GalleryFilter, GallerySortField, SortOrder } from "../../types";

interface FilterBarProps {
  filter: GalleryFilter;
  onFilterChange: (updates: Partial<GalleryFilter>) => void;
}

export function FilterBar({ filter, onFilterChange }: FilterBarProps) {
  const [searchInput, setSearchInput] = useState(filter.search ?? "");
  const onFilterChangeRef = useRef(onFilterChange);
  onFilterChangeRef.current = onFilterChange;

  // Debounce search input — wait 300ms after last keystroke before propagating
  useEffect(() => {
    const timer = setTimeout(() => {
      onFilterChangeRef.current({ search: searchInput || undefined });
    }, 300);
    return () => clearTimeout(timer);
  }, [searchInput]);

  // Sync local state if the filter is changed externally (e.g., reset)
  useEffect(() => {
    setSearchInput(filter.search ?? "");
  }, [filter.search]);

  return (
    <div className="flex flex-wrap items-center gap-3 bg-zinc-800 border border-zinc-700 rounded-lg px-4 py-3">
      {/* Search */}
      <div className="relative flex-1 min-w-[200px]">
        <Search
          size={14}
          className="absolute left-2.5 top-1/2 -translate-y-1/2 text-zinc-500"
        />
        <input
          type="text"
          value={searchInput}
          onChange={(e) => setSearchInput(e.target.value)}
          placeholder="Search prompts, captions..."
          className="w-full bg-zinc-700 border border-zinc-600 rounded pl-8 pr-3 py-1.5 text-sm text-zinc-100 placeholder-zinc-500 focus:border-blue-500 focus:outline-none"
        />
      </div>

      {/* Min Rating */}
      <select
        value={filter.minRating ?? ""}
        onChange={(e) =>
          onFilterChange({
            minRating: e.target.value ? Number(e.target.value) : undefined,
          })
        }
        className="bg-zinc-700 border border-zinc-600 rounded px-2 py-1.5 text-sm text-zinc-200 focus:border-blue-500 focus:outline-none"
      >
        <option value="">Any rating</option>
        <option value="1">1+ stars</option>
        <option value="2">2+ stars</option>
        <option value="3">3+ stars</option>
        <option value="4">4+ stars</option>
        <option value="5">5 stars</option>
      </select>

      {/* Sort */}
      <select
        value={filter.sortBy ?? "createdAt"}
        onChange={(e) =>
          onFilterChange({ sortBy: e.target.value as GallerySortField })
        }
        className="bg-zinc-700 border border-zinc-600 rounded px-2 py-1.5 text-sm text-zinc-200 focus:border-blue-500 focus:outline-none"
      >
        <option value="createdAt">Date</option>
        <option value="rating">Rating</option>
        <option value="random">Random</option>
      </select>

      <button
        onClick={() =>
          onFilterChange({
            sortOrder:
              (filter.sortOrder ?? "desc") === "desc"
                ? ("asc" as SortOrder)
                : ("desc" as SortOrder),
          })
        }
        className="p-1.5 text-zinc-400 hover:text-zinc-200 bg-zinc-700 border border-zinc-600 rounded"
        title={`Sort ${filter.sortOrder === "asc" ? "ascending" : "descending"}`}
      >
        {filter.sortOrder === "asc" ? (
          <SortAsc size={14} />
        ) : (
          <SortDesc size={14} />
        )}
      </button>

      {/* Toggles */}
      <button
        onClick={() =>
          onFilterChange({ favoriteOnly: !filter.favoriteOnly })
        }
        className={`p-1.5 rounded border ${
          filter.favoriteOnly
            ? "text-red-400 border-red-400/30 bg-red-400/10"
            : "text-zinc-400 border-zinc-600 bg-zinc-700 hover:text-zinc-200"
        }`}
        title="Favorites only"
      >
        <Heart size={14} />
      </button>

      <button
        onClick={() =>
          onFilterChange({ showDeleted: !filter.showDeleted })
        }
        className={`p-1.5 rounded border ${
          filter.showDeleted
            ? "text-red-400 border-red-400/30 bg-red-400/10"
            : "text-zinc-400 border-zinc-600 bg-zinc-700 hover:text-zinc-200"
        }`}
        title="Show deleted"
      >
        <Trash2 size={14} />
      </button>

      <button
        onClick={() =>
          onFilterChange({ untaggedOnly: !filter.untaggedOnly })
        }
        className={`px-2 py-1.5 rounded border text-xs flex items-center gap-1 ${
          filter.untaggedOnly
            ? "text-orange-400 border-orange-400/30 bg-orange-400/10"
            : "text-zinc-400 border-zinc-600 bg-zinc-700 hover:text-zinc-200"
        }`}
        title="Show only untagged images"
      >
        <Tag size={12} />
        Untagged
      </button>

      <button
        onClick={() =>
          onFilterChange({ uncaptionedOnly: !filter.uncaptionedOnly })
        }
        className={`px-2 py-1.5 rounded border text-xs flex items-center gap-1 ${
          filter.uncaptionedOnly
            ? "text-orange-400 border-orange-400/30 bg-orange-400/10"
            : "text-zinc-400 border-zinc-600 bg-zinc-700 hover:text-zinc-200"
        }`}
        title="Show only uncaptioned images"
      >
        <Type size={12} />
        Uncaptioned
      </button>
    </div>
  );
}
