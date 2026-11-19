import { BarChart3, CheckSquare } from "lucide-react";
import React, { useState } from "react";
import { auditImportJson, auditSteamApp, getSampleReport } from "./api/client";
import { AuditForm } from "./components/AuditForm";
import { AuditProgress } from "./components/AuditProgress";
import { BenchmarksView } from "./components/BenchmarksView";
import { FindingsList } from "./components/FindingsList";
import { MethodologyView } from "./components/MethodologyView";
import { Navbar } from "./components/Navbar";
import { OverviewKpis } from "./components/OverviewKpis";
import { ResultsHeader } from "./components/ResultsHeader";
import { AuditReport } from "./types";

export const App: React.FC = () => {
  const [currentNavTab, setCurrentNavTab] = useState<"audit" | "methodology">(
    "audit",
  );
  const [activeReportTab, setActiveReportTab] = useState<
    "findings" | "benchmarks"
  >("findings");
  const [report, setReport] = useState<AuditReport | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const handleAuditSteam = async (
    target: string,
    country: string,
    language: string,
    profile: "balanced" | "strict",
  ) => {
    setIsLoading(true);
    setErrorMessage(null);
    try {
      const data = await auditSteamApp(target, { country, language, profile });
      setReport(data);
      setActiveReportTab("findings");
    } catch (err) {
      setErrorMessage((err as Error).message);
    } finally {
      setIsLoading(false);
    }
  };

  const handleAuditUpload = async (
    jsonData: unknown,
    profile: "balanced" | "strict",
  ) => {
    setIsLoading(true);
    setErrorMessage(null);
    try {
      const data = await auditImportJson(jsonData, { profile });
      setReport(data);
      setActiveReportTab("findings");
    } catch (err) {
      setErrorMessage((err as Error).message);
    } finally {
      setIsLoading(false);
    }
  };

  const handleLoadSample = async () => {
    setIsLoading(true);
    setErrorMessage(null);
    setCurrentNavTab("audit");
    try {
      const data = await getSampleReport();
      setReport(data);
      setActiveReportTab("findings");
    } catch (err) {
      setErrorMessage((err as Error).message);
    } finally {
      setIsLoading(false);
    }
  };

  const handleReset = () => {
    setReport(null);
    setErrorMessage(null);
  };

  return (
    <div className="app-container">
      <Navbar
        currentTab={currentNavTab}
        onSelectTab={setCurrentNavTab}
        onLoadSample={handleLoadSample}
      />

      <main className="main-content">
        {currentNavTab === "methodology" ? (
          <MethodologyView />
        ) : (
          <>
            {isLoading && <AuditProgress />}

            {!isLoading && !report && (
              <AuditForm
                onAuditSteam={handleAuditSteam}
                onAuditUpload={handleAuditUpload}
                isLoading={isLoading}
                errorMessage={errorMessage}
              />
            )}

            {!isLoading && report && (
              <div>
                <ResultsHeader report={report} onReset={handleReset} />
                <OverviewKpis
                  summary={report.summary}
                  dataQuality={report.data_quality}
                />

                <div
                  style={{
                    display: "flex",
                    borderBottom: "1px solid var(--border-subtle)",
                    marginBottom: "1.5rem",
                    gap: "1rem",
                  }}
                >
                  <button
                    onClick={() => setActiveReportTab("findings")}
                    style={{
                      background: "none",
                      border: "none",
                      borderBottom:
                        activeReportTab === "findings"
                          ? "2px solid var(--accent-light)"
                          : "2px solid transparent",
                      padding: "0.65rem 1rem",
                      color:
                        activeReportTab === "findings"
                          ? "#fff"
                          : "var(--text-muted)",
                      fontWeight: 600,
                      cursor: "pointer",
                      display: "flex",
                      alignItems: "center",
                      gap: "0.5rem",
                    }}
                  >
                    <CheckSquare size={16} />
                    <span>Audit Findings ({report.findings.length})</span>
                  </button>

                  <button
                    onClick={() => setActiveReportTab("benchmarks")}
                    style={{
                      background: "none",
                      border: "none",
                      borderBottom:
                        activeReportTab === "benchmarks"
                          ? "2px solid var(--accent-light)"
                          : "2px solid transparent",
                      padding: "0.65rem 1rem",
                      color:
                        activeReportTab === "benchmarks"
                          ? "#fff"
                          : "var(--text-muted)",
                      fontWeight: 600,
                      cursor: "pointer",
                      display: "flex",
                      alignItems: "center",
                      gap: "0.5rem",
                    }}
                  >
                    <BarChart3 size={16} />
                    <span>Peer Benchmarks ({report.benchmarks.length})</span>
                  </button>
                </div>

                {activeReportTab === "findings" ? (
                  <FindingsList findings={report.findings} />
                ) : (
                  <BenchmarksView
                    benchmarks={report.benchmarks}
                    cohort={report.cohort}
                    methodology={report.methodology}
                  />
                )}
              </div>
            )}
          </>
        )}
      </main>

      <footer className="app-footer">
        <div>
          <strong>Steam Storefront Linter</strong> — Open-source listing
          completeness auditor & peer benchmarking tool.
        </div>
        <div style={{ marginTop: "0.35rem", color: "var(--text-dim)" }}>
          Powered by Rust & React. Benchmarks derived from
          FronkonGames/steam-games-dataset (MIT License). Not affiliated with
          Valve or Steam.
        </div>
      </footer>
    </div>
  );
};
