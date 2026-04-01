import { AgentSQLClient } from './client';
import { QueryTool } from './tools/query';
import { NlQueryTool } from './tools/nl_query';
import { SchemaTool } from './tools/schema';
import { StatsTool } from './tools/stats';
import { MemoryTool } from './tools/memory';

export interface OpenClawTool {
  name: string;
  description: string;
  inputSchema: Record<string, unknown>;
  execute: (input: Record<string, unknown>, context?: Record<string, unknown>) => Promise<unknown>;
}

export interface OpenClawSkill {
  name: string;
  version: string;
  description: string;
  tools: OpenClawTool[];
  endpoints: {
    query: string;
    nl_query: string;
    schema: string;
    stats: string;
    memory: string;
  };
}

export class AgentSQLOpenClawExtension implements OpenClawSkill {
  name = 'agentsql';
  version = '2.1.0';
  description = 'SQLRustGo AgentSQL Extension - AI Agent database access layer';

  private client: AgentSQLClient;
  
  tools: OpenClawTool[];
  endpoints: {
    query: string;
    nl_query: string;
    schema: string;
    stats: string;
    memory: string;
  };

  constructor(baseUrl: string = 'http://localhost:8080') {
    this.client = new AgentSQLClient(baseUrl);
    
    this.endpoints = {
      query: `${baseUrl}/query`,
      nl_query: `${baseUrl}/nl_query`,
      schema: `${baseUrl}/schema`,
      stats: `${baseUrl}/stats`,
      memory: `${baseUrl}/memory`,
    };

    const queryTool = new QueryTool(this.client);
    const nlQueryTool = new NlQueryTool(this.client);
    const schemaTool = new SchemaTool(this.client);
    const statsTool = new StatsTool(this.client);
    const memoryTool = new MemoryTool(this.client);

    this.tools = [
      queryTool.getToolDefinition(),
      nlQueryTool.getToolDefinition(),
      schemaTool.getToolDefinition(),
      statsTool.getToolDefinition(),
      memoryTool.getToolDefinition(),
    ];
  }

  async initialize(): Promise<void> {
    await this.client.health();
  }

  getToolByName(name: string): OpenClawTool | undefined {
    return this.tools.find(t => t.name === name);
  }

  async executeTool(
    toolName: string, 
    input: Record<string, unknown>,
    context?: Record<string, unknown>
  ): Promise<unknown> {
    const tool = this.getToolByName(toolName);
    if (!tool) {
      throw new Error(`Tool not found: ${toolName}`);
    }
    return tool.execute(input, context);
  }
}

export default AgentSQLOpenClawExtension;

export { AgentSQLClient };
export { QueryTool, QueryInput, QueryOutput } from './tools/query';
export { NlQueryTool, NlQueryInput, NlQueryOutput } from './tools/nl_query';
export { SchemaTool, SchemaInput, SchemaOutput } from './tools/schema';
export { StatsTool, StatsInput, StatsOutput } from './tools/stats';
export { MemoryTool, MemoryInput, MemoryOutput } from './tools/memory';
