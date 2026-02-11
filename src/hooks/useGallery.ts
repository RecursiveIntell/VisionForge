import { useState, useEffect, useCallback } from "react";
import { getGalleryImages } from "../api/gallery";
import type { ImageEntry, GalleryFilter } from "../types";

export function useGallery(initialFilter?: Partial<GalleryFilter>) {
  const [images, setImages] = useState<ImageEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [filter, setFilter] = useState<GalleryFilter>({
    sortBy: "createdAt",
    sortOrder: "desc",
    limit: 50,
    offset: 0,
    ...initialFilter,
  });

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await getGalleryImages(filter);
      setImages(result);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load gallery");
    } finally {
      setLoading(false);
    }
  }, [filter]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const updateFilter = useCallback((updates: Partial<GalleryFilter>) => {
    setFilter((prev) => ({ ...prev, ...updates, offset: 0 }));
  }, []);

  const loadMore = useCallback(() => {
    setFilter((prev) => ({
      ...prev,
      offset: (prev.offset ?? 0) + (prev.limit ?? 50),
    }));
  }, []);

  return { images, loading, error, filter, updateFilter, loadMore, refresh };
}
