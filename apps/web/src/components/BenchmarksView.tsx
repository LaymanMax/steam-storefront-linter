import { AlertCircle, Calendar, Info, Users } from "lucide-react";
import React from "react";
import { BenchmarkMethodology, BenchmarkResult, CohortInfo } from "../types";
import { DistributionBar } from "./DistributionBar";

interface BenchmarksViewProps {
  benchmarks: BenchmarkResult[];
  cohort?: CohortInfo;
  methodology: BenchmarkMethodology;
}

export const BenchmarksView: React.FC<BenchmarksViewProps> = ({
  benchmarks,
  cohort,
  methodology,
}) => {
  if (benchmarks.length === 0) {
    return (
      <div
        className="audit-card"
        style={{ textAlign: "center", padding: "3rem 1.5rem" }}
      >
        <AlertCircle
          size={32}
          color="var(--color-warning)"
          style={{ margin: "0 auto 1rem" }}
        />
        <h3
          style={{ fontSize: "1.2rem", color: "#fff", marginBottom: "0.5rem" }}
        >
          Peer Benchmarking Unavailable
        </h3>
        <p
          style={{
            color: "var(--text-muted)",
            maxWidth: "500px",
            margin: "0 auto 1.5rem",
            fontSize: "0.9rem",
          }}
        >
          The benchmark dataset has not yet been ingested into the local SQLite
          database.
        </p>
        <div
          style={{
            background: "#0b0f16",
            padding: "1rem",
            borderRadius: "6px",
            maxWidth: "420px",
            margin: "0 auto",
            textAlign: "left",
            fontFamily: "var(--font-mono)",
            fontSize: "0.85rem",
          }}
        >
          <div style={{ color: "var(--text-dim)" }}># In terminal run:</div>
          <div style={{ color: "var(--accent-light)" }}>
            steam-storefront-linter dataset download
          </div>
          <div style={{ color: "var(--accent-light)" }}>
            steam-storefront-linter dataset ingest
          </div>
        </div>
      </div>
    );
  }

  return (
    <div>
      {cohort && (
        <div
          style={{
            background: "var(--bg-secondary)",
            border: "1px solid var(--border-subtle)",
            borderRadius: "8px",
            padding: "1.25rem",
            marginBottom: "1.75rem",
          }}
        >
          <div
            style={{
              display: "flex",
              justifyContent: "space-between",
              alignItems: "flex-start",
              flexWrap: "wrap",
              gap: "0.75rem",
            }}
          >
            <div>
              <div
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: "0.5rem",
                  color: "var(--accent-light)",
                  fontSize: "0.82rem",
                  fontWeight: 600,
                  textTransform: "uppercase",
                }}
              >
                <Users size={15} /> Comparison Cohort
              </div>
              <h3
                style={{
                  fontSize: "1.2rem",
                  fontWeight: 700,
                  color: "#fff",
                  marginTop: "0.2rem",
                }}
              >
                {cohort.cohort_name}
              </h3>
              <p
                style={{
                  fontSize: "0.85rem",
                  color: "var(--text-muted)",
                  marginTop: "0.3rem",
                }}
              >
                {cohort.explanation}
              </p>
            </div>

            <div
              style={{
                textAlign: "right",
                fontSize: "0.82rem",
                color: "var(--text-muted)",
              }}
            >
              <div>
                Cohort Pool:{" "}
                <strong style={{ color: "#fff" }}>
                  {cohort.total_games_in_cohort} games
                </strong>
              </div>
              <div
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: "4px",
                  justifyContent: "flex-end",
                  marginTop: "0.2rem",
                }}
              >
                <Calendar size={13} /> Snapshot:{" "}
                <strong>{methodology.snapshot_date}</strong>
              </div>
            </div>
          </div>
        </div>
      )}

      <div
        style={{
          display: "grid",
          gridTemplateColumns: "repeat(auto-fill, minmax(480px, 1fr))",
          gap: "1.25rem",
        }}
      >
        {benchmarks.map((b) => (
          <div key={b.metric_id} className="benchmark-card">
            <div
              style={{
                display: "flex",
                justifyContent: "space-between",
                alignItems: "center",
                marginBottom: "0.75rem",
              }}
            >
              <div>
                <span
                  style={{
                    fontSize: "0.7rem",
                    color: "var(--text-dim)",
                    textTransform: "uppercase",
                    fontWeight: 600,
                  }}
                >
                  {b.category}
                </span>
                <h4
                  style={{
                    fontSize: "1.05rem",
                    fontWeight: 700,
                    color: "#fff",
                  }}
                >
                  {b.metric_name}
                </h4>
              </div>
              <span
                className="badge"
                style={{
                  backgroundColor:
                    b.status === "calculated"
                      ? "var(--color-pass-bg)"
                      : "rgba(148, 163, 184, 0.15)",
                  color:
                    b.status === "calculated"
                      ? "var(--color-pass)"
                      : "var(--text-muted)",
                  border: `1px solid ${b.status === "calculated" ? "var(--color-pass-border)" : "var(--border-subtle)"}`,
                }}
              >
                {b.status === "calculated"
                  ? "Calculated"
                  : b.status.replace(/_/g, " ")}
              </span>
            </div>

            {b.status === "calculated" && b.distribution && (
              <>
                <div className="stat-strip">
                  <div className="stat-cell">
                    <span className="stat-cell-title">Your Game</span>
                    <div className="stat-cell-val highlight">
                      {b.listing_value_display}
                    </div>
                  </div>
                  <div className="stat-cell">
                    <span className="stat-cell-title">Median</span>
                    <div className="stat-cell-val">
                      {b.distribution.median?.toFixed(0)}
                    </div>
                  </div>
                  <div className="stat-cell">
                    <span className="stat-cell-title">IQR (25th - 75th)</span>
                    <div className="stat-cell-val">
                      {b.distribution.p25?.toFixed(0)} -{" "}
                      {b.distribution.p75?.toFixed(0)}
                    </div>
                  </div>
                  <div className="stat-cell">
                    <span className="stat-cell-title">Percentile</span>
                    <div
                      className="stat-cell-val"
                      style={{ color: "var(--color-warning)" }}
                    >
                      {b.distribution.percentile_rank?.toFixed(0)}th
                    </div>
                  </div>
                </div>

                <DistributionBar
                  percentileRank={b.distribution.percentile_rank ?? 50}
                  p25={b.distribution.p25}
                  median={b.distribution.median}
                  p75={b.distribution.p75}
                />

                <div
                  style={{
                    display: "flex",
                    justifyContent: "space-between",
                    fontSize: "0.78rem",
                    color: "var(--text-muted)",
                    marginTop: "0.6rem",
                  }}
                >
                  <span>
                    Sample: <strong>{b.distribution.usable_records}</strong> of{" "}
                    {b.distribution.sample_size} usable (
                    {b.distribution.coverage_pct.toFixed(0)}% coverage)
                  </span>
                  <span>
                    Share above:{" "}
                    <strong>{b.distribution.share_above?.toFixed(0)}%</strong>
                  </span>
                </div>
              </>
            )}

            {b.status === "calculated" && b.categorical && (
              <>
                <div
                  className="stat-strip"
                  style={{ gridTemplateColumns: "repeat(3, 1fr)" }}
                >
                  <div className="stat-cell">
                    <span className="stat-cell-title">Your Game</span>
                    <div className="stat-cell-val highlight">
                      {b.listing_value_display}
                    </div>
                  </div>
                  <div className="stat-cell">
                    <span className="stat-cell-title">Cohort Adoption</span>
                    <div
                      className="stat-cell-val"
                      style={{ color: "var(--color-pass)" }}
                    >
                      {b.categorical.true_percentage.toFixed(0)}%
                    </div>
                  </div>
                  <div className="stat-cell">
                    <span className="stat-cell-title">Games With Feature</span>
                    <div className="stat-cell-val">
                      {b.categorical.true_count} /{" "}
                      {b.categorical.usable_records}
                    </div>
                  </div>
                </div>

                <div
                  style={{
                    fontSize: "0.78rem",
                    color: "var(--text-muted)",
                    marginTop: "0.5rem",
                  }}
                >
                  <span>
                    Sample: <strong>{b.categorical.usable_records}</strong>{" "}
                    usable games in cohort
                  </span>
                </div>
              </>
            )}

            {b.status !== "calculated" && (
              <div
                style={{
                  padding: "1rem",
                  background: "#0e141e",
                  borderRadius: "4px",
                  fontSize: "0.85rem",
                  color: "var(--text-muted)",
                  display: "flex",
                  alignItems: "center",
                  gap: "0.5rem",
                }}
              >
                <Info size={16} color="var(--accent-light)" />
                <span>
                  {b.explanation ||
                    "Benchmark comparison unavailable for this metric."}
                </span>
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
};
