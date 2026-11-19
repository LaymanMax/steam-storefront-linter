import { AlertOctagon, AlertTriangle, CheckCircle2, Info } from "lucide-react";
import React, { useState } from "react";
import { Finding, FindingCategory, Severity } from "../types";

interface FindingsListProps {
  findings: Finding[];
}

export const FindingsList: React.FC<FindingsListProps> = ({ findings }) => {
  const [selectedSeverity, setSelectedSeverity] = useState<Severity | "all">(
    "all",
  );
  const [selectedCategory, setSelectedCategory] = useState<
    FindingCategory | "all"
  >("all");

  const filtered = findings.filter((f) => {
    if (selectedSeverity !== "all" && f.severity !== selectedSeverity)
      return false;
    if (selectedCategory !== "all" && f.category !== selectedCategory)
      return false;
    return true;
  });

  const getSeverityIcon = (sev: Severity) => {
    switch (sev) {
      case "critical":
        return <AlertOctagon size={16} color="var(--color-critical)" />;
      case "warning":
        return <AlertTriangle size={16} color="var(--color-warning)" />;
      case "info":
        return <Info size={16} color="var(--color-info)" />;
      case "pass":
        return <CheckCircle2 size={16} color="var(--color-pass)" />;
    }
  };

  const getBorderClass = (sev: Severity) => {
    switch (sev) {
      case "critical":
        return "border-critical";
      case "warning":
        return "border-warning";
      case "info":
        return "border-info";
      case "pass":
        return "border-pass";
    }
  };

  const getBadgeClass = (sev: Severity) => {
    switch (sev) {
      case "critical":
        return "badge-critical";
      case "warning":
        return "badge-warning";
      case "info":
        return "badge-info";
      case "pass":
        return "badge-pass";
    }
  };

  return (
    <div>
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          marginBottom: "1.25rem",
          flexWrap: "wrap",
          gap: "0.75rem",
        }}
      >
        <div style={{ display: "flex", gap: "0.4rem", flexWrap: "wrap" }}>
          {(["all", "critical", "warning", "info", "pass"] as const).map(
            (sev) => {
              const count =
                sev === "all"
                  ? findings.length
                  : findings.filter((f) => f.severity === sev).length;
              const isSelected = selectedSeverity === sev;
              return (
                <button
                  key={sev}
                  onClick={() => setSelectedSeverity(sev)}
                  className="btn btn-secondary"
                  style={{
                    padding: "0.35rem 0.75rem",
                    fontSize: "0.82rem",
                    textTransform: "capitalize",
                    backgroundColor: isSelected
                      ? "var(--bg-tertiary)"
                      : "transparent",
                    borderColor: isSelected
                      ? "var(--accent-light)"
                      : "var(--border-subtle)",
                    color: isSelected ? "#fff" : "var(--text-muted)",
                  }}
                >
                  {sev} ({count})
                </button>
              );
            },
          )}
        </div>

        <div>
          <select
            className="form-input"
            style={{
              padding: "0.35rem 0.75rem",
              fontSize: "0.82rem",
              width: "auto",
            }}
            value={selectedCategory}
            onChange={(e) =>
              setSelectedCategory(e.target.value as FindingCategory | "all")
            }
          >
            <option value="all">All Categories</option>
            <option value="description">Description</option>
            <option value="screenshots">Screenshots</option>
            <option value="trailers">Trailers & Video</option>
            <option value="localization">Localization</option>
            <option value="support">Support & Links</option>
            <option value="features">Features & Categories</option>
            <option value="platforms">Platform Support</option>
            <option value="general">General Metadata</option>
          </select>
        </div>
      </div>

      {filtered.length === 0 ? (
        <div
          className="audit-card"
          style={{
            textAlign: "center",
            color: "var(--text-muted)",
            padding: "2rem",
          }}
        >
          No findings match the selected filters.
        </div>
      ) : (
        filtered.map((f) => (
          <div
            key={f.id}
            className={`finding-item ${getBorderClass(f.severity)}`}
          >
            <div
              style={{
                display: "flex",
                justifyContent: "space-between",
                alignItems: "center",
                marginBottom: "0.4rem",
                flexWrap: "wrap",
                gap: "0.5rem",
              }}
            >
              <div
                style={{ display: "flex", alignItems: "center", gap: "0.6rem" }}
              >
                {getSeverityIcon(f.severity)}
                <span className={`badge ${getBadgeClass(f.severity)}`}>
                  {f.severity}
                </span>
                <span
                  style={{
                    fontSize: "0.75rem",
                    color: "var(--text-dim)",
                    textTransform: "uppercase",
                    fontWeight: 600,
                  }}
                >
                  {f.category}
                </span>
                <strong style={{ fontSize: "0.98rem", color: "#fff" }}>
                  {f.title}
                </strong>
              </div>
              <span
                style={{
                  fontSize: "0.72rem",
                  color: "var(--text-dim)",
                  fontFamily: "var(--font-mono)",
                }}
              >
                {f.id} (v{f.rule_version})
              </span>
            </div>

            <p
              style={{
                color: "var(--text-main)",
                fontSize: "0.9rem",
                marginBottom: "0.5rem",
              }}
            >
              {f.message}
            </p>

            {f.evidence.actual !== undefined && (
              <div
                style={{
                  fontSize: "0.82rem",
                  color: "var(--text-muted)",
                  marginBottom: "0.5rem",
                }}
              >
                <span>Current value: </span>
                <code
                  style={{
                    background: "#090d14",
                    padding: "2px 6px",
                    borderRadius: "3px",
                    color: "#e2e8f0",
                  }}
                >
                  {typeof f.evidence.actual === "object"
                    ? JSON.stringify(f.evidence.actual)
                    : String(f.evidence.actual)}
                </code>
              </div>
            )}

            {f.severity !== "pass" && (
              <div className="recommendation-box">
                <strong>Actionable recommendation: </strong>
                <span>{f.recommendation}</span>
              </div>
            )}
          </div>
        ))
      )}
    </div>
  );
};
