// ============================================
// Pipeline Types
// ============================================

export interface PipelineResult {
  originalIdea: string;
  pipelineConfig: PipelineConfig;
  stages: PipelineStages;
  userEdits?: UserEdits;
  autoApproved: boolean;
  generationSettings?: GenerationSettings;
}

export interface PipelineConfig {
  stagesEnabled: [boolean, boolean, boolean, boolean, boolean];
  modelsUsed: ModelsUsed;
}

export interface ModelsUsed {
  ideator?: string;
  composer?: string;
  judge?: string;
  promptEngineer?: string;
  reviewer?: string;
}

export interface PipelineStages {
  ideator?: IdeatorOutput;
  composer?: ComposerOutput;
  judge?: JudgeOutput;
  promptEngineer?: PromptEngineerOutput;
  reviewer?: ReviewerOutput;
}

export interface IdeatorOutput {
  input: string;
  output: string[];
  durationMs: number;
  model: string;
  tokensIn?: number;
  tokensOut?: number;
}

export interface ComposerOutput {
  inputConceptIndex: number;
  input: string;
  output: string;
  durationMs: number;
  model: string;
  tokensIn?: number;
  tokensOut?: number;
}

export interface JudgeRanking {
  rank: number;
  conceptIndex: number;
  score: number;
  reasoning: string;
}

export interface JudgeOutput {
  input: string[];
  output: JudgeRanking[];
  durationMs: number;
  model: string;
}

export interface PromptPair {
  positive: string;
  negative: string;
}

export interface PromptEngineerOutput {
  input: string;
  checkpointContext?: string;
  output: PromptPair;
  durationMs: number;
  model: string;
  tokensIn?: number;
  tokensOut?: number;
}

export interface ReviewerOutput {
  approved: boolean;
  issues?: string[];
  suggestedPositive?: string;
  suggestedNegative?: string;
  durationMs: number;
  model: string;
}

export interface UserEdits {
  promptEdited: boolean;
  editDiff?: EditDiff;
}

export interface EditDiff {
  positiveAdded: string[];
  positiveRemoved: string[];
  negativeAdded: string[];
  negativeRemoved: string[];
}

export interface GenerationSettings {
  checkpoint: string;
  seed: number;
  steps: number;
  cfg: number;
  sampler: string;
  scheduler: string;
  width: number;
  height: number;
}

// ============================================
// Generation Controls (UI state for PromptStudio)
// ============================================

export interface GenSettings {
  checkpoint: string;
  sampler: string;
  scheduler: string;
  steps: number;
  cfg: number;
  width: number;
  height: number;
  seed: number;
  batchCount: number;
}

// ============================================
// Generation Types
// ============================================

export interface GenerationRequest {
  positivePrompt: string;
  negativePrompt: string;
  checkpoint: string;
  width: number;
  height: number;
  steps: number;
  cfgScale: number;
  sampler: string;
  scheduler: string;
  seed: number;
  batchSize: number;
}

export type GenerationStatusKind =
  | "queued"
  | "generating"
  | "completed"
  | "failed";

export interface GenerationStatus {
  promptId: string;
  status: GenerationStatusKind;
  progress?: number;
  currentStep?: number;
  totalSteps?: number;
  imageFilenames?: string[];
  error?: string;
}

// ============================================
// Gallery Types
// ============================================

export interface ImageEntry {
  id: string;
  filename: string;
  createdAt: string;
  positivePrompt?: string;
  negativePrompt?: string;
  originalIdea?: string;
  checkpoint?: string;
  width?: number;
  height?: number;
  steps?: number;
  cfgScale?: number;
  sampler?: string;
  scheduler?: string;
  seed?: number;
  pipelineLog?: string;
  selectedConcept?: number;
  autoApproved: boolean;
  caption?: string;
  captionEdited: boolean;
  rating?: number;
  favorite: boolean;
  deleted: boolean;
  userNote?: string;
  tags?: TagEntry[];
}

export interface TagEntry {
  id: number;
  name: string;
  source?: string;
  confidence?: number;
}

