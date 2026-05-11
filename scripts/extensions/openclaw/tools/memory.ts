import { AgentSQLClient, SaveMemoryRequest, LoadMemoryRequest, SearchMemoryRequest, ClearMemoryRequest, MemoryStats, MemoryEntry } from '../client';

export interface SaveMemoryInput {
  content: string;
  memory_type?: 'conversation' | 'query' | 'result' | 'schema' | 'policy' | 'custom';
  tags?: string[];
  agent_id?: string;
  session_id?: string;
  importance?: number;
  metadata?: Record<string, string>;
}

export interface LoadMemoryInput {
  agent_id?: string;
  session_id?: string;
  memory_type?: string;
  tags?: string[];
  limit?: number;
  since?: number;
}

export interface SearchMemoryInput {
  query: string;
  agent_id?: string;
  memory_type?: string;
  limit?: number;
}

export interface ClearMemoryInput {
  agent_id?: string;
  session_id?: string;
  memory_type?: string;
  older_than?: number;
}

export interface MemoryOperationOutput {
  success: boolean;
  id?: string;
  memories?: MemoryEntry[];
  total?: number;
  cleared?: number;
  error?: string;
}

export interface StatsOutput {
  total_memories?: number;
  by_type?: Record<string, number>;
  unique_agents?: number;
  unique_sessions?: number;
  unique_tags?: number;
  error?: string;
}

export interface ToolDefinition {
  name: string;
  description: string;
  inputSchema: {
    type: 'object';
    properties: {
      operation: {
        type: 'string';
        enum: ['save', 'load', 'search', 'clear', 'stats'];
        description: 'Memory operation to perform';
      };
      content?: {
        type: 'string';
        description: 'Content to save (for save operation)';
      };
      query?: {
        type: 'string';
        description: 'Search query (for search operation)';
      };
      agent_id?: {
        type: 'string';
        description: 'Agent ID for memory filtering';
      };
      session_id?: {
        type: 'string';
        description: 'Session ID for memory filtering';
      };
      memory_type?: {
        type: 'string';
        description: 'Type of memory (conversation, query, result, schema, policy, custom)';
      };
      tags?: {
        type: 'string[]';
        description: 'Tags for memory categorization';
      };
      limit?: {
        type: 'number';
        description: 'Maximum number of memories to return';
      };
      importance?: {
        type: 'number';
        description: 'Importance score (0-1) for saved memory';
      };
      older_than?: {
        type: 'number';
        description: 'Delete memories older than this Unix timestamp';
      };
    };
    required: ['operation'];
  };
}

export class MemoryTool {
  private client: AgentSQLClient;
  private definition: ToolDefinition;

  constructor(client: AgentSQLClient) {
    this.client = client;
    this.definition = {
      name: 'agentsql_memory',
      description: 'Manage agent memory for storing and retrieving conversation context, queries, and results.',
      inputSchema: {
        type: 'object',
        properties: {
          operation: {
            type: 'string',
            enum: ['save', 'load', 'search', 'clear', 'stats'],
            description: 'Memory operation: save (store new memory), load (retrieve memories), search (full-text search), clear (remove memories), stats (get statistics)',
          },
          content: {
            type: 'string',
            description: 'Content to save to memory (required for save operation)',
          },
          query: {
            type: 'string',
            description: 'Search query (required for search operation)',
          },
          agent_id: {
            type: 'string',
            description: 'Agent ID to filter memories',
          },
          session_id: {
            type: 'string',
            description: 'Session ID to filter memories',
          },
          memory_type: {
            type: 'string',
            description: 'Type of memory: conversation, query, result, schema, policy, custom',
          },
          tags: {
            type: 'string',
            description: 'Comma-separated tags for memory categorization',
          },
          limit: {
            type: 'number',
            description: 'Maximum number of memories to return',
          },
          importance: {
            type: 'number',
            description: 'Importance score (0-1) for saved memory',
          },
          older_than: {
            type: 'number',
            description: 'Delete memories older than this Unix timestamp (for clear operation)',
          },
        },
        required: ['operation'],
      },
    };
  }

  getToolDefinition(): ToolDefinition {
    return this.definition;
  }

  async execute(input: Record<string, unknown>, context?: Record<string, unknown>): Promise<MemoryOperationOutput | StatsOutput> {
    const operation = input.operation as string;

    try {
      switch (operation) {
        case 'save':
          return this.handleSave(input);
        case 'load':
          return this.handleLoad(input);
        case 'search':
          return this.handleSearch(input);
        case 'clear':
          return this.handleClear(input);
        case 'stats':
          return this.handleStats();
        default:
          return {
            success: false,
            error: `Unknown operation: ${operation}`,
          };
      }
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }

  private async handleSave(input: Record<string, unknown>): Promise<MemoryOperationOutput> {
    const content = input.content as string;
    if (!content) {
      return { success: false, error: 'content is required for save operation' };
    }

    const request: SaveMemoryRequest = {
      content,
      memory_type: input.memory_type as SaveMemoryRequest['memory_type'],
      agent_id: input.agent_id as string,
      session_id: input.session_id as string,
      importance: input.importance as number,
    };

    if (input.tags) {
      request.tags = String(input.tags).split(',').map(t => t.trim());
    }

    const response = await this.client.saveMemory(request);
    return {
      success: response.success,
      id: response.id,
    };
  }

  private async handleLoad(input: Record<string, unknown>): Promise<MemoryOperationOutput> {
    const request: LoadMemoryRequest = {
      agent_id: input.agent_id as string,
      session_id: input.session_id as string,
      memory_type: input.memory_type as string,
      limit: input.limit as number,
    };

    if (input.tags) {
      request.tags = String(input.tags).split(',').map(t => t.trim());
    }

    const response = await this.client.loadMemory(request);
    return {
      success: true,
      memories: response.memories,
      total: response.total,
    };
  }

  private async handleSearch(input: Record<string, unknown>): Promise<MemoryOperationOutput> {
    const query = input.query as string;
    if (!query) {
      return { success: false, error: 'query is required for search operation' };
    }

    const request: SearchMemoryRequest = {
      query,
      agent_id: input.agent_id as string,
      memory_type: input.memory_type as string,
      limit: input.limit as number,
    };

    const response = await this.client.searchMemory(request);
    return {
      success: true,
      memories: response.results,
      total: response.total,
    };
  }

  private async handleClear(input: Record<string, unknown>): Promise<MemoryOperationOutput> {
    const request: ClearMemoryRequest = {
      agent_id: input.agent_id as string,
      session_id: input.session_id as string,
      memory_type: input.memory_type as string,
      older_than: input.older_than as number,
    };

    const response = await this.client.clearMemory(request);
    return {
      success: response.success,
      cleared: response.cleared,
    };
  }

  private async handleStats(): Promise<StatsOutput> {
    const stats: MemoryStats = await this.client.getMemoryStats();
    return {
      total_memories: stats.total_memories,
      by_type: stats.by_type,
      unique_agents: stats.unique_agents,
      unique_sessions: stats.unique_sessions,
      unique_tags: stats.unique_tags,
    };
  }
}
