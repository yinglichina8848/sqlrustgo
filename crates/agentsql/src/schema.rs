use serde_json::{json, Value};
use std::collections::HashMap;

pub struct SchemaService {
    tables: HashMap<String, TableSchema>,
    views: HashMap<String, String>,
}

#[derive(Clone)]
pub struct ColumnSchema {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub primary_key: bool,
    pub unique: bool,
    pub default_value: Option<String>,
    pub max_length: Option<usize>,
    pub precision: Option<usize>,
    pub scale: Option<usize>,
    pub foreign_key: Option<ForeignKeyRef>,
}

#[derive(Clone)]
pub struct ForeignKeyRef {
    pub table: String,
    pub column: String,
    pub on_delete: Option<String>,
    pub on_update: Option<String>,
}

#[derive(Clone)]
pub struct IndexSchema {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
    pub index_type: String,
}

#[derive(Clone)]
pub struct TableSchema {
    pub name: String,
    pub columns: Vec<ColumnSchema>,
    pub indexes: Vec<IndexSchema>,
    pub comment: Option<String>,
}

impl SchemaService {
    pub fn new() -> Self {
        let mut tables = HashMap::new();

        tables.insert(
            "users".to_string(),
            TableSchema {
                name: "users".to_string(),
                columns: vec![
                    ColumnSchema {
                        name: "id".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        primary_key: true,
                        unique: true,
                        default_value: None,
                        max_length: None,
                        precision: None,
                        scale: None,
                        foreign_key: None,
                    },
                    ColumnSchema {
                        name: "name".to_string(),
                        data_type: "VARCHAR".to_string(),
                        nullable: false,
                        primary_key: false,
                        unique: false,
                        default_value: None,
                        max_length: Some(255),
                        precision: None,
                        scale: None,
                        foreign_key: None,
                    },
                    ColumnSchema {
                        name: "email".to_string(),
                        data_type: "VARCHAR".to_string(),
                        nullable: true,
                        primary_key: false,
                        unique: true,
                        default_value: None,
                        max_length: Some(255),
                        precision: None,
                        scale: None,
                        foreign_key: None,
                    },
                    ColumnSchema {
                        name: "password_hash".to_string(),
                        data_type: "VARCHAR".to_string(),
                        nullable: false,
                        primary_key: false,
                        unique: false,
                        default_value: None,
                        max_length: Some(512),
                        precision: None,
                        scale: None,
                        foreign_key: None,
                    },
                    ColumnSchema {
                        name: "created_at".to_string(),
                        data_type: "TIMESTAMP".to_string(),
                        nullable: true,
                        primary_key: false,
                        unique: false,
                        default_value: Some("CURRENT_TIMESTAMP".to_string()),
                        max_length: None,
                        precision: None,
                        scale: None,
                        foreign_key: None,
                    },
                ],
                indexes: vec![IndexSchema {
                    name: "idx_users_email".to_string(),
                    columns: vec!["email".to_string()],
                    unique: true,
                    index_type: "BTREE".to_string(),
                }],
                comment: Some("User accounts table".to_string()),
            },
        );

        tables.insert(
            "products".to_string(),
            TableSchema {
                name: "products".to_string(),
                columns: vec![
                    ColumnSchema {
                        name: "id".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        primary_key: true,
                        unique: true,
                        default_value: None,
                        max_length: None,
                        precision: None,
                        scale: None,
                        foreign_key: None,
                    },
                    ColumnSchema {
                        name: "name".to_string(),
                        data_type: "VARCHAR".to_string(),
                        nullable: false,
                        primary_key: false,
                        unique: false,
                        default_value: None,
                        max_length: Some(255),
                        precision: None,
                        scale: None,
                        foreign_key: None,
                    },
                    ColumnSchema {
                        name: "description".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: true,
                        primary_key: false,
                        unique: false,
                        default_value: None,
                        max_length: None,
                        precision: None,
                        scale: None,
                        foreign_key: None,
                    },
                    ColumnSchema {
                        name: "price".to_string(),
                        data_type: "DECIMAL".to_string(),
                        nullable: false,
                        primary_key: false,
                        unique: false,
                        default_value: Some("0.00".to_string()),
                        max_length: None,
                        precision: Some(10),
                        scale: Some(2),
                        foreign_key: None,
                    },
                    ColumnSchema {
                        name: "stock".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        primary_key: false,
                        unique: false,
                        default_value: Some("0".to_string()),
                        max_length: None,
                        precision: None,
                        scale: None,
                        foreign_key: None,
                    },
                ],
                indexes: vec![],
                comment: Some("Product catalog".to_string()),
            },
        );

        tables.insert(
            "orders".to_string(),
            TableSchema {
                name: "orders".to_string(),
                columns: vec![
                    ColumnSchema {
                        name: "id".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        primary_key: true,
                        unique: true,
                        default_value: None,
                        max_length: None,
                        precision: None,
                        scale: None,
                        foreign_key: None,
                    },
                    ColumnSchema {
                        name: "user_id".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        primary_key: false,
                        unique: false,
                        default_value: None,
                        max_length: None,
                        precision: None,
                        scale: None,
                        foreign_key: Some(ForeignKeyRef {
                            table: "users".to_string(),
                            column: "id".to_string(),
                            on_delete: Some("CASCADE".to_string()),
                            on_update: Some("CASCADE".to_string()),
                        }),
                    },
                    ColumnSchema {
                        name: "product_id".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        primary_key: false,
                        unique: false,
                        default_value: None,
                        max_length: None,
                        precision: None,
                        scale: None,
                        foreign_key: Some(ForeignKeyRef {
                            table: "products".to_string(),
                            column: "id".to_string(),
                            on_delete: Some("RESTRICT".to_string()),
                            on_update: Some("CASCADE".to_string()),
                        }),
                    },
                    ColumnSchema {
                        name: "quantity".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        primary_key: false,
                        unique: false,
                        default_value: Some("1".to_string()),
                        max_length: None,
                        precision: None,
                        scale: None,
                        foreign_key: None,
                    },
                    ColumnSchema {
                        name: "total_price".to_string(),
                        data_type: "DECIMAL".to_string(),
                        nullable: false,
                        primary_key: false,
                        unique: false,
                        default_value: None,
                        max_length: None,
                        precision: Some(10),
                        scale: Some(2),
                        foreign_key: None,
                    },
                    ColumnSchema {
                        name: "status".to_string(),
                        data_type: "VARCHAR".to_string(),
                        nullable: false,
                        primary_key: false,
                        unique: false,
                        default_value: Some("'pending'".to_string()),
                        max_length: Some(20),
                        precision: None,
                        scale: None,
                        foreign_key: None,
                    },
                    ColumnSchema {
                        name: "created_at".to_string(),
                        data_type: "TIMESTAMP".to_string(),
                        nullable: true,
                        primary_key: false,
                        unique: false,
                        default_value: Some("CURRENT_TIMESTAMP".to_string()),
                        max_length: None,
                        precision: None,
                        scale: None,
                        foreign_key: None,
                    },
                ],
                indexes: vec![
                    IndexSchema {
                        name: "idx_orders_user_id".to_string(),
                        columns: vec!["user_id".to_string()],
                        unique: false,
                        index_type: "BTREE".to_string(),
                    },
                    IndexSchema {
                        name: "idx_orders_status".to_string(),
                        columns: vec!["status".to_string()],
                        unique: false,
                        index_type: "BTREE".to_string(),
                    },
                ],
                comment: Some("Order records".to_string()),
            },
        );

        let mut views = HashMap::new();
        views.insert("user_orders".to_string(), 
            "SELECT users.name AS user_name, users.email, orders.id AS order_id, orders.total_price, orders.status, orders.created_at FROM users JOIN orders ON users.id = orders.user_id".to_string());
        views.insert("order_details".to_string(),
            "SELECT orders.id, products.name AS product_name, products.price, orders.quantity, orders.total_price FROM orders JOIN products ON orders.product_id = products.id".to_string());

        Self { tables, views }
    }

