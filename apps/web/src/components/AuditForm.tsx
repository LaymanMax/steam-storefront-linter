import { AlertCircle, FileCode, Search, Upload } from "lucide-react";
import React, { useState } from "react";
import { getDownloadExampleUrl } from "../api/client";

interface AuditFormProps {
  onAuditSteam: (
    target: string,
    country: string,
    language: string,
    profile: "balanced" | "strict",
  ) => void;
  onAuditUpload: (jsonContent: unknown, profile: "balanced" | "strict") => void;
  isLoading: boolean;
  errorMessage?: string | null;
}

export const AuditForm: React.FC<AuditFormProps> = ({
  onAuditSteam,
  onAuditUpload,
  isLoading,
  errorMessage,
}) => {
  const [target, setTarget] = useState("");
  const [country, setCountry] = useState("us");
  const [language, setLanguage] = useState("english");
  const [profile, setProfile] = useState<"balanced" | "strict">("balanced");
  const [uploadError, setUploadError] = useState<string | null>(null);

  const handleSubmitSteam = (e: React.FormEvent) => {
    e.preventDefault();
    if (!target.trim()) return;
    onAuditSteam(target.trim(), country, language, profile);
  };

  const handleFileUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    setUploadError(null);
    const file = e.target.files?.[0];
    if (!file) return;

    if (file.size > 10 * 1024 * 1024) {
      setUploadError("File size exceeds 10 MB limit.");
      return;
    }

    const reader = new FileReader();
    reader.onload = (event) => {
      try {
        const parsed = JSON.parse(event.target?.result as string);
        onAuditUpload(parsed, profile);
      } catch (err) {
        setUploadError(`Invalid JSON format: ${(err as Error).message}`);
      }
    };
    reader.onerror = () => setUploadError("Failed to read file.");
    reader.readAsText(file);
  };

  return (
    <div className="audit-card">
      <div style={{ marginBottom: "1.5rem" }}>
        <h2
          style={{
            fontSize: "1.4rem",
            fontWeight: 700,
            color: "#fff",
            marginBottom: "0.4rem",
          }}
        >
          Audit Steam Store Listing Completeness
        </h2>
        <p style={{ color: "var(--text-muted)", fontSize: "0.95rem" }}>
          Enter a Steam AppID, store URL, or upload your listing metadata JSON
          to audit completeness and benchmark against peer indie games.
        </p>
      </div>

      {(errorMessage || uploadError) && (
        <div className="alert alert-error" role="alert">
          <AlertCircle size={18} style={{ flexShrink: 0 }} />
          <span>{errorMessage || uploadError}</span>
        </div>
      )}

      <form onSubmit={handleSubmitSteam}>
        <div className="form-group">
          <label className="form-label" htmlFor="steam-target">
            Steam AppID or Store URL
          </label>
          <div style={{ display: "flex", gap: "0.75rem", flexWrap: "wrap" }}>
            <input
              id="steam-target"
              type="text"
              className="form-input"
              style={{ flex: 1, minWidth: "240px" }}
              placeholder="e.g. 2539430 or https://store.steampowered.com/app/2539430/"
              value={target}
              onChange={(e) => setTarget(e.target.value)}
              disabled={isLoading}
            />
            <button
              type="submit"
              className="btn btn-primary"
              disabled={isLoading || !target.trim()}
              style={{ minWidth: "150px" }}
            >
              <Search size={16} />
              {isLoading ? "Auditing..." : "Audit Listing"}
            </button>
          </div>
          <span
            style={{
              fontSize: "0.78rem",
              color: "var(--text-dim)",
              marginTop: "0.35rem",
              display: "block",
            }}
          >
            Supports raw numeric AppID (e.g. 2539430) or any
            store.steampowered.com URL.
          </span>
        </div>

        <div className="form-row" style={{ marginTop: "1.25rem" }}>
          <div className="form-group">
            <label className="form-label" htmlFor="country-select">
              Store Country
            </label>
            <select
              id="country-select"
              className="form-input"
              value={country}
              onChange={(e) => setCountry(e.target.value)}
              disabled={isLoading}
            >
              <option value="us">United States (USD)</option>
              <option value="gb">United Kingdom (GBP)</option>
              <option value="de">Germany / EU (EUR)</option>
              <option value="jp">Japan (JPY)</option>
              <option value="cn">China (RMB)</option>
              <option value="ca">Canada (CAD)</option>
            </select>
          </div>

          <div className="form-group">
            <label className="form-label" htmlFor="lang-select">
              Storefront Language
            </label>
            <select
              id="lang-select"
              className="form-input"
              value={language}
              onChange={(e) => setLanguage(e.target.value)}
              disabled={isLoading}
            >
              <option value="english">English</option>
              <option value="german">German (Deutsch)</option>
              <option value="french">French (Français)</option>
              <option value="spanish">Spanish (Español)</option>
              <option value="japanese">Japanese (日本語)</option>
              <option value="schinese">Simplified Chinese (简体中文)</option>
            </select>
          </div>

          <div className="form-group">
            <label className="form-label" htmlFor="profile-select">
              Audit Profile
            </label>
            <select
              id="profile-select"
              className="form-input"
              value={profile}
              onChange={(e) =>
                setProfile(e.target.value as "balanced" | "strict")
              }
              disabled={isLoading}
            >
              <option value="balanced">Balanced (Recommended)</option>
              <option value="strict">Strict (Higher thresholds)</option>
            </select>
          </div>
        </div>
      </form>

      <div
        style={{
          display: "flex",
          alignItems: "center",
          margin: "1.75rem 0",
          gap: "1rem",
        }}
      >
        <div
          style={{ flex: 1, height: "1px", background: "var(--border-subtle)" }}
        />
        <span
          style={{
            fontSize: "0.8rem",
            color: "var(--text-dim)",
            textTransform: "uppercase",
          }}
        >
          or
        </span>
        <div
          style={{ flex: 1, height: "1px", background: "var(--border-subtle)" }}
        />
      </div>

      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          flexWrap: "wrap",
          gap: "1rem",
        }}
      >
        <div>
          <label
            htmlFor="json-upload-input"
            className="btn btn-secondary"
            style={{ cursor: "pointer" }}
          >
            <Upload size={16} />
            <span>Upload Metadata JSON</span>
          </label>
          <input
            id="json-upload-input"
            type="file"
            accept=".json,application/json"
            style={{ display: "none" }}
            onChange={handleFileUpload}
            disabled={isLoading}
          />
          <span
            style={{
              fontSize: "0.82rem",
              color: "var(--text-muted)",
              marginLeft: "0.75rem",
            }}
          >
            Pre-release drafts, Steamworks JSON, or benchmark formats.
          </span>
        </div>

        <a
          href={getDownloadExampleUrl()}
          download="example_listing.json"
          className="btn btn-secondary"
          style={{ fontSize: "0.82rem", padding: "0.5rem 0.9rem" }}
        >
          <FileCode size={14} /> Download Example JSON
        </a>
      </div>
    </div>
  );
};
