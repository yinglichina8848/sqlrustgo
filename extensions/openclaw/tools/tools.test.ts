import { QueryTool } from './query';
import { NlQueryTool } from './nl_query';
import { SchemaTool } from './schema';
import { StatsTool } from './stats';
import { MemoryTool } from './memory';
import { AgentSQLClient } from '../client';

const mockFetch = jest.fn();

global.fetch = mockFetch;

describe('QueryTool', () => {
  let tool: QueryTool;
  let mockClient: jest.Mocked<AgentSQLClient>;

  beforeEach(() => {
    mockClient = {
      query: jest.fn(),
    } as any;
    tool = new QueryTool(mockClient);
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  test('getToolDefinition returns correct structure', () => {
    const def = tool.getToolDefinition();
    expect(def.name).toBe('agentsql_query');
    expect(def.description).toBeDefined();
    expect(def.inputSchema.type).toBe('object');
    expect(def.inputSchema.properties.sql).toBeDefined();
  });

  test('execute returns query results', async () => {
    mockClient.query.mockResolvedValue({
      success: true,
      data: [[1, 'test']],
      execution_time_ms: 5,
    });

    const result = await tool.execute({ sql: 'SELECT * FROM users' });
    
    expect(result.success).toBe(true);
    expect(result.data).toEqual([[1, 'test']]);
    expect(result.row_count).toBe(1);
    expect(mockClient.query).toHaveBeenCalledWith('SELECT * FROM users');
  });

  test('execute handles errors', async () => {
    mockClient.query.mockRejectedValue(new Error('Network error'));

    const result = await tool.execute({ sql: 'SELECT * FROM users' });
    
    expect(result.success).toBe(false);
    expect(result.error).toBe('Network error');
  });
});

describe('NlQueryTool', () => {
  let tool: NlQueryTool;
  let mockClient: jest.Mocked<AgentSQLClient>;

  beforeEach(() => {
    mockClient = {
      nlQuery: jest.fn(),
    } as any;
    tool = new NlQueryTool(mockClient);
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  test('getToolDefinition returns correct structure', () => {
    const def = tool.getToolDefinition();
    expect(def.name).toBe('agentsql_nl_query');
    expect(def.inputSchema.properties.query).toBeDefined();
    expect(def.inputSchema.properties.context).toBeDefined();
  });

  test('execute returns SQL conversion', async () => {
    mockClient.nlQuery.mockResolvedValue({
      success: true,
      sql: 'SELECT * FROM users',
      confidence: 0.9,
      table_hint: 'users',
    });

    const result = await tool.execute({ query: 'show all users' });
    
    expect(result.success).toBe(true);
    expect(result.sql).toBe('SELECT * FROM users');
    expect(result.confidence).toBe(0.9);
    expect(result.table_hint).toBe('users');
  });

  test('execute handles errors', async () => {
    mockClient.nlQuery.mockRejectedValue(new Error('Conversion failed'));

    const result = await tool.execute({ query: 'invalid query' });
    
    expect(result.success).toBe(false);
    expect(result.error).toBe('Conversion failed');
  });
});

describe('SchemaTool', () => {
  let tool: SchemaTool;
  let mockClient: jest.Mocked<AgentSQLClient>;

  beforeEach(() => {
    mockClient = {
      getSchema: jest.fn(),
      getTableSchema: jest.fn(),
    } as any;
    tool = new SchemaTool(mockClient);
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  test('getToolDefinition returns correct structure', () => {
    const def = tool.getToolDefinition();
    expect(def.name).toBe('agentsql_schema');
    expect(def.inputSchema.properties.table).toBeDefined();
  });

  test('execute returns full schema when no table specified', async () => {
    const mockSchema = {
      database: 'sqlrustgo',
      version: '2.1.0',
      tables: [{ name: 'users', columns: [] }],
      views: [],
    };
    mockClient.getSchema.mockResolvedValue(mockSchema);

    const result = await tool.execute({});
    
    expect(result.database).toBe('sqlrustgo');
    expect(result.tables).toHaveLength(1);
    expect(result.tables?.[0].name).toBe('users');
  });

  test('execute returns specific table schema', async () => {
    mockClient.getTableSchema.mockResolvedValue({
      name: 'users',
      columns: [
        { name: 'id', type: 'INTEGER', nullable: false, primary_key: true, unique: true },
      ],
    });

    const result = await tool.execute({ table: 'users' });
    
    expect(result.tables?.[0].name).toBe('users');
    expect(result.tables?.[0].columns).toHaveLength(1);
  });

  test('execute handles errors', async () => {
    mockClient.getSchema.mockRejectedValue(new Error('Schema fetch failed'));

    const result = await tool.execute({});
    
    expect(result.error).toBe('Schema fetch failed');
  });
});

describe('StatsTool', () => {
  let tool: StatsTool;
  let mockClient: jest.Mocked<AgentSQLClient>;

  beforeEach(() => {
    mockClient = {
      getStats: jest.fn(),
      getTableStats: jest.fn(),
    } as any;
    tool = new StatsTool(mockClient);
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  test('getToolDefinition returns correct structure', () => {
    const def = tool.getToolDefinition();
    expect(def.name).toBe('agentsql_stats');
  });

  test('execute returns overall stats', async () => {
    const mockStats = {
      tables: { users: { row_count: 100, size_bytes: 4096, indexes_count: 1 } },
      query_statistics: { total_queries: 50 },
    };
    mockClient.getStats.mockResolvedValue(mockStats);

    const result = await tool.execute({});
    
    expect(result.tables).toBeDefined();
    expect(result.query_statistics).toBeDefined();
  });

  test('execute handles errors', async () => {
    mockClient.getStats.mockRejectedValue(new Error('Stats fetch failed'));

    const result = await tool.execute({});
    
    expect(result.error).toBe('Stats fetch failed');
  });
});

describe('MemoryTool', () => {
  let tool: MemoryTool;
  let mockClient: jest.Mocked<AgentSQLClient>;

  beforeEach(() => {
    mockClient = {
      saveMemory: jest.fn(),
      loadMemory: jest.fn(),
      searchMemory: jest.fn(),
      clearMemory: jest.fn(),
      getMemoryStats: jest.fn(),
    } as any;
    tool = new MemoryTool(mockClient);
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  test('getToolDefinition returns correct structure', () => {
    const def = tool.getToolDefinition();
    expect(def.name).toBe('agentsql_memory');
    expect(def.inputSchema.properties.operation).toBeDefined();
  });

  test('execute save operation', async () => {
    mockClient.saveMemory.mockResolvedValue({
      id: 'mem_123',
      success: true,
      message: 'Memory saved',
    });

    const result = await tool.execute({
      operation: 'save',
      content: 'Test memory',
    });
    
    expect(result.success).toBe(true);
    expect(result.id).toBe('mem_123');
  });

  test('execute load operation', async () => {
    const mockMemories = [{
      id: 'mem_123',
      content: 'Test',
      memory_type: 'conversation',
      timestamp: 1234567890,
      tags: [],
      importance: 0.5,
      metadata: {},
    }];
    mockClient.loadMemory.mockResolvedValue({
      memories: mockMemories,
      total: 1,
    });

    const result = await tool.execute({
      operation: 'load',
      agent_id: 'agent-1',
    });
    
    expect(result.success).toBe(true);
    expect(result.memories).toHaveLength(1);
    expect(result.total).toBe(1);
  });

  test('execute search operation', async () => {
    mockClient.searchMemory.mockResolvedValue({
      results: [],
      total: 0,
      scores: [],
    });

    const result = await tool.execute({
      operation: 'search',
      query: 'test query',
    });
    
    expect(result.success).toBe(true);
  });

  test('execute clear operation', async () => {
    mockClient.clearMemory.mockResolvedValue({
      cleared: 5,
      success: true,
    });

    const result = await tool.execute({
      operation: 'clear',
      agent_id: 'agent-1',
    });
    
    expect(result.success).toBe(true);
    expect(result.cleared).toBe(5);
  });

  test('execute stats operation', async () => {
    mockClient.getMemoryStats.mockResolvedValue({
      total_memories: 10,
      by_type: { conversation: 5, query: 5 },
      unique_agents: 2,
      unique_sessions: 3,
      unique_tags: 4,
    });

    const result = await tool.execute({
      operation: 'stats',
    });
    
    expect(result.total_memories).toBe(10);
    expect(result.unique_agents).toBe(2);
  });

  test('execute requires operation', async () => {
    const result = await tool.execute({});
    
    expect(result.success).toBe(false);
  });
});
