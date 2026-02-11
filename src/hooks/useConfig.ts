import { useState, useEffect, useCallback } from "react";
import { getConfig, saveConfig } from "../api/config";
import type { AppConfig } from "../types";

export function useConfig() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await getConfig();
      setConfig(result);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const save = useCallback(
    async (updated: AppConfig) => {
      setSaving(true);
      setError(null);
      try {
        await saveConfig(updated);
        setConfig(updated);
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
      } finally {
        setSaving(false);
      }
    },
    [],
  );

  const update = useCallback(
    (partial: Partial<AppConfig>) => {
      if (config) {
        setConfig({ ...config, ...partial });
      }
    },
    [config],
  );

  return { config, loading, error, saving, save, update, reload: load };
}
