interface RequestOptions {
  method?: string;
  body?: unknown;
  headers?: Record<string, string>;
}

interface Project {
  id: string;
  name: string;
  description?: string;
  status: string;
  created_at: string;
  updated_at: string;
}

interface Upload {
  id: string;
  project_id: string;
  filename: string;
  size: number;
  content_type: string;
  storage_path: string;
  hash: string;
  created_at: string;
}

interface Job {
  id: string;
  project_id: string;
  job_type: string;
  status: string;
  params: Record<string, unknown>;
  result?: Record<string, unknown>;
  created_at: string;
  started_at?: string;
  completed_at?: string;
}

interface Artifact {
  id: string;
  job_id: string;
  filename: string;
  size: number;
  content_type: string;
  storage_path: string;
  hash: string;
  created_at: string;
}

interface JobLog {
  timestamp: string;
  level: string;
  message: string;
}

interface Stats {
  total_projects: number;
  total_uploads: number;
  total_jobs: number;
  total_artifacts: number;
  total_api_keys: number;
  jobs_by_status: Record<string, number>;
  uptime_secs: number;
}

interface ApiKey {
  id: string;
  key_prefix: string;
  name: string;
  project_id?: string;
  permissions: string[];
  expires_at?: string;
  created_at: string;
}

interface CreatedApiKey {
  api_key_id: string;
  api_key: string;
  key_prefix: string;
}

interface Health {
  status: string;
  version: string;
  uptime_secs: number;
}

interface JobCreated {
  job_id: string;
  project_id: string;
  status: string;
  job_type: string;
}

export class CirbiniusClient {
  private baseUrl: string;
  private apiKey?: string;

  constructor(host: string = '127.0.0.1', port: number = 8080, apiKey?: string) {
    this.baseUrl = `http://${host}:${port}`;
    this.apiKey = apiKey;
  }

  private async request<T>(method: string, path: string, body?: unknown): Promise<T> {
    const headers: Record<string, string> = { 'Content-Type': 'application/json' };
    if (this.apiKey) headers['Authorization'] = `Bearer ${this.apiKey}`;

    const opts: RequestInit = { method, headers };
    if (body !== undefined) opts.body = JSON.stringify(body);

    const res = await fetch(`${this.baseUrl}${path}`, opts);
    if (!res.ok) {
      const text = await res.text();
      throw new Error(`${res.status}: ${text}`);
    }
    const ct = res.headers.get('content-type') || '';
    if (ct.includes('json')) return res.json();
    return res.text() as unknown as T;
  }

  // Health
  async health(): Promise<Health> { return this.request('GET', '/health'); }

  // Projects
  async listProjects(): Promise<Project[]> { return this.request('GET', '/api/v1/projects'); }
  async createProject(name: string, description?: string): Promise<Project> { return this.request('POST', '/api/v1/projects', { name, description }); }
  async getProject(id: string): Promise<Project> { return this.request('GET', `/api/v1/projects/${id}`); }
  async updateProject(id: string, updates: Partial<Project>): Promise<Project> { return this.request('PATCH', `/api/v1/projects/${id}`, updates); }
  async deleteProject(id: string): Promise<{ deleted: boolean }> { return this.request('DELETE', `/api/v1/projects/${id}`); }

  // Uploads
  async listUploads(projectId: string): Promise<Upload[]> { return this.request('GET', `/api/v1/projects/${projectId}/uploads`); }
  async uploadFile(projectId: string, filename: string, contentType: string, data: string): Promise<Upload> {
    const path = `/api/v1/projects/${projectId}/uploads?filename=${filename}&content_type=${contentType}`;
    const res = await fetch(`${this.baseUrl}${path}`, { method: 'POST', headers: this.apiKey ? { 'Authorization': `Bearer ${this.apiKey}` } : {}, body: data });
    if (!res.ok) throw new Error(`${res.status}: ${await res.text()}`);
    return res.json();
  }
  async getUpload(projectId: string, uploadId: string): Promise<Upload> { return this.request('GET', `/api/v1/projects/${projectId}/uploads/${uploadId}`); }
  async deleteUpload(projectId: string, uploadId: string): Promise<{ deleted: boolean }> { return this.request('DELETE', `/api/v1/projects/${projectId}/uploads/${uploadId}`); }

  // Jobs
  async listJobs(projectId: string): Promise<Job[]> { return this.request('GET', `/api/v1/projects/${projectId}/jobs`); }
  async createCompileJob(projectId: string, params?: Record<string, unknown>): Promise<JobCreated> { return this.request('POST', `/api/v1/projects/${projectId}/compile`, params ?? {}); }
  async createProveJob(projectId: string, params?: Record<string, unknown>): Promise<JobCreated> { return this.request('POST', `/api/v1/projects/${projectId}/prove`, params ?? {}); }
  async createVerifyJob(projectId: string, params?: Record<string, unknown>): Promise<JobCreated> { return this.request('POST', `/api/v1/projects/${projectId}/verify`, params ?? {}); }
  async createAnalyzeJob(projectId: string, params?: Record<string, unknown>): Promise<JobCreated> { return this.request('POST', `/api/v1/projects/${projectId}/analyze`, params ?? {}); }
  async createConformanceJob(projectId: string, params?: Record<string, unknown>): Promise<JobCreated> { return this.request('POST', `/api/v1/projects/${projectId}/conformance`, params ?? {}); }
  async getJob(projectId: string, jobId: string): Promise<Job> { return this.request('GET', `/api/v1/projects/${projectId}/jobs/${jobId}`); }
  async cancelJob(projectId: string, jobId: string): Promise<Job> { return this.request('POST', `/api/v1/projects/${projectId}/jobs/${jobId}/cancel`); }
  async getJobLogs(projectId: string, jobId: string): Promise<JobLog[]> { return this.request('GET', `/api/v1/projects/${projectId}/jobs/${jobId}/logs`); }

  // Artifacts
  async listArtifacts(jobId: string): Promise<Artifact[]> { return this.request('GET', `/api/v1/jobs/${jobId}/artifacts`); }
  async getArtifact(jobId: string, artifactId: string): Promise<Artifact> { return this.request('GET', `/api/v1/jobs/${jobId}/artifacts/${artifactId}`); }
  async downloadArtifact(jobId: string, artifactId: string): Promise<string> { return this.request('GET', `/api/v1/jobs/${jobId}/artifacts/${artifactId}/download`); }

  // Admin
  async getStats(): Promise<Stats> { return this.request('GET', '/api/v1/admin/stats'); }
  async listApiKeys(): Promise<ApiKey[]> { return this.request('GET', '/api/v1/admin/api-keys'); }
  async createApiKey(name: string, permissions?: string[], expiresInDays?: number): Promise<CreatedApiKey> {
    return this.request('POST', '/api/v1/admin/api-keys', { name, permissions: permissions ?? ['read'], expires_in_days: expiresInDays });
  }
  async deleteApiKey(keyId: string): Promise<{ deleted: boolean }> { return this.request('DELETE', `/api/v1/admin/api-keys/${keyId}`); }
  async checkAuth(): Promise<{ authenticated: boolean; key_id: string; key_prefix: string; permissions: string[] }> {
    return this.request('POST', '/api/v1/admin/auth');
  }
}
