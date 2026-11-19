export type Severity = "critical" | "warning" | "info" | "pass";

export type FindingCategory =
  | "description"
  | "screenshots"
  | "trailers"
  | "localization"
  | "support"
  | "features"
  | "platforms"
  | "general";

export interface FindingEvidence {
  field?: string;
  actual?: unknown;
  expected?: string;
  threshold?: unknown;
}

export interface Finding {
  id: string;
  category: FindingCategory;
  severity: Severity;
  title: string;
  message: string;
  recommendation: string;
  evidence: FindingEvidence;
  rule_version: string;
}

export interface Screenshot {
  id: number;
  url_thumbnail: string;
  url_full: string;
}

export interface Movie {
  id: number;
  name: string;
  thumbnail_url?: string;
}

export interface PlatformSupport {
  windows: boolean;
  mac: boolean;
  linux: boolean;
}

export interface FieldProvenance {
  field: string;
  known: boolean;
  source: string;
  notes?: string;
}

export interface StoreListing {
  app_id: number;
  name: string;
  listing_type?: string;
  release_date?: string;
  release_year?: number;
  short_description?: string;
  detailed_description?: string;
  about_the_game?: string;
  developers: string[];
  publishers: string[];
  supported_languages?: string[];
  full_audio_languages?: string[];
  screenshots?: Screenshot[];
  movies?: Movie[];
  website?: string;
  support_url?: string;
  support_email?: string;
  platforms?: PlatformSupport;
  categories?: string[];
  genres?: string[];
  header_image?: string;
  store_url?: string;
  source:
    "steam_store_api" | "json_upload" | "benchmark_dataset" | "sample_fixture";
  provenance: Record<string, FieldProvenance>;
  fetched_at?: string;
  country?: string;
  language?: string;
}

export interface BenchmarkDistribution {
  sample_size: number;
  usable_records: number;
  coverage_pct: number;
  listing_value?: number;
  min?: number;
  max?: number;
  p25?: number;
  median?: number;
  p75?: number;
  percentile_rank?: number;
  share_above?: number;
  share_below?: number;
}

export interface CategoricalBenchmark {
  sample_size: number;
  usable_records: number;
  coverage_pct: number;
  listing_has_feature?: boolean;
  true_count: number;
  false_count: number;
  true_percentage: number;
}

export type BenchmarkStatus =
  | "calculated"
  | "insufficient_sample"
  | "missing_listing_data"
  | "dataset_unavailable";

export interface BenchmarkResult {
  metric_id: string;
  metric_name: string;
  category: FindingCategory;
  listing_value_display: string;
  status: BenchmarkStatus;
  distribution?: BenchmarkDistribution;
  categorical?: CategoricalBenchmark;
  cohort_definition: string;
  explanation?: string;
}

export interface CohortInfo {
  cohort_name: string;
  total_games_in_cohort: number;
  genres: string[];
  release_year?: number;
  is_indie_restricted: boolean;
  fallback_applied: boolean;
  explanation: string;
}

export interface BenchmarkMethodology {
  dataset_source: string;
  dataset_license: string;
  snapshot_date: string;
  total_dataset_games: number;
  min_sample_size_threshold: number;
  percentile_convention: string;
  disclaimer: string;
}

export interface AuditSummary {
  critical: number;
  warnings: number;
  info: number;
  passed: number;
  unknown: number;
  total_checks: number;
  section_coverage: Record<string, number>;
}

export interface AuditDataQuality {
  fields_evaluated: number;
  fields_known: number;
  fields_missing_from_source: number;
  data_completeness_pct: number;
}

export interface AuditReport {
  schema_version: string;
  audit_id: string;
  generated_at: string;
  audit_profile: "balanced" | "strict";
  listing: StoreListing;
  summary: AuditSummary;
  findings: Finding[];
  benchmarks: BenchmarkResult[];
  cohort?: CohortInfo;
  methodology: BenchmarkMethodology;
  data_quality: AuditDataQuality;
  provenance: Record<string, FieldProvenance>;
}
