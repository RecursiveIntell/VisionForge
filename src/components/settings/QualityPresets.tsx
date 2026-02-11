import type { AppConfig, QualityPreset } from "../../types";

interface QualityPresetsProps {
  config: AppConfig;
  onChange: (config: AppConfig) => void;
}

export function QualityPresets({ config, onChange }: QualityPresetsProps) {
  const presets = Object.entries(config.presets);

  const updatePreset = (name: string, field: keyof QualityPreset, value: string | number) => {
    onChange({
      ...config,
      presets: {
        ...config.presets,
        [name]: { ...config.presets[name], [field]: value },
      },
    });
  };

  return (
    <section className="space-y-4">
      <h3 className="text-sm font-semibold text-zinc-300 uppercase tracking-wider">
        Quality Presets
      </h3>
      <div className="space-y-4">
        {presets.map(([name, preset]) => (
          <div
            key={name}
            className="bg-zinc-800/50 border border-zinc-700 rounded p-4 space-y-3"
          >
            <h4 className="text-sm font-medium text-zinc-200 capitalize">
              {name.replace(/_/g, " ")}
            </h4>
            <div className="grid grid-cols-2 sm:grid-cols-3 gap-3">
              <label className="block">
                <span className="text-xs text-zinc-500">Steps</span>
                <input
                  type="number"
                  min={1}
                  value={preset.steps}
                  onChange={(e) => updatePreset(name, "steps", parseInt(e.target.value) || 1)}
                  className="mt-1 block w-full bg-zinc-700 border border-zinc-600 rounded px-2 py-1.5 text-sm text-zinc-100 focus:border-blue-500 focus:outline-none"
                />
              </label>
              <label className="block">
                <span className="text-xs text-zinc-500">CFG</span>
                <input
                  type="number"
                  step={0.5}
                  min={1}
                  value={preset.cfg}
                  onChange={(e) => updatePreset(name, "cfg", parseFloat(e.target.value) || 1)}
                  className="mt-1 block w-full bg-zinc-700 border border-zinc-600 rounded px-2 py-1.5 text-sm text-zinc-100 focus:border-blue-500 focus:outline-none"
                />
              </label>
              <label className="block">
                <span className="text-xs text-zinc-500">Width</span>
                <input
                  type="number"
                  step={64}
                  min={256}
                  value={preset.width}
                  onChange={(e) => updatePreset(name, "width", parseInt(e.target.value) || 512)}
                  className="mt-1 block w-full bg-zinc-700 border border-zinc-600 rounded px-2 py-1.5 text-sm text-zinc-100 focus:border-blue-500 focus:outline-none"
                />
              </label>
              <label className="block">
                <span className="text-xs text-zinc-500">Height</span>
                <input
                  type="number"
                  step={64}
                  min={256}
                  value={preset.height}
                  onChange={(e) => updatePreset(name, "height", parseInt(e.target.value) || 512)}
                  className="mt-1 block w-full bg-zinc-700 border border-zinc-600 rounded px-2 py-1.5 text-sm text-zinc-100 focus:border-blue-500 focus:outline-none"
                />
              </label>
              <label className="block">
                <span className="text-xs text-zinc-500">Sampler</span>
                <input
                  type="text"
                  value={preset.sampler}
                  onChange={(e) => updatePreset(name, "sampler", e.target.value)}
                  className="mt-1 block w-full bg-zinc-700 border border-zinc-600 rounded px-2 py-1.5 text-sm text-zinc-100 focus:border-blue-500 focus:outline-none"
                />
              </label>
              <label className="block">
                <span className="text-xs text-zinc-500">Scheduler</span>
                <input
                  type="text"
                  value={preset.scheduler}
                  onChange={(e) => updatePreset(name, "scheduler", e.target.value)}
                  className="mt-1 block w-full bg-zinc-700 border border-zinc-600 rounded px-2 py-1.5 text-sm text-zinc-100 focus:border-blue-500 focus:outline-none"
                />
              </label>
            </div>
          </div>
        ))}
      </div>
    </section>
  );
}
