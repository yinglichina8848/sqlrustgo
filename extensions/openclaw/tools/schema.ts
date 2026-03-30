import { AgentSQLClient, SchemaResponse, TableSchema } from '../client';

export interface SchemaInput {
  table?: string;
}

export interface SchemaOutput {
  database: string;
  version: string;
  tables?: TableSchema[];
  views?: Array<{ name: string; definition: string }>;
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
        description: 'Optional table name to get schema for a specific table. If not provided, returns all tables.';
      };
    };
  };
}

export class SchemaTool {
  private client: AgentSQLClient;
  private definition: ToolDefinition;

  constructor(client: AgentSQLClient) {
    this.client = client;
    this.definition = {
      name: 'agentsql_schema',
      description: 'Get the database schema including tables, columns, indexes, and views. Use this to understand the available data structure.',
      inputSchema: {
        type: 'object',
        properties: {
          table: {
            type: 'string',
            description: 'Optional table name to get schema for. If omitted, returns all tables.',
          },
        },
      },
    };
  }

  getToolDefinition(): ToolDefinition {
    return this.definition;
  }

  async execute(input: SchemaInput, context?: Record<string, unknown>): Promise<SchemaOutput> {
    try {
      if (input.table) {
        const response = await this.client.getTableSchema(input.table);
        
        if ('error' in response) {
          return {
            database: '',
            version: '',
            error: response.error,
          };
        }

        return {
          database: 'sqlrustgo',
          version: '2.1.0',
          tables: [response],
        };
      } else {
        const response: SchemaResponse = await this.client.getSchema();
        
        return {
          database: response.database,
          version: response.version,
          tables: response.tables,
          views: response.views,
        };
      }
    } catch (error) {
      return {
        database: '',
        version: '',
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }
}
