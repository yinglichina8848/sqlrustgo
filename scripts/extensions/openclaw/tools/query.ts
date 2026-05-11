import { AgentSQLClient, QueryRequest, QueryResponse } from '../client';

export interface QueryInput {
  sql: string;
}

export interface QueryOutput {
  success: boolean;
  data?: unknown[][];
  error?: string;
  execution_time_ms?: number;
  row_count?: number;
}

export interface ToolDefinition {
  name: string;
  description: string;
  inputSchema: {
    type: 'object';
    properties: {
      sql: {
        type: 'string';
        description: 'SQL query to execute';
      };
    };
    required: ['sql'];
  };
}

export class QueryTool {
  private client: AgentSQLClient;
  private definition: ToolDefinition;

  constructor(client: AgentSQLClient) {
    this.client = client;
    this.definition = {
      name: 'agentsql_query',
      description: 'Execute a SQL query against the SQLRustGo database. Use this tool to run SELECT, INSERT, UPDATE, DELETE, or other SQL statements.',
      inputSchema: {
        type: 'object',
        properties: {
          sql: {
            type: 'string',
            description: 'The SQL query to execute. Example: SELECT * FROM users WHERE id = 1',
          },
        },
        required: ['sql'],
      },
    };
  }

  getToolDefinition(): ToolDefinition {
    return this.definition;
  }

  async execute(input: QueryInput, context?: Record<string, unknown>): Promise<QueryOutput> {
    try {
      const response = await this.client.query(input.sql);
      
      const result: QueryOutput = {
        success: response.success,
        execution_time_ms: response.execution_time_ms,
      };

      if (response.success && response.data) {
        result.data = response.data;
        result.row_count = response.data.length;
      }

      if (response.error) {
        result.error = response.error;
      }

      return result;
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }
}
