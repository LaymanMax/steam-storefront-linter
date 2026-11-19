# Benchmark Methodology & Cohort Selection

## Product Vision & Principles

Steam Storefront Linter is a **listing completeness auditor and descriptive benchmarking tool**, not a commercial success predictor.

The system does **NOT**:

- Predict sales revenue, units sold, or peak concurrent users (CCU).
- Score game quality or gameplay mechanics.
- Predict Steam recommendation algorithm rankings or review sentiment.

A game with 15 screenshots is not inherently better than one with 6 screenshots, and Linux native support does not guarantee sales. Descriptive benchmarking provides peer context so indie developers can see how their store presence compares with peer titles in their genre and release window.

---

## 1. Comparison Cohort Selection

Games are compared against relevant cohorts rather than the entire 140,000+ game catalog. Cohorts are selected using a multi-stage fallback policy to balance specificity with sample significance:

1. **Specific Cohort (Preferred):**
   - Matches one or more primary genres.
   - Matches the target game's release year.
   - Matches the Indie classification when the target game is tagged as Indie.
2. **Stage 2 Fallback:**
   - Same genre(s) + same release year (relaxing the Indie restriction if sample size is below the minimum threshold).
3. **Stage 3 Fallback:**
   - Same genre(s) across all release years.
4. **Stage 4 Fallback:**
   - All Indie games released in the same calendar year.
5. **Stage 5 Fallback:**
   - Broad eligible benchmark pool.

### Minimum Sample Threshold

The default threshold is **30 usable records**. If a cohort contains fewer than 30 usable values for a given metric, the system marks the metric status as `insufficient_sample` rather than fabricating comparisons.

---

## 2. Statistical Definitions & Formulas

For any quantitative metric (e.g. screenshot count, trailer count, description word count, language count):

1. **Missing Data Handling:** Records where a field was omitted or unparseable are excluded from that metric's calculation. `usable_records` and `coverage_pct` are explicitly reported. Missing values are **never converted to 0**.
2. **Median (50th percentile):** Value dividing the sorted distribution into two equal halves.
3. **Interquartile Range (IQR):**
   - **P25 (25th percentile):** Lower quartile.
   - **P75 (75th percentile):** Upper quartile.
4. **Percentile Rank:**
   $$\text{Percentile Rank} = \frac{\text{count}(\text{value} < v) + 0.5 \times \text{count}(\text{value} = v)}{\text{total usable records}} \times 100$$
5. **Share of Games Above:**
   $$\text{Share Above} = \frac{\text{count}(\text{value} > v)}{\text{total usable records}} \times 100$$

Mathematical consistency is strictly preserved: a listing at the 25th percentile with value $v$ will correctly report that approximately 75% of comparable games have higher values.

---

## 3. Categorical Metrics

For binary store capabilities (Trailer Presence, Support Contact, Linux Support, macOS Support):

- Calculated as:
  $$\text{Adoption Percentage} = \frac{\text{count}(\text{feature} = \text{true})}{\text{total usable records}} \times 100$$
- Reports both the percentage and the raw fraction (e.g., `28/33 games`).
