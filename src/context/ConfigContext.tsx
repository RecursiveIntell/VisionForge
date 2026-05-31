import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { getConfig, saveConfig } from "../api/config";
import type { AppConfig } from "../types";

interface ConfigContextValue {
  config: AppConfig | null;
  loading: boolean;
  error: string | null;
  saving: boolean;
  save: (updated?: AppConfig) => Promise<boolean>;
  update: (updated: AppConfig | ((current: AppConfig) => AppConfig)) => void;
  reload: () => Promise<void>;
}

const ConfigContext = createContext<ConfigContextValue | null>(null);

export function ConfigProvider({ children }: { children: ReactNode }) {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  const reload = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      setConfig(await getConfig());
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void reload();
  }, [reload]);

  const update = useCallback(
    (updated: AppConfig | ((current: AppConfig) => AppConfig)) => {
      setConfig((current) => {
        if (!current) return current;
        return typeof updated === "function" ? updated(current) : updated;
      });
    },
    [],
  );

  const save = useCallback(
    async (updated?: AppConfig): Promise<boolean> => {
      const nextConfig = updated ?? config;
      if (!nextConfig) return false;

      setSaving(true);
      setError(null);
      try {
        await saveConfig(nextConfig);
        setConfig(nextConfig);
        window.dispatchEvent(new CustomEvent("visionforge:config-changed"));
        return true;
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
        return false;
      } finally {
        setSaving(false);
      }
    },
    [config],
  );

  const value = useMemo(
    () => ({ config, loading, error, saving, save, update, reload }),
    [config, loading, error, saving, save, update, reload],
  );

  return (
    <ConfigContext.Provider value={value}>{children}</ConfigContext.Provider>
  );
}

export function useConfigContext() {
  const value = useContext(ConfigContext);
  if (!value) {
    throw new Error("useConfig must be used within ConfigProvider");
  }
  return value;
}
