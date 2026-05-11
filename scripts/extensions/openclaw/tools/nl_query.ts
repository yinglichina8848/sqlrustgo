import { AgentSQLClient, NlQueryRequest, NlQueryResponse } from '../client';

export interface NlQueryInput {
  query: string;
  context?: string;
}

export interface NlQueryOutput {
  success: boolean;
  sql?: string;
  confidence?: number;
  table_hint?: string;
  where_conditions?: string[];
  error?: string;
}

export interface ToolDefinition {
  name: string;
  description: string;
  inputSchema: {
    type: 'object';
    properties: {
      query: {
        type: 'string';
        description: 'Natural language query to convert to SQL';
      };
      context?: {
        type: 'string';
        description: 'Optional context to help with query interpretation';
      };
    };
    required: ['query'];
  };
}

export class NlQueryTool {
  private client: AgentSQLClient;
  private definition: ToolDefinition;

  constructor(client: AgentSQLClient) {
    this.client = client;
    this.definition = {
      name: 'agentsql_nl_query',
      description: 'Convert a natural language query to SQL. Use this tool when the user asks questions in plain English rather than SQL.',
      inputSchema: {
        type: 'object',
        properties: {
          query: {
            type: 'string',
            description: 'Natural language query. Examples: "show all users", "find orders where status is pending", "count total products"',
          },
          context: {
            type: 'string',
            description: 'Optional context to help interpret the query, such as previous conversation or specific requirements',
          },
        },
        required: ['query'],
      },
    };
  }

  getToolDefinition(): ToolDefinition {
    return this.definition;
  }

  async execute(input: NlQueryInput, context?: Record<string, unknown>): Promise<NlQueryOutput> {
    try {
      const request: NlQueryRequest = {
        query: input.query,
        context: input.context,
      };

      const response = await this.client.nlQuery(request.query, request.context);
      
      const result: NlQueryOutput = {
        success: response.success,
      };

      if (response.success) {
        result.sql = response.sql;
        result.confidence = response.confidence;
        result.table_hint = response.table_hint;
        result.where_conditions = response.where_conditions;
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
