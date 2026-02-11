import { useState, useEffect } from "react";
import {
  Settings2,
  ChevronDown,
  Zap,
  Gem,
  Crown,
  Shuffle,
  Link,
  Unlink,
  ArrowUpDown,
} from "lucide-react";
import {
  getComfyuiCheckpoints,
  getComfyuiSamplers,
  getComfyuiSchedulers,
} from "../../api/comfyui";
import type { AppConfig, GenSettings, QualityPreset } from "../../types";

export interface GenerationControlsProps {
  config: AppConfig | null;
  settings: GenSettings;
  onChange: (settings: GenSettings) => void;
  disabled?: boolean;
}

const ASPECT_RATIOS = [
  { label: "1:1", w: 512, h: 512 },
  { label: "2:3", w: 512, h: 768 },
  { label: "3:2", w: 768, h: 512 },
  { label: "1:1 HD", w: 768, h: 768 },
];

const PRESET_ICONS: Record<string, typeof Zap> = {
  quick_draft: Zap,
  quality: Gem,
  max_effort: Crown,
};

export function getDefaultSettings(config: AppConfig | null): GenSettings {
  const preset = config?.presets?.["quality"];
  return {
    checkpoint: "",
    sampler: preset?.sampler ?? "dpmpp_2m",
    scheduler: preset?.scheduler ?? "karras",
    steps: preset?.steps ?? 25,
    cfg: preset?.cfg ?? 7.5,
    width: preset?.width ?? 512,
    height: preset?.height ?? 768,
    seed: -1,
    batchCount: 1,
  };
}

function presetMatches(settings: GenSettings, preset: QualityPreset): boolean {
  return (
    settings.steps === preset.steps &&
    settings.cfg === preset.cfg &&
    settings.width === preset.width &&
    settings.height === preset.height &&
    settings.sampler === preset.sampler &&
    settings.scheduler === preset.scheduler
  );
}

function clampDim(v: number): number {
  return Math.max(256, Math.min(2048, Math.round(v / 64) * 64));
}

function stripExtension(filename: string): string {
  return filename.replace(/\.(safetensors|ckpt|pt|bin)$/i, "");
}

