import { useState, useEffect, useCallback } from "react";
import {
  listComparisons,
  createComparison,
  deleteComparison,
  updateComparisonNote,
} from "../api/comparison";
import type { Comparison } from "../types";

export function useComparison() {
  const [comparisons, setComparisons] = useState<Comparison[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await listComparisons();
      setComparisons(result);
    } catch (e) {
      setError(
        e instanceof Error ? e.message : "Failed to load comparisons",
      );
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const create = useCallback(
    async (comparison: Comparison) => {
      await createComparison(comparison);
      refresh();
    },
    [refresh],
  );

  const remove = useCallback(
    async (id: string) => {
      await deleteComparison(id);
      refresh();
    },
    [refresh],
  );

  const updateNote = useCallback(
    async (id: string, note: string) => {
      await updateComparisonNote(id, note);
      refresh();
    },
    [refresh],
  );

  return { comparisons, loading, error, refresh, create, remove, updateNote };
}
