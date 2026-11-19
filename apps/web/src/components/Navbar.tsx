import { BookOpen, Code2, ShieldCheck, Sparkles } from "lucide-react";
import React from "react";

interface NavbarProps {
  currentTab: "audit" | "methodology";
  onSelectTab: (tab: "audit" | "methodology") => void;
  onLoadSample: () => void;
}

export const Navbar: React.FC<NavbarProps> = ({
  currentTab,
  onSelectTab,
  onLoadSample,
}) => {
  return (
    <nav className="navbar">
      <div className="navbar-inner">
        <div
          className="brand"
          onClick={() => onSelectTab("audit")}
          style={{ cursor: "pointer" }}
        >
          <ShieldCheck className="brand-icon" />
          <span>Steam Storefront Linter</span>
        </div>

        <div className="nav-links">
          <button
            className={`nav-link-btn ${currentTab === "audit" ? "active" : ""}`}
            onClick={() => onSelectTab("audit")}
          >
            Auditor
          </button>
          <button
            className={`nav-link-btn ${currentTab === "methodology" ? "active" : ""}`}
            onClick={() => onSelectTab("methodology")}
          >
            <span
              style={{
                display: "inline-flex",
                alignItems: "center",
                gap: "4px",
              }}
            >
              <BookOpen size={14} /> Methodology & Dataset
            </span>
          </button>
          <button
            className="nav-link-btn"
            onClick={onLoadSample}
            title="Load a demonstration report for Black Dragon Mage"
          >
            <span
              style={{
                display: "inline-flex",
                alignItems: "center",
                gap: "4px",
                color: "var(--accent-light)",
              }}
            >
              <Sparkles size={14} /> Sample Report
            </span>
          </button>
          <a
            href="https://github.com/LaymanMax/steam-storefront-linter"
            target="_blank"
            rel="noopener noreferrer"
            className="nav-link-btn"
            style={{ display: "inline-flex", alignItems: "center", gap: "4px" }}
          >
            <Code2 size={15} /> GitHub
          </a>
        </div>
      </div>
    </nav>
  );
};
