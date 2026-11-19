import { AlertTriangle, Scale } from "lucide-react";
import React from "react";

export const MethodologyView: React.FC = () => {
  return (
    <div style={{ maxWidth: "850px", margin: "0 auto" }}>
      <div className="audit-card">
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: "0.75rem",
            marginBottom: "1.25rem",
          }}
        >
          <Scale size={24} color="var(--accent-light)" />
          <h2 style={{ fontSize: "1.5rem", fontWeight: 800, color: "#fff" }}>
            Benchmark Methodology & Dataset Provenance
          </h2>
        </div>

        <div className="alert alert-info" style={{ marginBottom: "1.75rem" }}>
          <AlertTriangle
            size={18}
            style={{ flexShrink: 0, color: "var(--color-warning)" }}
          />
          <div>
            <strong>Core Product Principle:</strong> Steam Storefront Linter
            audits metadata completeness and descriptive peer distributions. It
            never attempts to predict sales, revenue, player counts, Steam
            rankings, or review sentiment. A game with more screenshots or Linux
            support is not guaranteed to be commercially successful.
          </div>
        </div>

        <section style={{ marginBottom: "2rem" }}>
          <h3
            style={{
              fontSize: "1.15rem",
              fontWeight: 700,
              color: "#fff",
              marginBottom: "0.6rem",
            }}
          >
            1. Dataset Source & Attribution
          </h3>
          <p
            style={{
              color: "var(--text-main)",
              fontSize: "0.92rem",
              marginBottom: "0.75rem",
            }}
          >
            The primary benchmark dataset is the open-source{" "}
            <strong>FronkonGames/steam-games-dataset</strong>, distributed under
            the <strong>MIT License</strong> via Hugging Face. The dataset
            compiles public metadata from the Steam Store API and SteamSpy.
          </p>
          <ul
            style={{
              paddingLeft: "1.25rem",
              color: "var(--text-muted)",
              fontSize: "0.88rem",
              display: "flex",
              flexDirection: "column",
              gap: "0.35rem",
            }}
          >
            <li>
              <strong>Dataset Repository:</strong>{" "}
              <a
                href="https://huggingface.co/datasets/FronkonGames/steam-games-dataset"
                target="_blank"
                rel="noopener noreferrer"
              >
                huggingface.co/datasets/FronkonGames/steam-games-dataset
              </a>
            </li>
            <li>
              <strong>Format:</strong> Apache Parquet (tabular)
            </li>
            <li>
              <strong>License:</strong> MIT License
            </li>
            <li>
              <strong>Attribution:</strong> Maintained by Fronkon Games (MIT),
              generated via Steam public endpoints.
            </li>
          </ul>
        </section>

        <section style={{ marginBottom: "2rem" }}>
          <h3
            style={{
              fontSize: "1.15rem",
              fontWeight: 700,
              color: "#fff",
              marginBottom: "0.6rem",
            }}
          >
            2. Ingestion & Data Quality Controls
          </h3>
          <p
            style={{
              color: "var(--text-main)",
              fontSize: "0.92rem",
              marginBottom: "0.75rem",
            }}
          >
            Steam scraped datasets naturally contain incomplete records, closed
            beta entries, and unreleased playtest artifacts. The ingestion
            pipeline enforces deterministic data quality standards:
          </p>
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(auto-fit, minmax(280px, 1fr))",
              gap: "1rem",
            }}
          >
            <div
              style={{
                background: "var(--bg-primary)",
                padding: "1rem",
                borderRadius: "6px",
                border: "1px solid var(--border-subtle)",
              }}
            >
              <strong
                style={{
                  color: "var(--color-warning)",
                  fontSize: "0.9rem",
                  display: "block",
                  marginBottom: "0.3rem",
                }}
              >
                Playtest Exclusion
              </strong>
              <span style={{ fontSize: "0.82rem", color: "var(--text-muted)" }}>
                Titles containing "Playtest" (such as AppID 2539430) are flagged
                and excluded from the peer benchmark pool to prevent skewed
                baselines.
              </span>
            </div>
            <div
              style={{
                background: "var(--bg-primary)",
                padding: "1rem",
                borderRadius: "6px",
                border: "1px solid var(--border-subtle)",
              }}
            >
              <strong
                style={{
                  color: "var(--color-warning)",
                  fontSize: "0.9rem",
                  display: "block",
                  marginBottom: "0.3rem",
                }}
              >
                Sparse Metadata Handling
              </strong>
              <span style={{ fontSize: "0.82rem", color: "var(--text-muted)" }}>
                Rows lacking both short and detailed descriptions are marked as
                sparse and excluded from comparison calculations.
              </span>
            </div>
            <div
              style={{
                background: "var(--bg-primary)",
                padding: "1rem",
                borderRadius: "6px",
                border: "1px solid var(--border-subtle)",
              }}
            >
              <strong
                style={{
                  color: "var(--accent-light)",
                  fontSize: "0.9rem",
                  display: "block",
                  marginBottom: "0.3rem",
                }}
              >
                Deterministic Deduplication
              </strong>
              <span style={{ fontSize: "0.82rem", color: "var(--text-muted)" }}>
                Records are deduplicated by numeric Steam AppID. Duplicate rows
                are logged and discarded.
              </span>
            </div>
            <div
              style={{
                background: "var(--bg-primary)",
                padding: "1rem",
                borderRadius: "6px",
                border: "1px solid var(--border-subtle)",
              }}
            >
              <strong
                style={{
                  color: "var(--color-pass)",
                  fontSize: "0.9rem",
                  display: "block",
                  marginBottom: "0.3rem",
                }}
              >
                Missing Data vs Zero
              </strong>
              <span style={{ fontSize: "0.82rem", color: "var(--text-muted)" }}>
                Missing values remain null and are excluded from denominators.
                An unrecorded audio field is never converted to 0.
              </span>
            </div>
          </div>
        </section>

        <section style={{ marginBottom: "2rem" }}>
          <h3
            style={{
              fontSize: "1.15rem",
              fontWeight: 700,
              color: "#fff",
              marginBottom: "0.6rem",
            }}
          >
            3. Cohort Selection Strategy
          </h3>
          <p
            style={{
              color: "var(--text-main)",
              fontSize: "0.92rem",
              marginBottom: "0.75rem",
            }}
          >
            Games are not compared blindly against the entire Steam catalog. The
            system evaluates multi-stage cohort specificity:
          </p>
          <ol
            style={{
              paddingLeft: "1.25rem",
              color: "var(--text-muted)",
              fontSize: "0.88rem",
              display: "flex",
              flexDirection: "column",
              gap: "0.4rem",
            }}
          >
            <li>
              <strong>Specific Cohort (Preferred):</strong> Games sharing one or
              more relevant genres, released in the same calendar year, and
              classified as Indie.
            </li>
            <li>
              <strong>Stage 2 Fallback:</strong> Same genre(s) and release year
              (relaxing the Indie restriction if sample size &lt; 30).
            </li>
            <li>
              <strong>Stage 3 Fallback:</strong> Same genre(s) across all
              release years.
            </li>
            <li>
              <strong>Stage 4 Fallback:</strong> All Indie games released in the
              same calendar year.
            </li>
            <li>
              <strong>Stage 5 Fallback:</strong> Broad eligible dataset pool.
            </li>
          </ol>
          <p
            style={{
              fontSize: "0.82rem",
              color: "var(--text-dim)",
              marginTop: "0.5rem",
            }}
          >
            Whenever a fallback is triggered, the report transparently discloses
            that the comparison cohort was broadened.
          </p>
        </section>

        <section>
          <h3
            style={{
              fontSize: "1.15rem",
              fontWeight: 700,
              color: "#fff",
              marginBottom: "0.6rem",
            }}
          >
            4. Statistical Conventions
          </h3>
          <ul
            style={{
              paddingLeft: "1.25rem",
              color: "var(--text-muted)",
              fontSize: "0.88rem",
              display: "flex",
              flexDirection: "column",
              gap: "0.4rem",
            }}
          >
            <li>
              <strong>Median (50th Percentile):</strong> Middle value of the
              sorted distribution.
            </li>
            <li>
              <strong>Interquartile Range (IQR):</strong> Span from the 25th
              percentile (P25) to the 75th percentile (P75).
            </li>
            <li>
              <strong>Percentile Rank:</strong> Calculated as{" "}
              <code style={{ color: "var(--accent-light)" }}>
                (count_below + 0.5 * count_equal) / total_usable * 100
              </code>
              .
            </li>
            <li>
              <strong>Share Above:</strong> Exact percentage of comparable games
              having a value strictly greater than your game.
            </li>
          </ul>
        </section>
      </div>
    </div>
  );
};
