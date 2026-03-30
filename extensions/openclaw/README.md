# SQLRustGo OpenClaw Extension

AgentSQL Extension for OpenClaw AI Agent platform.

## Installation

```bash
npm install @sqlrustgo/openclaw-extension
```

## Quick Start

```typescript
import AgentSQLOpenClawExtension from '@sqlrustgo/openclaw-extension';

const extension = new AgentSQLOpenClawExtension('http://localhost:8080');
await extension.initialize();

console.log('Tools:', extension.tools.map(t => t.name));
```

## Tools

### agentsql_query

Execute SQL queries against the database.

```typescript
const result = await extension.executeTool('agentsql_query', {
  sql: 'SELECT * FROM users WHERE id = 1'
});
```

### agentsql_nl_query

Convert natural language to SQL.

```typescript
const result = await extension.executeTool('agentsql_nl_query', {
  query: 'show all users where active'
});
```

### agentsql_schema

Get database schema information.

```typescript
const schema = await extension.executeTool('agentsql_schema', {
  table: 'users'
});
```

### agentsql_stats

Get database statistics.

```typescript
const stats = await extension.executeTool('agentsql_stats', {});
```

### agentsql_memory

Manage agent memory for context storage.

```typescript
// Save memory
await extension.executeTool('agentsql_memory', {
  operation: 'save',
  content: 'User asked about orders',
  memory_type: 'conversation',
  agent_id: 'agent-123'
});

// Load memories
await extension.executeTool('agentsql_memory', {
  operation: 'load',
  agent_id: 'agent-123'
});

// Search memories
await extension.executeTool('agentsql_memory', {
  operation: 'search',
  query: 'orders'
});
```

## API Endpoints

The extension requires the following AgentSQL API endpoints:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Health check |
| `/query` | POST | Execute SQL |
| `/nl_query` | POST | Natural language to SQL |
| `/schema` | GET | Get full schema |
| `/schema/:table` | GET | Get table schema |
| `/stats` | GET | Get all stats |
| `/stats/:table` | GET | Get table stats |
| `/stats/queries` | GET | Get query statistics |
| `/memory/save` | POST | Save memory |
| `/memory/load` | POST | Load memories |
| `/memory/search` | POST | Search memories |
| `/memory/clear` | POST | Clear memories |
| `/memory/stats` | GET | Get memory stats |

## License

MIT
