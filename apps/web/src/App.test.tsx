import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { App } from "./App";
import * as api from "./api/client";
import { AuditReport } from "./types";

const mockReport: AuditReport = {
  schema_version: "1.0.0",
  audit_id: "test-uuid-1234",
  generated_at: "2024-11-19T00:00:00Z",
  audit_profile: "balanced",
  listing: {
    app_id: 2539430,
    name: "Dragon Forge: Chronicles",
    short_description: "A great indie game.",
    detailed_description: "An expansive action adventure dungeon crawler.",
    developers: ["Emberstone Studios"],
    publishers: ["Emberstone Studios"],
    source: "json_upload",
    provenance: {},
  },
  summary: {
    critical: 0,
    warnings: 1,
    info: 2,
    passed: 5,
    unknown: 0,
    total_checks: 8,
    section_coverage: { Description: 100, Screenshots: 50 },
  },
  findings: [
    {
      id: "screenshots.too_few",
      category: "screenshots",
      severity: "warning",
      title: "Limited screenshot coverage",
      message: "Listing only contains 2 screenshots.",
      recommendation: "Upload at least 5 screenshots.",
      evidence: { actual: 2 },
      rule_version: "1.0.0",
    },
    {
      id: "description.short.pass",
      category: "description",
      severity: "pass",
      title: "Short description present",
      message: "Short description is well proportioned.",
      recommendation: "Keep descriptions up to date.",
      evidence: {},
      rule_version: "1.0.0",
    },
  ],
  benchmarks: [
    {
      metric_id: "screenshots.count",
      metric_name: "Screenshot Count",
      category: "screenshots",
      listing_value_display: "2 Screenshots",
      status: "calculated",
      distribution: {
        sample_size: 50,
        usable_records: 50,
        coverage_pct: 100,
        listing_value: 2,
        min: 1,
        max: 20,
        p25: 4,
        median: 8,
        p75: 12,
        percentile_rank: 15,
        share_above: 85,
        share_below: 10,
      },
      cohort_definition: "Indie games in 2024",
      explanation: "Your listing is at the 15th percentile.",
    },
  ],
  cohort: {
    cohort_name: "Indie games in 2024",
    total_games_in_cohort: 50,
    genres: ["Indie"],
    is_indie_restricted: true,
    fallback_applied: false,
    explanation: "Comparison cohort matching Indie games in 2024",
  },
  methodology: {
    dataset_source: "FronkonGames/steam-games-dataset",
    dataset_license: "MIT",
    snapshot_date: "2024-11",
    total_dataset_games: 215,
    min_sample_size_threshold: 30,
    percentile_convention: "Linear rank interpolation",
    disclaimer: "Benchmarking provides descriptive context only.",
  },
  data_quality: {
    fields_evaluated: 10,
    fields_known: 10,
    fields_missing_from_source: 0,
    data_completeness_pct: 100,
  },
  provenance: {},
};

describe("Steam Storefront Linter Web UI", () => {
  it("renders landing page with audit input form", () => {
    render(<App />);
    expect(
      screen.getAllByText("Steam Storefront Linter")[0],
    ).toBeInTheDocument();
    expect(
      screen.getByText("Audit Steam Store Listing Completeness"),
    ).toBeInTheDocument();
    expect(
      screen.getByPlaceholderText(/e\.g\. 2539430 or https:/i),
    ).toBeInTheDocument();
    expect(screen.getByText("Audit Listing")).toBeInTheDocument();
  });

  it("switches to methodology and dataset view when clicking navbar link", () => {
    render(<App />);
    const methodologyBtn = screen.getByText(/Methodology & Dataset/i);
    fireEvent.click(methodologyBtn);

    expect(
      screen.getByText("Benchmark Methodology & Dataset Provenance"),
    ).toBeInTheDocument();
    expect(
      screen.getAllByText(/FronkonGames\/steam-games-dataset/i)[0],
    ).toBeInTheDocument();
  });

  it("loads sample report and displays findings and benchmarks", async () => {
    vi.spyOn(api, "getSampleReport").mockResolvedValue(mockReport);

    render(<App />);
    const sampleBtn = screen.getByTitle(/Load a demonstration report/i);
    fireEvent.click(sampleBtn);

    await waitFor(() => {
      expect(screen.getByText("Dragon Forge: Chronicles")).toBeInTheDocument();
    });

    expect(screen.getByText("Limited screenshot coverage")).toBeInTheDocument();
    expect(
      screen.getByText("Upload at least 5 screenshots."),
    ).toBeInTheDocument();

    // Switch to peer benchmarks tab
    const benchmarksTab = screen.getByText(/Peer Benchmarks/i);
    fireEvent.click(benchmarksTab);

    expect(screen.getByText("Screenshot Count")).toBeInTheDocument();
    expect(screen.getByText("Indie games in 2024")).toBeInTheDocument();
    expect(screen.getByText("15th")).toBeInTheDocument();
  });
});
