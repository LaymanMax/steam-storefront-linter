import { CheckCircle2, Loader2 } from "lucide-react";
import React, { useEffect, useState } from "react";

const STAGES = [
  "Fetching Steam listing from public endpoint...",
  "Normalizing store metadata and stripping HTML...",
  "Evaluating rule completeness across 8 categories...",
  "Selecting benchmark cohort from peer dataset...",
  "Computing quartile distributions and percentile rank...",
  "Finalizing report...",
];

export const AuditProgress: React.FC = () => {
  const [currentStage, setCurrentStage] = useState(0);

  useEffect(() => {
    const interval = setInterval(() => {
      setCurrentStage((prev) => (prev < STAGES.length - 1 ? prev + 1 : prev));
    }, 450);
    return () => clearInterval(interval);
  }, []);

  return (
    <div
      className="audit-card"
      style={{ textAlign: "center", padding: "3rem 2rem" }}
    >
      <Loader2
        size={36}
        className="animate-spin"
        style={{
          color: "var(--accent-light)",
          margin: "0 auto 1.5rem",
          animation: "spin 1s linear infinite",
        }}
      />
      <h3
        style={{
          fontSize: "1.2rem",
          fontWeight: 600,
          color: "#fff",
          marginBottom: "1.5rem",
        }}
      >
        Auditing Steam Storefront Listing
      </h3>

      <div
        style={{
          maxWidth: "420px",
          margin: "0 auto",
          textAlign: "left",
          display: "flex",
          flexDirection: "column",
          gap: "0.75rem",
        }}
      >
        {STAGES.map((stage, idx) => {
          const isDone = idx < currentStage;
          const isCurrent = idx === currentStage;

          return (
            <div
              key={stage}
              style={{
                display: "flex",
                alignItems: "center",
                gap: "0.75rem",
                fontSize: "0.88rem",
                color: isCurrent
                  ? "#fff"
                  : isDone
                    ? "var(--color-pass)"
                    : "var(--text-dim)",
                fontWeight: isCurrent ? 600 : 400,
              }}
            >
              {isDone ? (
                <CheckCircle2 size={16} color="var(--color-pass)" />
              ) : isCurrent ? (
                <div
                  style={{
                    width: "14px",
                    height: "14px",
                    borderRadius: "50%",
                    border: "2px solid var(--accent-light)",
                    borderTopColor: "transparent",
                    animation: "spin 0.8s linear infinite",
                  }}
                />
              ) : (
                <div
                  style={{
                    width: "12px",
                    height: "12px",
                    borderRadius: "50%",
                    border: "1px solid var(--text-dim)",
                  }}
                />
              )}
              <span>{stage}</span>
            </div>
          );
        })}
      </div>

      <style>{`
        @keyframes spin {
          from { transform: rotate(0deg); }
          to { transform: rotate(360deg); }
        }
      `}</style>
    </div>
  );
};
