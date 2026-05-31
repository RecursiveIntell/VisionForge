import { useState, useEffect, useCallback, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { getGalleryImages } from "../api/gallery";
import type { ImageEntry, GalleryFilter } from "../types";

export function useGallery(initialFilter?: Partial<GalleryFilter>) {
  const [images, setImages] = useState<ImageEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [hasMore, setHasMore] = useState(true);
  const [filter, setFilter] = useState<GalleryFilter>({
    sortBy: "createdAt",
    sortOrder: "desc",
    limit: 50,
    offset: 0,
    ...initialFilter,
  });

  // Track the current offset for pagination separately from the filter.
  // This avoids the bug where mutating the filter triggers refresh (full replace).
  const currentOffsetRef = useRef(0);

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    currentOffsetRef.current = 0;
    try {
      const refreshFilter = { ...filter, offset: 0 };
      const result = await getGalleryImages(refreshFilter);
      setImages(result);
      setHasMore(result.length >= (filter.limit ?? 50));
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load gallery");
    } finally {
      setLoading(false);
    }
  }, [filter]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  // Auto-refresh when a new image is generated
  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;

    listen("queue:job_completed", () => refresh()).then((u) => {
      if (cancelled) {
        u(); // Immediately unlisten if effect already cleaned up
      } else {
        unlisten = u;
      }
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [refresh]);

  const updateFilter = useCallback((updates: Partial<GalleryFilter>) => {
    setFilter((prev) => ({ ...prev, ...updates, offset: 0 }));
  }, []);

  const loadMore = useCallback(async () => {
    const pageSize = filter.limit ?? 50;
    const nextOffset = currentOffsetRef.current + pageSize;
    try {
      const pageFilter = { ...filter, offset: nextOffset };
      const moreImages = await getGalleryImages(pageFilter);
      if (moreImages.length > 0) {
        currentOffsetRef.current = nextOffset;
        setImages((prev) => [...prev, ...moreImages]);
      }
      setHasMore(moreImages.length >= pageSize);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load more images");
    }
  }, [filter]);

  return { images, loading, error, filter, updateFilter, loadMore, hasMore, refresh };
}
