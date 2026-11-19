import React from "react";
import { AuditDataQuality, AuditSummary } from "../types";

interface OverviewKpisProps {
  summary: AuditSummary;
  dataQuality: AuditDataQuality;
}

export const OverviewKpis: React.FC<OverviewKpisProps> = ({
  summary,
  dataQuality,
}) => {
  return (
    <div style={{ marginBottom: "1.75rem" }}>
      <div className="kpi-grid">
        <div className="kpi-card">
          <div className="kpi-num" style={{ color: "var(--color-critical)" }}>
            {summary.critical}
          </div>
          <div className="kpi-label">Critical Issues</div>
        </div>

        <div className="kpi-card">
          <div className="kpi-num" style={{ color: "var(--color-warning)" }}>
            {summary.warnings}
          </div>
          <div className="kpi-label">Warnings</div>
        </div>

        <div className="kpi-card">
          <div className="kpi-num" style={{ color: "var(--color-info)" }}>
            {summary.info}
          </div>
          <div className="kpi-label">Observations</div>
        </div>

        <div className="kpi-card">
          <div className="kpi-num" style={{ color: "var(--color-pass)" }}>
            {summary.passed}
          </div>
          <div className="kpi-label">Checks Passed</div>
        </div>

        <div className="kpi-card">
          <div className="kpi-num" style={{ color: "var(--accent-light)" }}>
            {dataQuality.data_completeness_pct.toFixed(0)}%
          </div>
          <div className="kpi-label">Data Coverage</div>
        </div>
      </div>

      <div
        style={{
          display: "flex",
          gap: "0.5rem",
          flexWrap: "wrap",
          marginTop: "0.75rem",
        }}
      >
        {Object.entries(summary.section_coverage).map(([cat, pct]) => (
          <div
            key={cat}
            style={{
              background: "var(--bg-secondary)",
              border: "1px solid var(--border-subtle)",
              borderRadius: "4px",
              padding: "0.3rem 0.6rem",
              fontSize: "0.78rem",
              color: "var(--text-muted)",
              display: "flex",
              alignItems: "center",
              gap: "6px",
            }}
          >
            <span>{cat}:</span>
            <strong
              style={{ color: pct === 100 ? "var(--color-pass)" : "#fff" }}
            >
              {pct.toFixed(0)}%
            </strong>
          </div>
        ))}
      </div>

      <div
        style={{
          fontSize: "0.8rem",
          color: "var(--text-dim)",
          marginTop: "0.75rem",
          fontStyle: "italic",
        }}
      >
        Completeness metrics reflect declared metadata fields and store
        formatting. Completeness does not correlate with or predict commercial
        sales.
      </div>
    </div>
  );
};
