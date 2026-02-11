import type { AppConfig } from "../../types";

interface HardwareSettingsProps {
  config: AppConfig;
  onChange: (config: AppConfig) => void;
}

export function HardwareSettings({ config, onChange }: HardwareSettingsProps) {
  const hw = config.hardware;

  const updateHw = (partial: Partial<AppConfig["hardware"]>) => {
    onChange({
      ...config,
      hardware: { ...hw, ...partial },
    });
  };

  return (
    <section className="space-y-4">
      <h3 className="text-sm font-semibold text-zinc-300 uppercase tracking-wider">
        Hardware / Throttling
      </h3>
      <div className="space-y-3">
        <label className="block">
          <span className="text-sm text-zinc-400">
            Cooldown between generations (seconds)
          </span>
          <input
            type="number"
            min={0}
            value={hw.cooldownSeconds}
            onChange={(e) =>
              updateHw({ cooldownSeconds: parseInt(e.target.value) || 0 })
            }
            className="mt-1 block w-32 bg-zinc-700 border border-zinc-600 rounded px-3 py-2 text-sm text-zinc-100 focus:border-blue-500 focus:outline-none"
          />
        </label>
        <label className="block">
          <span className="text-sm text-zinc-400">
            Max consecutive generations before forced cooldown
          </span>
          <input
            type="number"
            min={0}
            value={hw.maxConsecutiveGenerations}
            onChange={(e) =>
              updateHw({
                maxConsecutiveGenerations: parseInt(e.target.value) || 0,
              })
            }
            className="mt-1 block w-32 bg-zinc-700 border border-zinc-600 rounded px-3 py-2 text-sm text-zinc-100 focus:border-blue-500 focus:outline-none"
          />
        </label>

        <div className="pt-2 border-t border-zinc-700">
          <label className="flex items-center gap-3 cursor-pointer mb-3">
            <input
              type="checkbox"
              checked={hw.enableHaPowerMonitoring}
              onChange={() =>
                updateHw({
                  enableHaPowerMonitoring: !hw.enableHaPowerMonitoring,
                })
              }
              className="w-4 h-4 rounded bg-zinc-700 border-zinc-600 text-blue-500 focus:ring-blue-500 focus:ring-offset-zinc-800"
            />
            <span className="text-sm text-zinc-300">
              Home Assistant Power Monitoring
            </span>
          </label>
          {hw.enableHaPowerMonitoring && (
            <div className="space-y-3 pl-7">
              <label className="block">
                <span className="text-sm text-zinc-400">HA Entity ID</span>
                <input
                  type="text"
                  value={hw.haEntityId}
                  onChange={(e) => updateHw({ haEntityId: e.target.value })}
                  className="mt-1 block w-full bg-zinc-700 border border-zinc-600 rounded px-3 py-2 text-sm text-zinc-100 focus:border-blue-500 focus:outline-none"
                />
              </label>
              <label className="block">
                <span className="text-sm text-zinc-400">Max Watts</span>
                <input
                  type="number"
                  min={0}
                  value={hw.haMaxWatts}
                  onChange={(e) =>
                    updateHw({ haMaxWatts: parseInt(e.target.value) || 0 })
                  }
                  className="mt-1 block w-32 bg-zinc-700 border border-zinc-600 rounded px-3 py-2 text-sm text-zinc-100 focus:border-blue-500 focus:outline-none"
                />
              </label>
            </div>
          )}
        </div>
      </div>
    </section>
  );
}