export function GenerationControls({
  config,
  settings,
  onChange,
  disabled,
}: GenerationControlsProps) {
  const [expanded, setExpanded] = useState(false);
  const [checkpoints, setCheckpoints] = useState<string[]>([]);
  const [samplers, setSamplers] = useState<string[]>([]);
  const [schedulers, setSchedulers] = useState<string[]>([]);
  const [linkedDimensions, setLinkedDimensions] = useState(false);

  useEffect(() => {
    let cancelled = false;
    getComfyuiCheckpoints()
      .then((cp) => {
        if (cancelled) return;
        setCheckpoints(cp);
        if (!settings.checkpoint && cp.length > 0) {
          onChange({ ...settings, checkpoint: cp[0] });
        }
      })
      .catch(() => {});
    getComfyuiSamplers()
      .then((s) => !cancelled && setSamplers(s))
      .catch(() => {});
    getComfyuiSchedulers()
      .then((s) => !cancelled && setSchedulers(s))
      .catch(() => {});
    return () => { cancelled = true; };
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  const activePresetName = config
    ? Object.entries(config.presets).find(([, p]) => presetMatches(settings, p))?.[0]
    : undefined;

  const applyPreset = (name: string) => {
    const preset = config?.presets?.[name];
    if (!preset) return;
    onChange({
      ...settings,
      steps: preset.steps,
      cfg: preset.cfg,
      width: preset.width,
      height: preset.height,
      sampler: preset.sampler,
      scheduler: preset.scheduler,
    });
  };

  const handleWidthChange = (newWidth: number) => {
    const clamped = clampDim(newWidth);
    if (linkedDimensions && settings.width !== 0) {
      const ratio = clamped / settings.width;
      onChange({
        ...settings,
        width: clamped,
        height: clampDim(settings.height * ratio),
      });
    } else {
      onChange({ ...settings, width: clamped });
    }
  };

  const handleHeightChange = (newHeight: number) => {
    const clamped = clampDim(newHeight);
    if (linkedDimensions && settings.height !== 0) {
      const ratio = clamped / settings.height;
      onChange({
        ...settings,
        height: clamped,
        width: clampDim(settings.width * ratio),
      });
    } else {
      onChange({ ...settings, height: clamped });
    }
  };

  const swapDimensions = () => {
    onChange({ ...settings, width: settings.height, height: settings.width });
  };

  const summaryParts = [
    settings.checkpoint ? stripExtension(settings.checkpoint) : "No checkpoint",
    `${settings.steps} steps`,
    `${settings.width}\u00d7${settings.height}`,
    settings.sampler,
  ];
  if (settings.seed !== -1) summaryParts.push(`seed ${settings.seed}`);
  if (settings.batchCount > 1) summaryParts.push(`\u00d7${settings.batchCount}`);

  return (
    <div
      className={`bg-zinc-800 border ${expanded ? "border-zinc-600" : "border-zinc-700"} rounded-lg transition-colors duration-200 ${disabled ? "opacity-50 pointer-events-none" : ""}`}
    >
      {/* Header */}
      <button
        onClick={() => setExpanded(!expanded)}
        className="w-full flex items-center gap-3 px-4 py-3 text-left cursor-pointer"
      >
        <Settings2 size={16} className="text-zinc-400 shrink-0" />
        <span className="text-sm font-medium text-zinc-200">
          Generation Settings
        </span>
        {activePresetName && (
          <span className="px-2 py-0.5 text-xs rounded-full bg-blue-500/20 text-blue-400 border border-blue-500/30">
            {activePresetName.replace(/_/g, " ")}
          </span>
        )}
        <ChevronDown
          size={16}
          className={`text-zinc-400 ml-auto transition-transform duration-200 ${expanded ? "rotate-180" : ""}`}
        />
      </button>

      {/* Collapsed summary */}
      {!expanded && (
        <div className="px-4 pb-3 -mt-1">
          <p className="text-xs text-zinc-500">
            {summaryParts.join("  \u00b7  ")}
          </p>
        </div>
      )}

      {/* Expandable body */}
      <div
        className={`transition-all duration-300 ease-in-out overflow-hidden ${expanded ? "max-h-[600px] opacity-100" : "max-h-0 opacity-0"}`}
      >
        <div className="px-4 pb-4 space-y-4 border-t border-zinc-700/50">
          {/* Presets */}
          {config && Object.keys(config.presets).length > 0 && (
            <div className="pt-3">
              <span className="text-xs text-zinc-500 block mb-2">Presets</span>
              <div className="flex flex-wrap gap-2">
                {Object.entries(config.presets).map(([name]) => {
                  const isActive = activePresetName === name;
                  const Icon = PRESET_ICONS[name] ?? Settings2;
                  return (
                    <button
                      key={name}
                      onClick={() => applyPreset(name)}
                      className={`flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-full border transition-colors ${
                        isActive
                          ? "bg-blue-500/20 text-blue-400 border-blue-500/30"
                          : "bg-zinc-700/50 text-zinc-400 border-zinc-600/50 hover:bg-zinc-700"
                      }`}
                    >
                      <Icon size={12} />
                      {name.replace(/_/g, " ")}
                    </button>
                  );
                })}
              </div>
            </div>
          )}

          {/* Checkpoint */}
          <label className="block">
            <span className="text-xs text-zinc-500">Checkpoint</span>
            {checkpoints.length > 0 ? (
              <select
                value={settings.checkpoint}
                onChange={(e) =>
                  onChange({ ...settings, checkpoint: e.target.value })
                }
                className="mt-1 block w-full bg-zinc-700 border border-zinc-600 rounded px-2 py-1.5 text-sm text-zinc-100 focus:border-blue-500 focus:outline-none"
              >
                {checkpoints.map((cp) => (
                  <option key={cp} value={cp}>
                    {cp}
                  </option>
                ))}
              </select>
            ) : (
              <input
                type="text"
                value={settings.checkpoint}
                onChange={(e) =>
                  onChange({ ...settings, checkpoint: e.target.value })
                }
                placeholder="No checkpoints found (ComfyUI offline?)"
                className="mt-1 block w-full bg-zinc-700 border border-zinc-600 rounded px-2 py-1.5 text-sm text-zinc-100 placeholder-zinc-500 focus:border-blue-500 focus:outline-none"
              />
            )}
          </label>

          {/* Steps / CFG / Batch Count */}
          <div className="grid grid-cols-3 gap-3">
            <label className="block">
              <span className="text-xs text-zinc-500">Steps</span>
              <input
                type="number"
                min={1}
                max={150}
                value={settings.steps}
                onChange={(e) =>
                  onChange({
                    ...settings,
                    steps: parseInt(e.target.value) || 1,
                  })
                }
                className="mt-1 block w-full bg-zinc-700 border border-zinc-600 rounded px-2 py-1.5 text-sm text-zinc-100 focus:border-blue-500 focus:outline-none"
              />
            </label>
            <label className="block">
              <span className="text-xs text-zinc-500">CFG Scale</span>
              <input
                type="number"
                step={0.5}
                min={1}
                max={30}
                value={settings.cfg}
                onChange={(e) =>
                  onChange({
                    ...settings,
                    cfg: parseFloat(e.target.value) || 1,
                  })
                }
                className="mt-1 block w-full bg-zinc-700 border border-zinc-600 rounded px-2 py-1.5 text-sm text-zinc-100 focus:border-blue-500 focus:outline-none"
              />
            </label>
            <label className="block">
              <span className="text-xs text-zinc-500">Batch Count</span>
              <input
                type="number"
                min={1}
                max={20}
                value={settings.batchCount}
                onChange={(e) =>
                  onChange({
                    ...settings,
                    batchCount: parseInt(e.target.value) || 1,
                  })
                }
                className="mt-1 block w-full bg-zinc-700 border border-zinc-600 rounded px-2 py-1.5 text-sm text-zinc-100 focus:border-blue-500 focus:outline-none"
              />
            </label>
          </div>

          {/* Sampler / Scheduler / Seed */}
          <div className="grid grid-cols-3 gap-3">
            <label className="block">
              <span className="text-xs text-zinc-500">Sampler</span>
              {samplers.length > 0 ? (
                <select
                  value={settings.sampler}
                  onChange={(e) =>
                    onChange({ ...settings, sampler: e.target.value })
                  }
                  className="mt-1 block w-full bg-zinc-700 border border-zinc-600 rounded px-2 py-1.5 text-sm text-zinc-100 focus:border-blue-500 focus:outline-none"
                >
                  {samplers.map((s) => (
                    <option key={s} value={s}>
                      {s}
                    </option>
                  ))}
                </select>
              ) : (
                <input
                  type="text"
                  value={settings.sampler}
                  onChange={(e) =>
                    onChange({ ...settings, sampler: e.target.value })
                  }
                  className="mt-1 block w-full bg-zinc-700 border border-zinc-600 rounded px-2 py-1.5 text-sm text-zinc-100 focus:border-blue-500 focus:outline-none"
                />
              )}
            </label>
            <label className="block">
              <span className="text-xs text-zinc-500">Scheduler</span>
              {schedulers.length > 0 ? (
                <select
                  value={settings.scheduler}
                  onChange={(e) =>
                    onChange({ ...settings, scheduler: e.target.value })
                  }
                  className="mt-1 block w-full bg-zinc-700 border border-zinc-600 rounded px-2 py-1.5 text-sm text-zinc-100 focus:border-blue-500 focus:outline-none"
                >
                  {schedulers.map((s) => (
                    <option key={s} value={s}>
                      {s}
                    </option>
                  ))}
                </select>
              ) : (
                <input
                  type="text"
                  value={settings.scheduler}
                  onChange={(e) =>
                    onChange({ ...settings, scheduler: e.target.value })
                  }
                  className="mt-1 block w-full bg-zinc-700 border border-zinc-600 rounded px-2 py-1.5 text-sm text-zinc-100 focus:border-blue-500 focus:outline-none"
                />
              )}
            </label>
            <label className="block">
              <span className="text-xs text-zinc-500">Seed</span>
              <div className="mt-1 flex">
                <input
                  type="number"
                  value={settings.seed}
                  onChange={(e) =>
                    onChange({
                      ...settings,
                      seed: parseInt(e.target.value) || -1,
                    })
                  }
                  className="block w-full bg-zinc-700 border border-zinc-600 rounded-l px-2 py-1.5 text-sm text-zinc-100 focus:border-blue-500 focus:outline-none"
                />
                <button
                  onClick={() => onChange({ ...settings, seed: -1 })}
                  title="Random seed"
                  className="px-2 bg-zinc-700 border border-l-0 border-zinc-600 rounded-r text-zinc-400 hover:text-zinc-200 hover:bg-zinc-600"
                >
                  <Shuffle size={14} />
                </button>
              </div>
            </label>
          </div>

          {/* Dimensions */}
          <div>
            <div className="flex items-center justify-between mb-2">
              <span className="text-xs text-zinc-500">Dimensions</span>
              <div className="flex gap-1">
                {ASPECT_RATIOS.map((ar) => {
                  const isActive =
                    settings.width === ar.w && settings.height === ar.h;
                  return (
                    <button
                      key={ar.label}
                      onClick={() =>
                        onChange({
                          ...settings,
                          width: ar.w,
                          height: ar.h,
                        })
                      }
                      className={`px-1.5 py-0.5 text-[10px] rounded border transition-colors ${
                        isActive
                          ? "bg-blue-500/20 text-blue-400 border-blue-500/30"
                          : "bg-zinc-700/50 text-zinc-500 border-zinc-600/50 hover:text-zinc-400"
                      }`}
                    >
                      {ar.label}
                    </button>
                  );
                })}
              </div>
            </div>
            <div className="flex items-center gap-2">
              <div className="relative flex-1">
                <input
                  type="number"
                  step={64}
                  min={256}
                  max={2048}
                  value={settings.width}
                  onChange={(e) =>
                    handleWidthChange(parseInt(e.target.value) || 512)
                  }
                  className="block w-full bg-zinc-700 border border-zinc-600 rounded px-2 py-1.5 pr-7 text-sm text-zinc-100 focus:border-blue-500 focus:outline-none"
                />
                <span className="absolute right-2 top-1/2 -translate-y-1/2 text-[10px] text-zinc-500 pointer-events-none">
                  W
                </span>
              </div>
              <button
                onClick={() => setLinkedDimensions(!linkedDimensions)}
                title={linkedDimensions ? "Unlink dimensions" : "Link dimensions"}
                className="p-1.5 text-zinc-400 hover:text-zinc-200 rounded"
              >
                {linkedDimensions ? <Link size={14} /> : <Unlink size={14} />}
              </button>
              <div className="relative flex-1">
                <input
                  type="number"
                  step={64}
                  min={256}
                  max={2048}
                  value={settings.height}
                  onChange={(e) =>
                    handleHeightChange(parseInt(e.target.value) || 512)
                  }
                  className="block w-full bg-zinc-700 border border-zinc-600 rounded px-2 py-1.5 pr-7 text-sm text-zinc-100 focus:border-blue-500 focus:outline-none"
                />
                <span className="absolute right-2 top-1/2 -translate-y-1/2 text-[10px] text-zinc-500 pointer-events-none">
                  H
                </span>
              </div>
              <button
                onClick={swapDimensions}
                disabled={settings.width === settings.height}
                title="Swap width and height"
                className="p-1.5 text-zinc-400 hover:text-zinc-200 disabled:opacity-30 disabled:cursor-not-allowed rounded"
              >
                <ArrowUpDown size={14} />
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
