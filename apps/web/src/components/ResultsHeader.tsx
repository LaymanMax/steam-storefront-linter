import { Download, ExternalLink, FileCode, RefreshCw } from "lucide-react";
import React from "react";
import { getExportHtmlUrl, getExportJsonUrl } from "../api/client";
import { AuditReport } from "../types";

interface ResultsHeaderProps {
  report: AuditReport;
  onReset: () => void;
}

export const ResultsHeader: React.FC<ResultsHeaderProps> = ({
  report,
  onReset,
}) => {
  const { listing, audit_id, generated_at, audit_profile } = report;
  const storeUrl =
    listing.store_url ||
    `https://store.steampowered.com/app/${listing.app_id}/`;
  const formattedDate = new Date(generated_at).toLocaleString();

  return (
    <div
      className="audit-card"
      style={{ marginBottom: "1.5rem", padding: "1.75rem" }}
    >
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "flex-start",
          flexWrap: "wrap",
          gap: "1rem",
        }}
      >
        <div>
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: "0.75rem",
              flexWrap: "wrap",
            }}
          >
            <h1 style={{ fontSize: "1.75rem", fontWeight: 800, color: "#fff" }}>
              {listing.name}
            </h1>
            <span className="badge badge-info">AppID: {listing.app_id}</span>
            <span className="badge badge-info">Profile: {audit_profile}</span>
          </div>

          <div
            style={{
              display: "flex",
              gap: "1.25rem",
              fontSize: "0.82rem",
              color: "var(--text-muted)",
              marginTop: "0.5rem",
              flexWrap: "wrap",
            }}
          >
            <span>
              Audited: <strong>{formattedDate}</strong>
            </span>
            <span>
              Source: <strong>{listing.source}</strong>
            </span>
            <a
              href={storeUrl}
              target="_blank"
              rel="noopener noreferrer"
              style={{
                display: "inline-flex",
                alignItems: "center",
                gap: "4px",
              }}
            >
              View on Steam Store <ExternalLink size={13} />
            </a>
          </div>
        </div>

        <div style={{ display: "flex", gap: "0.6rem", flexWrap: "wrap" }}>
          <button
            className="btn btn-secondary"
            onClick={onReset}
            style={{ fontSize: "0.85rem" }}
          >
            <RefreshCw size={14} /> New Audit
          </button>
          <a
            href={getExportJsonUrl(audit_id)}
            download={`steam-audit-${listing.app_id}.json`}
            className="btn btn-secondary"
            style={{ fontSize: "0.85rem" }}
          >
            <FileCode size={14} /> JSON
          </a>
          <a
            href={getExportHtmlUrl(audit_id)}
            download={`steam-audit-${listing.app_id}.html`}
            className="btn btn-primary"
            style={{ fontSize: "0.85rem" }}
          >
            <Download size={14} /> Standalone HTML
          </a>
        </div>
      </div>
    </div>
  );
};
