use rand::Rng;

pub struct DdlGenerator {
    table_counter: usize,
}

impl DdlGenerator {
    pub fn new() -> Self {
        Self { table_counter: 0 }
    }

    pub fn generate_create_table(&mut self) -> String {
        let table_name = format!("t{}", self.table_counter);
        self.table_counter += 1;

        let col_count = rand::thread_rng().gen_range(1..=5);
        let mut columns = Vec::new();

        let types = ["INTEGER", "TEXT", "FLOAT", "DATE"];

        for i in 0..col_count {
            let col_name = format!("c{}", i);
            let data_type = types[rand::thread_rng().gen_range(0..types.len())];
            let nullable = if rand::thread_rng().gen_bool(0.3) {
                " NOT NULL"
            } else {
                ""
            };
            let primary_key = if i == 0 { " PRIMARY KEY" } else { "" };

            columns.push(format!(
                "{} {}{}{}",
                col_name, data_type, nullable, primary_key
            ));
        }

        format!("CREATE TABLE {} ({})", table_name, columns.join(", "))
    }

    pub fn generate_drop_table(&mut self) -> String {
        if self.table_counter > 0 {
            let table_idx = rand::thread_rng().gen_range(0..self.table_counter);
            format!("DROP TABLE t{}", table_idx)
        } else {
            "DROP TABLE nonexistent".to_string()
        }
    }

    pub fn get_table_count(&self) -> usize {
        self.table_counter
    }
}

impl Default for DdlGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_create_table() {
        let mut gen = DdlGenerator::new();
        let sql = gen.generate_create_table();
        assert!(sql.contains("CREATE TABLE"));
        assert!(sql.contains("t0"));
    }

    #[test]
    fn test_generate_drop_table() {
        let mut gen = DdlGenerator::new();
        gen.generate_create_table();
        gen.generate_create_table();

        let sql = gen.generate_drop_table();
        assert!(sql.contains("DROP TABLE"));
    }
}
