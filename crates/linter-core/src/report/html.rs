use crate::models::benchmark::BenchmarkStatus;
use crate::models::finding::Severity;
use crate::models::report::AuditReport;

pub fn escape_html(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#39;"),
            _ => output.push(c),
        }
    }
    output
}

pub fn generate_html_report(report: &AuditReport) -> String {
    let title = escape_html(&report.listing.name);
    let app_id = report.listing.app_id;
    let store_url = format!("https://store.steampowered.com/app/{}/", app_id);
    let generated_at = report
        .generated_at
        .format("%Y-%m-%d %H:%M:%S UTC")
        .to_string();
    let profile = escape_html(report.audit_profile.as_str());

    let mut findings_html = String::new();
    for f in &report.findings {
        let (badge_class, badge_label) = match f.severity {
            Severity::Critical => ("badge-critical", "CRITICAL"),
            Severity::Warning => ("badge-warning", "WARNING"),
            Severity::Info => ("badge-info", "INFO"),
            Severity::Pass => ("badge-pass", "PASS"),
        };

        let evidence_html = if let Some(actual) = &f.evidence.actual {
            format!(
                r#"<div class="evidence"><strong>Current value:</strong> <code>{}</code></div>"#,
                escape_html(&actual.to_string())
            )
        } else {
            String::new()
        };

        findings_html.push_str(&format!(
            r#"
            <div class="finding-card finding-{severity}">
                <div class="finding-header">
                    <span class="badge {badge_class}">{badge_label}</span>
                    <span class="category-tag">{category}</span>
                    <h3 class="finding-title">{title}</h3>
                </div>
                <p class="finding-message">{message}</p>
                {evidence_html}
                <div class="finding-rec">
                    <strong>Recommendation:</strong> {recommendation}
                </div>
            </div>
            "#,
            severity = f.severity.as_str(),
            badge_class = badge_class,
            badge_label = badge_label,
            category = escape_html(f.category.display_name()),
            title = escape_html(&f.title),
            message = escape_html(&f.message),
            evidence_html = evidence_html,
            recommendation = escape_html(&f.recommendation),
        ));
    }

    let mut benchmarks_html = String::new();
    for b in &report.benchmarks {
        let status_desc = match b.status {
            BenchmarkStatus::Calculated => "Calculated",
            BenchmarkStatus::InsufficientSample => "Insufficient Sample",
            BenchmarkStatus::MissingListingData => "Listing Data Missing",
            BenchmarkStatus::DatasetUnavailable => "Dataset Unavailable",
        };

        let body_html = if let Some(dist) = &b.distribution {
            if b.status == BenchmarkStatus::Calculated {
                let rank = dist.percentile_rank.unwrap_or(0.0);
                let median = dist.median.unwrap_or(0.0);
                let p25 = dist.p25.unwrap_or(0.0);
                let p75 = dist.p75.unwrap_or(0.0);
                let min = dist.min.unwrap_or(0.0);
                let max = dist.max.unwrap_or(0.0);

                format!(
                    r#"
                    <div class="metric-stats">
                        <div class="stat-box"><span class="stat-label">Your Game</span><span class="stat-val highlight">{val}</span></div>
                        <div class="stat-box"><span class="stat-label">Median</span><span class="stat-val">{median:.0}</span></div>
                        <div class="stat-box"><span class="stat-label">IQR (25th - 75th)</span><span class="stat-val">{p25:.0} - {p75:.0}</span></div>
                        <div class="stat-box"><span class="stat-label">Range</span><span class="stat-val">{min:.0} - {max:.0}</span></div>
                        <div class="stat-box"><span class="stat-label">Percentile Rank</span><span class="stat-val">{rank:.0}th</span></div>
                    </div>
                    <div class="percentile-track">
                        <div class="percentile-fill" style="width: {rank}%"></div>
                        <div class="percentile-marker" style="left: {rank}%"></div>
                    </div>
                    <div class="metric-meta">
                        <span>Cohort: <em>{cohort}</em></span>
                        <span>Sample: <strong>{usable}/{total} usable</strong> ({cov:.0}% coverage)</span>
                    </div>
                    "#,
                    val = escape_html(&b.listing_value_display),
                    median = median,
                    p25 = p25,
                    p75 = p75,
                    min = min,
                    max = max,
                    rank = rank,
                    cohort = escape_html(&b.cohort_definition),
                    usable = dist.usable_records,
                    total = dist.sample_size,
                    cov = dist.coverage_pct,
                )
            } else {
                format!(
                    r#"<div class="benchmark-note">{}</div>"#,
                    escape_html(
                        b.explanation
                            .as_deref()
                            .unwrap_or("Comparison unavailable.")
                    )
                )
            }
        } else if let Some(cat) = &b.categorical {
            if b.status == BenchmarkStatus::Calculated {
                format!(
                    r#"
                    <div class="metric-stats">
                        <div class="stat-box"><span class="stat-label">Your Game</span><span class="stat-val highlight">{val}</span></div>
                        <div class="stat-box"><span class="stat-label">Cohort Adoption</span><span class="stat-val">{pct:.0}%</span></div>
                        <div class="stat-box"><span class="stat-label">Supporting Games</span><span class="stat-val">{true_count}/{usable}</span></div>
                    </div>
                    <div class="metric-meta">
                        <span>Cohort: <em>{cohort}</em></span>
                        <span>Sample: <strong>{usable}/{total} usable</strong></span>
                    </div>
                    "#,
                    val = escape_html(&b.listing_value_display),
                    pct = cat.true_percentage,
                    true_count = cat.true_count,
                    usable = cat.usable_records,
                    total = cat.sample_size,
                    cohort = escape_html(&b.cohort_definition),
                )
            } else {
                format!(
                    r#"<div class="benchmark-note">{}</div>"#,
                    escape_html(
                        b.explanation
                            .as_deref()
                            .unwrap_or("Comparison unavailable.")
                    )
                )
            }
        } else {
            format!(
                r#"<div class="benchmark-note">{}</div>"#,
                escape_html(
                    b.explanation
                        .as_deref()
                        .unwrap_or("Comparison unavailable.")
                )
            )
        };

        benchmarks_html.push_str(&format!(
            r#"
            <div class="benchmark-card">
                <div class="benchmark-card-header">
                    <h4>{name}</h4>
                    <span class="status-pill status-{status_class}">{status}</span>
                </div>
                {body}
            </div>
            "#,
            name = escape_html(&b.metric_name),
            status = status_desc,
            status_class = match b.status {
                BenchmarkStatus::Calculated => "calculated",
                _ => "other",
            },
            body = body_html
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Steam Storefront Audit — {title} ({app_id})</title>
    <style>
        :root {{
            --bg: #0f141c;
            --surface: #18202c;
            --surface-border: #283548;
            --text-main: #e2e8f0;
            --text-muted: #94a3b8;
            --accent: #38bdf8;
            --critical: #ef4444;
            --warning: #f59e0b;
            --info: #3b82f6;
            --pass: #10b981;
        }}
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
            background-color: var(--bg);
            color: var(--text-main);
            line-height: 1.5;
            padding: 2rem 1rem;
        }}
        .container {{
            max-width: 1100px;
            margin: 0 auto;
        }}
        header.report-header {{
            background: var(--surface);
            border: 1px solid var(--surface-border);
            border-radius: 8px;
            padding: 1.75rem;
            margin-bottom: 2rem;
        }}
        .header-top {{
            display: flex;
            justify-content: space-between;
            align-items: flex-start;
            flex-wrap: wrap;
            gap: 1rem;
            margin-bottom: 1rem;
        }}
        h1.game-title {{
            font-size: 1.85rem;
            font-weight: 700;
            color: #fff;
        }}
        .meta-badges {{
            display: flex;
            gap: 0.5rem;
            flex-wrap: wrap;
            margin-top: 0.4rem;
        }}
        .badge {{
            display: inline-block;
            font-size: 0.75rem;
            font-weight: 700;
            text-transform: uppercase;
            padding: 0.25rem 0.6rem;
            border-radius: 4px;
            letter-spacing: 0.05em;
        }}
        .badge-critical {{ background: rgba(239, 68, 68, 0.2); color: #fca5a5; border: 1px solid var(--critical); }}
        .badge-warning {{ background: rgba(245, 158, 11, 0.2); color: #fde68a; border: 1px solid var(--warning); }}
        .badge-info {{ background: rgba(59, 130, 246, 0.2); color: #bfdbfe; border: 1px solid var(--info); }}
        .badge-pass {{ background: rgba(16, 185, 129, 0.2); color: #a7f3d0; border: 1px solid var(--pass); }}

        .kpi-row {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
            gap: 1rem;
            margin-top: 1.25rem;
            padding-top: 1.25rem;
            border-top: 1px solid var(--surface-border);
        }}
        .kpi-card {{
            background: #111824;
            padding: 1rem;
            border-radius: 6px;
            text-align: center;
        }}
        .kpi-val {{ font-size: 1.75rem; font-weight: 800; display: block; }}
        .kpi-lbl {{ font-size: 0.75rem; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.05em; }}

        h2.section-heading {{
            font-size: 1.35rem;
            font-weight: 700;
            margin: 2.5rem 0 1rem;
            padding-bottom: 0.5rem;
            border-bottom: 1px solid var(--surface-border);
            display: flex;
            justify-content: space-between;
            align-items: center;
        }}

        .findings-list {{ display: flex; flex-direction: column; gap: 1rem; }}
        .finding-card {{
            background: var(--surface);
            border: 1px solid var(--surface-border);
            border-radius: 6px;
            padding: 1.25rem;
        }}
        .finding-critical {{ border-left: 4px solid var(--critical); }}
        .finding-warning {{ border-left: 4px solid var(--warning); }}
        .finding-info {{ border-left: 4px solid var(--info); }}
        .finding-pass {{ border-left: 4px solid var(--pass); }}
        .finding-header {{ display: flex; align-items: center; gap: 0.75rem; margin-bottom: 0.5rem; flex-wrap: wrap; }}
        .category-tag {{ font-size: 0.75rem; color: var(--text-muted); font-weight: 600; text-transform: uppercase; }}
        .finding-title {{ font-size: 1.05rem; font-weight: 600; color: #fff; }}
        .finding-message {{ color: #cbd5e1; font-size: 0.95rem; margin-bottom: 0.6rem; }}
        .finding-rec {{
            margin-top: 0.6rem;
            padding: 0.6rem 0.8rem;
            background: #121a24;
            border-radius: 4px;
            font-size: 0.88rem;
            color: #93c5fd;
        }}
        .evidence {{ font-size: 0.85rem; color: var(--text-muted); margin-bottom: 0.4rem; }}
        .evidence code {{ background: #0b0f15; padding: 0.2rem 0.4rem; border-radius: 3px; color: #e2e8f0; }}

        .benchmarks-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(480px, 1fr));
            gap: 1.25rem;
        }}
        @media (max-width: 600px) {{
            .benchmarks-grid {{ grid-template-columns: 1fr; }}
        }}
        .benchmark-card {{
            background: var(--surface);
            border: 1px solid var(--surface-border);
            border-radius: 6px;
            padding: 1.25rem;
        }}
        .benchmark-card-header {{ display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; }}
        .benchmark-card-header h4 {{ font-size: 1rem; font-weight: 600; color: #fff; }}
        .status-pill {{ font-size: 0.7rem; font-weight: 600; padding: 0.2rem 0.5rem; border-radius: 4px; text-transform: uppercase; }}
        .status-calculated {{ background: rgba(16, 185, 129, 0.2); color: #34d399; }}
        .status-other {{ background: rgba(148, 163, 184, 0.2); color: #94a3b8; }}

        .metric-stats {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(90px, 1fr));
            gap: 0.5rem;
            margin-bottom: 1rem;
        }}
        .stat-box {{ background: #111824; padding: 0.6rem; border-radius: 4px; text-align: center; }}
        .stat-label {{ font-size: 0.68rem; color: var(--text-muted); text-transform: uppercase; display: block; }}
        .stat-val {{ font-size: 1rem; font-weight: 700; color: #f1f5f9; }}
        .stat-val.highlight {{ color: var(--accent); }}

        .percentile-track {{
            position: relative;
            height: 10px;
            background: #202b3a;
            border-radius: 5px;
            margin: 0.75rem 0;
            overflow: visible;
        }}
        .percentile-fill {{
            height: 100%;
            background: linear-gradient(90deg, #0284c7, #38bdf8);
            border-radius: 5px;
        }}
        .percentile-marker {{
            position: absolute;
            top: -3px;
            width: 4px;
            height: 16px;
            background: #fff;
            border-radius: 2px;
            transform: translateX(-50%);
            box-shadow: 0 0 6px rgba(255,255,255,0.6);
        }}
        .metric-meta {{
            font-size: 0.78rem;
            color: var(--text-muted);
            display: flex;
            justify-content: space-between;
            margin-top: 0.5rem;
            flex-wrap: wrap;
            gap: 0.5rem;
        }}
        .benchmark-note {{ font-size: 0.88rem; color: var(--text-muted); font-style: italic; }}

        footer.report-footer {{
            margin-top: 3rem;
            padding: 1.5rem;
            background: var(--surface);
            border: 1px solid var(--surface-border);
            border-radius: 6px;
            font-size: 0.82rem;
            color: var(--text-muted);
            line-height: 1.6;
        }}
        footer.report-footer strong {{ color: #e2e8f0; }}
        a {{ color: var(--accent); text-decoration: none; }}
        a:hover {{ text-decoration: underline; }}

        @media print {{
            body {{ background: #fff; color: #000; padding: 0; }}
            header.report-header, .finding-card, .benchmark-card, footer.report-footer {{
                background: #fff !important;
                border: 1px solid #ccc !important;
                color: #000 !important;
            }}
            .finding-title, .benchmark-card-header h4, h1.game-title {{ color: #000 !important; }}
            .percentile-track {{ background: #eee; }}
            .percentile-fill {{ background: #999; }}
        }}
    </style>
</head>
<body>
    <div class="container">
        <header class="report-header">
            <div class="header-top">
                <div>
                    <h1 class="game-title">{title}</h1>
                    <div class="meta-badges">
                        <span class="badge badge-info">AppID: {app_id}</span>
                        <span class="badge badge-info">Profile: {profile}</span>
                        <span class="badge badge-pass"><a href="{store_url}" target="_blank" style="color: inherit;">View on Steam ↗</a></span>
                    </div>
                </div>
                <div style="text-align: right; font-size: 0.8rem; color: var(--text-muted);">
                    <div>Generated: <strong>{generated_at}</strong></div>
                    <div>Schema Version: <strong>{schema_ver}</strong></div>
                </div>
            </div>

            <div class="kpi-row">
                <div class="kpi-card"><span class="kpi-val" style="color: var(--critical);">{crit}</span><span class="kpi-lbl">Critical</span></div>
                <div class="kpi-card"><span class="kpi-val" style="color: var(--warning);">{warn}</span><span class="kpi-lbl">Warnings</span></div>
                <div class="kpi-card"><span class="kpi-val" style="color: var(--info);">{info}</span><span class="kpi-lbl">Informational</span></div>
                <div class="kpi-card"><span class="kpi-val" style="color: var(--pass);">{pass_count}</span><span class="kpi-lbl">Passed</span></div>
                <div class="kpi-card"><span class="kpi-val" style="color: var(--accent);">{comp_pct:.0}%</span><span class="kpi-lbl">Completeness</span></div>
            </div>
        </header>

        <section>
            <h2 class="section-heading">Listing Findings ({total_findings})</h2>
            <div class="findings-list">
                {findings_html}
            </div>
        </section>

        <section>
            <h2 class="section-heading">Peer Benchmarking & Distributions</h2>
            <div class="benchmarks-grid">
                {benchmarks_html}
            </div>
        </section>

        <footer class="report-footer">
            <p><strong>Benchmark Methodology & Attribution:</strong></p>
            <p>{methodology_disclaimer}</p>
            <p style="margin-top: 0.5rem;">Dataset: <strong>{dataset_source}</strong> ({dataset_license} License). Snapshot date: <strong>{snapshot_date}</strong>. Percentile calculation: <strong>{percentile_conv}</strong>.</p>
            <p style="margin-top: 0.5rem; font-style: italic;">Disclaimer: Steam Storefront Linter audits listing completeness and metadata formatting. It does not predict sales, commercial success, review sentiment, or player engagement.</p>
        </footer>
    </div>
</body>
</html>
"#,
        title = title,
        app_id = app_id,
        profile = profile,
        store_url = store_url,
        generated_at = generated_at,
        schema_ver = report.schema_version,
        crit = report.summary.critical,
        warn = report.summary.warnings,
        info = report.summary.info,
        pass_count = report.summary.passed,
        comp_pct = report.data_quality.data_completeness_pct,
        total_findings = report.findings.len(),
        findings_html = findings_html,
        benchmarks_html = benchmarks_html,
        methodology_disclaimer = escape_html(&report.methodology.disclaimer),
        dataset_source = escape_html(&report.methodology.dataset_source),
        dataset_license = escape_html(&report.methodology.dataset_license),
        snapshot_date = escape_html(&report.methodology.snapshot_date),
        percentile_conv = escape_html(&report.methodology.percentile_convention),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::benchmark::BenchmarkMethodology;
    use crate::models::listing::{ListingSource, StoreListing};
    use crate::rules::profile::AuditProfile;

    #[test]
    fn test_generate_html_report() {
        let listing = StoreListing::new(2539430, "Black Dragon Mage", ListingSource::JsonUpload);
        let methodology = BenchmarkMethodology {
            dataset_source: "FronkonGames/steam-games-dataset".into(),
            dataset_license: "MIT".into(),
            snapshot_date: "2024-11".into(),
            total_dataset_games: 1000,
            min_sample_size_threshold: 30,
            percentile_convention: "Linear rank interpolation".into(),
            disclaimer: "Benchmarking provides descriptive context only.".into(),
        };

        let report = AuditReport::new(
            listing,
            vec![],
            vec![],
            None,
            methodology,
            AuditProfile::Balanced,
        );
        let html = generate_html_report(&report);
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Black Dragon Mage"));
        assert!(html.contains("2539430"));
    }
}