    pub fn get_schema(&self) -> Value {
        let tables: Vec<Value> = self
            .tables
            .values()
            .map(|t| {
                let columns: Vec<Value> = t
                    .columns
                    .iter()
                    .map(|c| {
                        let mut obj = json!({
                            "name": c.name,
                            "type": c.data_type,
                            "nullable": c.nullable,
                            "primary_key": c.primary_key,
                            "unique": c.unique,
                        });
                        if let Some(ref default) = c.default_value {
                            obj["default"] = json!(default);
                        }
                        if let Some(len) = c.max_length {
                            obj["max_length"] = json!(len);
                        }
                        if let Some(prec) = c.precision {
                            obj["precision"] = json!(prec);
                        }
                        if let Some(scale) = c.scale {
                            obj["scale"] = json!(scale);
                        }
                        if let Some(ref fk) = c.foreign_key {
                            obj["foreign_key"] = json!({
                                "table": fk.table,
                                "column": fk.column,
                            });
                        }
                        obj
                    })
                    .collect();

                let indexes: Vec<Value> = t
                    .indexes
                    .iter()
                    .map(|i| {
                        json!({
                            "name": i.name,
                            "columns": i.columns,
                            "unique": i.unique,
                            "type": i.index_type,
                        })
                    })
                    .collect();

                let mut table_obj = json!({
                    "name": t.name,
                    "columns": columns,
                    "indexes": indexes,
                });
                if let Some(ref comment) = t.comment {
                    table_obj["comment"] = json!(comment);
                }
                table_obj
            })
            .collect();

        let views: Vec<Value> = self
            .views
            .iter()
            .map(|(name, def)| {
                json!({
                    "name": name,
                    "definition": def,
                })
            })
            .collect();

        json!({
            "database": "sqlrustgo",
            "version": "1.6.1",
            "tables": tables,
            "views": views,
        })
    }

    pub fn get_table_schema(&self, table_name: &str) -> Option<Value> {
        self.tables.get(table_name).map(|t| {
            let columns: Vec<Value> = t
                .columns
                .iter()
                .map(|c| {
                    json!({
                        "name": c.name,
                        "type": c.data_type,
                        "nullable": c.nullable,
                        "primary_key": c.primary_key,
                        "unique": c.unique,
                    })
                })
                .collect();

            let indexes: Vec<Value> = t
                .indexes
                .iter()
                .map(|i| {
                    json!({
                        "name": i.name,
                        "columns": i.columns,
                        "unique": i.unique,
                        "type": i.index_type,
                    })
                })
                .collect();

            json!({
                "name": t.name,
                "columns": columns,
                "indexes": indexes,
            })
        })
    }

    pub fn list_tables(&self) -> Vec<String> {
        self.tables.keys().cloned().collect()
    }

    pub fn list_views(&self) -> Vec<String> {
        self.views.keys().cloned().collect()
    }

    pub fn get_view_definition(&self, view_name: &str) -> Option<String> {
        self.views.get(view_name).cloned()
    }
}

impl Default for SchemaService {
    fn default() -> Self {
        Self::new()
    }
}
