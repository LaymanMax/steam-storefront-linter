import { AuditReport } from "../types";

const API_BASE = "/api/v1";

export class ApiError extends Error {
  constructor(
    message: string,
    public code: string,
    public status: number,
  ) {
    super(message);
    this.name = "ApiError";
  }
}

async function handleResponse<T>(res: Response): Promise<T> {
  if (!res.ok) {
    let errorMsg = `Server error (HTTP ${res.status})`;
    let code = "UNKNOWN_ERROR";
    try {
      const data = await res.json();
      if (data.error) errorMsg = data.error;
      if (data.code) code = data.code;
    } catch {
      // ignore
    }
    throw new ApiError(errorMsg, code, res.status);
  }
  return res.json();
}

export async function checkHealth(): Promise<{
  status: string;
  version: string;
}> {
  const res = await fetch(`${API_BASE}/health`);
  return handleResponse(res);
}

export async function auditSteamApp(
  appIdOrUrl: string,
  options: {
    country?: string;
    language?: string;
    profile?: "balanced" | "strict";
    minSamples?: number;
  } = {},
): Promise<AuditReport> {
  const params = new URLSearchParams();
  if (options.country) params.set("country", options.country);
  if (options.language) params.set("language", options.language);
  if (options.profile) params.set("profile", options.profile);
  if (options.minSamples)
    params.set("min_samples", options.minSamples.toString());

  const encodedTarget = encodeURIComponent(appIdOrUrl.trim());
  const res = await fetch(
    `${API_BASE}/audits/steam/${encodedTarget}?${params.toString()}`,
    {
      method: "POST",
    },
  );
  return handleResponse<AuditReport>(res);
}

export async function auditImportJson(
  jsonData: unknown,
  options: {
    profile?: "balanced" | "strict";
    minSamples?: number;
  } = {},
): Promise<AuditReport> {
  const params = new URLSearchParams();
  if (options.profile) params.set("profile", options.profile);
  if (options.minSamples)
    params.set("min_samples", options.minSamples.toString());

  const res = await fetch(`${API_BASE}/audits/import?${params.toString()}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(jsonData),
  });
  return handleResponse<AuditReport>(res);
}

export async function getSampleReport(): Promise<AuditReport> {
  const res = await fetch(`${API_BASE}/audits/sample-report`);
  return handleResponse<AuditReport>(res);
}

export async function getBenchmarkStatus(): Promise<{
  available: boolean;
  database_path: string;
  metadata?: unknown;
}> {
  const res = await fetch(`${API_BASE}/benchmarks/status`);
  return handleResponse(res);
}

export function getExportJsonUrl(auditId: string): string {
  return `${API_BASE}/audits/${auditId}/json`;
}

export function getExportHtmlUrl(auditId: string): string {
  return `${API_BASE}/audits/${auditId}/html`;
}

export function getDownloadExampleUrl(): string {
  return `${API_BASE}/audits/example`;
}
