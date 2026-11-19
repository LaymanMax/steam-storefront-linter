import React from "react";

interface DistributionBarProps {
  percentileRank: number; // 0 to 100
  p25?: number;
  median?: number;
  p75?: number;
}

export const DistributionBar: React.FC<DistributionBarProps> = ({
  percentileRank,
}) => {
  const clamped = Math.max(0, Math.min(100, percentileRank));

  return (
    <div
      className="percentile-bar-container"
      title={`Ranked at ${clamped.toFixed(0)}th percentile`}
    >
      <div className="percentile-track">
        <div className="percentile-fill" style={{ width: `${clamped}%` }} />
        <div className="percentile-pin" style={{ left: `${clamped}%` }} />
      </div>
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          fontSize: "0.7rem",
          color: "var(--text-dim)",
          marginTop: "4px",
        }}
      >
        <span>0th pct (Min)</span>
        <span>25th</span>
        <span>50th (Median)</span>
        <span>75th</span>
        <span>100th (Max)</span>
      </div>
    </div>
  );
};
