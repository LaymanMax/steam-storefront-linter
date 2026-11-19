# Security & Privacy Policy

Steam Storefront Linter is designed to operate safely on untrusted external Steam responses and uploaded user files.

---

## 1. Untrusted Input Handling

- **No Execution of Imported HTML:** Store descriptions returned by Steam Store API or user JSON uploads contain HTML formatting tags (`<p>`, `<b>`, `<img>`). These descriptions are never injected into the DOM as executable HTML, and standalone HTML reports escape all tags using strict HTML entity encoding.
- **Strict Size Limits:** The HTTP API and JSON parser enforce a strict **10 MB payload limit** (`DefaultBodyLimit::max(10 * 1024 * 1024)`). Requests exceeding this limit are terminated with HTTP 413 (`Payload Too Large`).
- **Safe Path Traversal Protections:** File export routes write only to caller-specified local files and do not accept arbitrary client paths for server-side persistence.

---

## 2. Network & Privacy

- **No Telemetry or Tracking:** The application does not collect usage telemetry, user analytics, or phone-home pings.
- **No Third-Party AI Keys:** Audits run 100% deterministically using the built-in rule engine. No OpenAI, Anthropic, or external LLM API keys are required or contacted.
- **Respectful Steam Rate Limiting:** The `SteamClient` incorporates an in-memory TTL cache and detects HTTP 429 rate limit responses, pausing and surfacing clear guidance rather than hammering Steam servers.
- **No Private Credentials:** Accessing the Steam app details endpoint and downloading the Hugging Face benchmark dataset are completely public and require zero API tokens.

---

## 3. Reporting a Vulnerability

If you discover a security concern or vulnerability, please open a private security advisory on GitHub or email contact@laymanmax.com.
