import { AgentSQLClient, StatsResponse, QueryStatistics } from '../client';

export interface StatsInput {
  table?: string;
}

export interface StatsOutput {
  tables?: Record<string, { row_count: number; size_bytes: number; indexes_count: number }>;
  query_statistics?: QueryStatistics;
  error?: string;
}

export interface ToolDefinition {
  name: string;
  description: string;
  inputSchema: {
    type: 'object';
    properties: {
      table: {
        type: 'string';
        description: 'Optional table name to get stats for. If not provided, returns overall stats.';
      };
    };
  };
}

export class StatsTool {
  private client: AgentSQLClient;
  private definition: ToolDefinition;

  constructor(client: AgentSQLClient) {
    this.client = client;
    this.definition = {
      name: 'agentsql_stats',
      description: 'Get database statistics including table sizes, row counts, and query performance metrics.',
      inputSchema: {
        type: 'object',
        properties: {
          table: {
            type: 'string',
            description: 'Optional table name to get specific stats for.',
          },
        },
      },
    };
  }

  getToolDefinition(): ToolDefinition {
    return this.definition;
  }

  async execute(input: StatsInput, context?: Record<string, unknown>): Promise<StatsOutput> {
    try {
      if (input.table) {
        const stats = await this.client.getTableStats(input.table);
        
        if ('error' in stats) {
          return {
            error: stats.error,
          };
        }

        return {
          tables: {
            [input.table]: stats,
          },
        };
      } else {
        const response: StatsResponse = await this.client.getStats();
        
        return {
          tables: response.tables,
          query_statistics: response.query_statistics,
        };
      }
    } catch (error) {
      return {
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }
}
