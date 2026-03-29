use serde_json::{json, Value};

pub struct SchemaService {
    initialized: bool,
}

impl SchemaService {
    pub fn new() -> Self {
        Self { initialized: true }
    }

    pub fn get_schema(&self) -> Value {
        json!({
            "database": "sqlrustgo",
            "version": "1.6.1",
            "tables": [
                {
                    "name": "users",
                    "columns": [
                        {"name": "id", "type": "INTEGER", "nullable": false, "primary_key": true},
                        {"name": "name", "type": "VARCHAR", "nullable": false, "max_length": 255},
                        {"name": "email", "type": "VARCHAR", "nullable": true, "max_length": 255},
                        {"name": "created_at", "type": "TIMESTAMP", "nullable": true}
                    ],
                    "indexes": [
                        {"name": "idx_users_email", "columns": ["email"], "unique": false}
                    ]
                },
                {
                    "name": "products",
                    "columns": [
                        {"name": "id", "type": "INTEGER", "nullable": false, "primary_key": true},
                        {"name": "name", "type": "VARCHAR", "nullable": false, "max_length": 255},
                        {"name": "price", "type": "DECIMAL", "nullable": false, "precision": 10, "scale": 2},
                        {"name": "stock", "type": "INTEGER", "nullable": false, "default": 0}
                    ],
                    "indexes": []
                },
                {
                    "name": "orders",
                    "columns": [
                        {"name": "id", "type": "INTEGER", "nullable": false, "primary_key": true},
                        {"name": "user_id", "type": "INTEGER", "nullable": false, "foreign_key": "users.id"},
                        {"name": "product_id", "type": "INTEGER", "nullable": false, "foreign_key": "products.id"},
                        {"name": "quantity", "type": "INTEGER", "nullable": false, "default": 1},
                        {"name": "total_price", "type": "DECIMAL", "nullable": false, "precision": 10, "scale": 2},
                        {"name": "created_at", "type": "TIMESTAMP", "nullable": true}
                    ],
                    "indexes": [
                        {"name": "idx_orders_user_id", "columns": ["user_id"], "unique": false}
                    ]
                }
            ],
            "views": [
                {
                    "name": "user_orders",
                    "definition": "SELECT users.name, orders.id, orders.total_price FROM users JOIN orders ON users.id = orders.user_id"
                }
            ],
            "procedures": []
        })
    }

    pub fn get_table_schema(&self, table_name: &str) -> Option<Value> {
        let schema = self.get_schema();
        schema["tables"]
            .as_array()
            .and_then(|tables| tables.iter().find(|t| t["name"] == table_name).cloned())
    }

    pub fn list_tables(&self) -> Vec<String> {
        let schema = self.get_schema();
        schema["tables"]
            .as_array()
            .map(|tables| {
                tables
                    .iter()
                    .map(|t| t["name"].as_str().unwrap_or("").to_string())
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl Default for SchemaService {
    fn default() -> Self {
        Self::new()
    }
}
