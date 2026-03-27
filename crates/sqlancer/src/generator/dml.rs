use rand::Rng;

pub struct DmlGenerator {
    table_count: usize,
}

impl DmlGenerator {
    pub fn new(table_count: usize) -> Self {
        Self { table_count }
    }

    pub fn update_table_count(&mut self, count: usize) {
        self.table_count = count;
    }

    pub fn generate_insert(&self) -> Option<String> {
        if self.table_count == 0 {
            return None;
        }

        let table_idx = rand::thread_rng().gen_range(0..self.table_count);
        let table_name = format!("t{}", table_idx);

        let col_count = rand::thread_rng().gen_range(1..=4);
        let mut values = Vec::new();

        for _ in 0..col_count {
            let value = match rand::thread_rng().gen_range(0..4) {
                0 => format!("{}", rand::thread_rng().gen_range(1..1000)),
                1 => format!("'{}'", self.random_string(5)),
                2 => format!("{}", rand::thread_rng().gen::<f64>() * 100.0),
                _ => "NULL".to_string(),
            };
            values.push(value);
        }

        Some(format!(
            "INSERT INTO {} VALUES ({})",
            table_name,
            values.join(", ")
        ))
    }

    pub fn generate_select(&self) -> Option<String> {
        if self.table_count == 0 {
            return None;
        }

        let table_idx = rand::thread_rng().gen_range(0..self.table_count);
        let table_name = format!("t{}", table_idx);

        let has_where = rand::thread_rng().gen_bool(0.5);
        let has_order = rand::thread_rng().gen_bool(0.3);

        let mut sql = String::from("SELECT * FROM ");
        sql.push_str(&table_name);

        if has_where {
            let col_idx = rand::thread_rng().gen_range(0..4);
            let op = match rand::thread_rng().gen_range(0..4) {
                0 => "=",
                1 => ">",
                2 => "<",
                _ => ">=",
            };
            let value = rand::thread_rng().gen_range(1..100);
            sql.push_str(&format!(" WHERE c{} {} {}", col_idx, op, value));
        }

        if has_order {
            let col_idx = rand::thread_rng().gen_range(0..4);
            let dir = if rand::thread_rng().gen_bool(0.5) {
                "ASC"
            } else {
                "DESC"
            };
            sql.push_str(&format!(" ORDER BY c{} {}", col_idx, dir));
        }

        Some(sql)
    }

    pub fn generate_update(&self) -> Option<String> {
        if self.table_count == 0 {
            return None;
        }

        let table_idx = rand::thread_rng().gen_range(0..self.table_count);
        let table_name = format!("t{}", table_idx);

        let col_idx = rand::thread_rng().gen_range(0..4);
        let value = rand::thread_rng().gen_range(1..1000);

        let mut sql = format!("UPDATE {} SET c{} = {}", table_name, col_idx, value);

        if rand::thread_rng().gen_bool(0.5) {
            let where_col = rand::thread_rng().gen_range(0..4);
            let where_val = rand::thread_rng().gen_range(1..100);
            sql.push_str(&format!(" WHERE c{} = {}", where_col, where_val));
        }

        Some(sql)
    }

    pub fn generate_delete(&self) -> Option<String> {
        if self.table_count == 0 {
            return None;
        }

        let table_idx = rand::thread_rng().gen_range(0..self.table_count);
        let table_name = format!("t{}", table_idx);

        let mut sql = String::from("DELETE FROM ");
        sql.push_str(&table_name);

        if rand::thread_rng().gen_bool(0.5) {
            let col_idx = rand::thread_rng().gen_range(0..4);
            let value = rand::thread_rng().gen_range(1..100);
            sql.push_str(&format!(" WHERE c{} = {}", col_idx, value));
        }

        Some(sql)
    }

    fn random_string(&self, len: usize) -> String {
        let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyz".chars().collect();
        (0..len)
            .map(|_| chars[rand::thread_rng().gen_range(0..chars.len())])
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_insert() {
        let gen = DmlGenerator::new(1);
        let sql = gen.generate_insert();
        assert!(sql.is_some());
        assert!(sql.unwrap().contains("INSERT INTO"));
    }

    #[test]
    fn test_generate_select() {
        let gen = DmlGenerator::new(1);
        let sql = gen.generate_select();
        assert!(sql.is_some());
        assert!(sql.unwrap().contains("SELECT * FROM"));
    }

    #[test]
    fn test_no_tables() {
        let gen = DmlGenerator::new(0);
        assert!(gen.generate_insert().is_none());
        assert!(gen.generate_select().is_none());
    }
}
