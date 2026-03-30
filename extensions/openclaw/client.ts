export interface HealthResponse {
  status: string;
  version: string;
}

export interface QueryRequest {
  sql: string;
}

export interface QueryResponse {
  success: boolean;
  data?: unknown[][];
  error?: string;
  execution_time_ms?: number;
}

export interface NlQueryRequest {
  query: string;
  context?: string;
}

export interface NlQueryResponse {
  success: boolean;
  sql?: string;
  confidence?: number;
  table_hint?: string;
  where_conditions?: string[];
  error?: string;
}

export interface SchemaResponse {
  database: string;
  version: string;
  tables: TableSchema[];
  views: ViewSchema[];
}

export interface TableSchema {
  name: string;
  columns: ColumnSchema[];
  indexes?: IndexSchema[];
  comment?: string;
}

export interface ColumnSchema {
  name: string;
  type: string;
  nullable: boolean;
  primary_key: boolean;
  unique: boolean;
  default?: string;
  max_length?: number;
  precision?: number;
  scale?: number;
  foreign_key?: {
    table: string;
    column: string;
  };
}

export interface ViewSchema {
  name: string;
  definition: string;
}

export interface IndexSchema {
  name: string;
  columns: string[];
  unique: boolean;
  type: string;
}

export interface StatsResponse {
  tables: Record<string, TableStats>;
  query_statistics: QueryStatistics;
}

export interface TableStats {
  row_count: number;
  size_bytes: number;
  indexes_count: number;
}

export interface QueryStatistics {
  total_queries: number;
  select_queries: number;
  insert_queries: number;
  update_queries: number;
  delete_queries: number;
  avg_execution_time_ms: number;
}

export interface SaveMemoryRequest {
  content: string;
  memory_type?: 'conversation' | 'query' | 'result' | 'schema' | 'policy' | 'custom';
  tags?: string[];
  agent_id?: string;
  session_id?: string;
  importance?: number;
  metadata?: Record<string, string>;
}

export interface SaveMemoryResponse {
  id: string;
  success: boolean;
  message: string;
}

export interface LoadMemoryRequest {
  agent_id?: string;
  session_id?: string;
  memory_type?: string;
  tags?: string[];
  limit?: number;
  since?: number;
}

export interface LoadMemoryResponse {
  memories: MemoryEntry[];
  total: number;
}

export interface MemoryEntry {
  id: string;
  content: string;
  memory_type: string;
  timestamp: number;
  tags: string[];
  agent_id?: string;
  session_id?: string;
  importance: number;
  metadata: Record<string, string>;
}

export interface SearchMemoryRequest {
  query: string;
  agent_id?: string;
  memory_type?: string;
  limit?: number;
}

export interface SearchMemoryResponse {
  results: MemoryEntry[];
  total: number;
  scores: number[];
}

export interface ClearMemoryRequest {
  agent_id?: string;
  session_id?: string;
  memory_type?: string;
  older_than?: number;
}

export interface ClearMemoryResponse {
  cleared: number;
  success: boolean;
}

export interface MemoryStats {
  total_memories: number;
  by_type: Record<string, number>;
  unique_agents: number;
  unique_sessions: number;
  unique_tags: number;
}

export class AgentSQLClient {
  private baseUrl: string;

  constructor(baseUrl: string = 'http://localhost:8080') {
    this.baseUrl = baseUrl;
  }

  private async request<T>(
    endpoint: string, 
    method: string = 'GET', 
    body?: unknown
  ): Promise<T> {
    const options: RequestInit = {
      method,
      headers: {
        'Content-Type': 'application/json',
      },
    };

    if (body) {
      options.body = JSON.stringify(body);
    }

    const response = await fetch(`${this.baseUrl}${endpoint}`, options);
    
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    return response.json();
  }

  async health(): Promise<HealthResponse> {
    return this.request<HealthResponse>('/health');
  }

  async query(sql: string): Promise<QueryResponse> {
    return this.request<QueryResponse>('/query', 'POST', { sql });
  }

  async nlQuery(query: string, context?: string): Promise<NlQueryResponse> {
    return this.request<NlQueryResponse>('/nl_query', 'POST', { query, context });
  }

  async getSchema(): Promise<SchemaResponse> {
    return this.request<SchemaResponse>('/schema');
  }

  async getTableSchema(tableName: string): Promise<TableSchema | { error: string }> {
    return this.request<TableSchema | { error: string }>(`/schema/${tableName}`);
  }

  async getStats(): Promise<StatsResponse> {
    return this.request<StatsResponse>('/stats');
  }

  async getTableStats(tableName: string): Promise<TableStats | { error: string }> {
    return this.request<TableStats | { error: string }>(`/stats/${tableName}`);
  }

  async getQueryStats(): Promise<QueryStatistics> {
    return this.request<QueryStatistics>('/stats/queries');
  }

  async saveMemory(request: SaveMemoryRequest): Promise<SaveMemoryResponse> {
    return this.request<SaveMemoryResponse>('/memory/save', 'POST', request);
  }

  async loadMemory(request: LoadMemoryRequest): Promise<LoadMemoryResponse> {
    return this.request<LoadMemoryResponse>('/memory/load', 'POST', request);
  }

  async searchMemory(request: SearchMemoryRequest): Promise<SearchMemoryResponse> {
    return this.request<SearchMemoryResponse>('/memory/search', 'POST', request);
  }

  async clearMemory(request: ClearMemoryRequest): Promise<ClearMemoryResponse> {
    return this.request<ClearMemoryResponse>('/memory/clear', 'POST', request);
  }

  async getMemoryStats(): Promise<MemoryStats> {
    return this.request<MemoryStats>('/memory/stats');
  }
}

export default AgentSQLClient;