export type GallerySortField = "createdAt" | "rating" | "random";
export type SortOrder = "asc" | "desc";

export interface GalleryFilter {
  search?: string;
  tags?: string[];
  checkpoint?: string;
  minRating?: number;
  favoriteOnly?: boolean;
  showDeleted?: boolean;
  autoApproved?: boolean;
  sortBy?: GallerySortField;
  sortOrder?: SortOrder;
  limit?: number;
  offset?: number;
}

// ============================================
// Seed Types
// ============================================

export interface SeedEntry {
  id?: number;
  seedValue: number;
  comment: string;
  checkpoint?: string;
  sampleImageId?: string;
  createdAt?: string;
  tags?: string[];
}

export interface SeedCheckpointNote {
  seedId: number;
  checkpoint: string;
  note: string;
  sampleImageId?: string;
}

export interface SeedFilter {
  search?: string;
  checkpoint?: string;
  tags?: string[];
}

// ============================================
// Checkpoint Types
// ============================================

export interface CheckpointProfile {
  id?: number;
  filename: string;
  displayName?: string;
  baseModel?: string;
  createdAt?: string;
  strengths?: string[];
  weaknesses?: string[];
  preferredCfg?: number;
  cfgRangeLow?: number;
  cfgRangeHigh?: number;
  preferredSampler?: string;
  preferredScheduler?: string;
  optimalResolution?: string;
  notes?: string;
}

export type TermStrength = "strong" | "moderate" | "weak" | "broken";

export interface PromptTerm {
  id?: number;
  checkpointId: number;
  term: string;
  effect: string;
  strength: TermStrength;
  exampleImageId?: string;
  createdAt?: string;
}

export type ObservationSource =
  | "user"
  | "abComparison"
  | "pipelineNote"
  | "autoRating";

export interface CheckpointObservation {
  id?: number;
  checkpointId: number;
  observation: string;
  source: ObservationSource;
  comparisonId?: string;
  createdAt?: string;
}

// ============================================
// Comparison Types
// ============================================

export interface Comparison {
  id: string;
  imageAId: string;
  imageBId: string;
  variableChanged: string;
  note?: string;
  createdAt?: string;
}

// ============================================
// Queue Types
// ============================================

export type QueuePriority = "high" | "normal" | "low";

export type QueueJobStatus =
  | "pending"
  | "generating"
  | "completed"
  | "failed"
  | "cancelled";

export interface QueueJob {
  id: string;
  priority: QueuePriority;
  status: QueueJobStatus;
  positivePrompt: string;
  negativePrompt: string;
  settingsJson: string;
  pipelineLog?: string;
  originalIdea?: string;
  linkedComparisonId?: string;
  createdAt?: string;
  startedAt?: string;
  completedAt?: string;
  resultImageId?: string;
}

// ============================================
// Config Types
// ============================================

export interface AppConfig {
  comfyui: ComfyUiConfig;
  ollama: OllamaConfig;
  models: ModelAssignments;
  pipeline: PipelineSettings;
  hardware: HardwareSettings;
  presets: Record<string, QualityPreset>;
  storage: StorageSettings;
}

export interface StorageSettings {
  imageDirectory: string;
}

export interface ComfyUiConfig {
  endpoint: string;
}

export interface OllamaConfig {
  endpoint: string;
}

export interface ModelAssignments {
  ideator: string;
  composer: string;
  judge: string;
  promptEngineer: string;
  reviewer: string;
  tagger: string;
  captioner: string;
}

export interface PipelineSettings {
  enableIdeator: boolean;
  enableComposer: boolean;
  enableJudge: boolean;
  enablePromptEngineer: boolean;
  enableReviewer: boolean;
  autoApprove: boolean;
}

export interface HardwareSettings {
  cooldownSeconds: number;
  maxConsecutiveGenerations: number;
  enableHaPowerMonitoring: boolean;
  haEntityId: string;
  haMaxWatts: number;
}

export interface QualityPreset {
  steps: number;
  cfg: number;
  width: number;
  height: number;
  sampler: string;
  scheduler: string;
}
