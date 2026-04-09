use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct QueryStats {
    pub query_id: Uuid,
    pub total_time_ms: u64,
    pub sql_time_ms: u64,
    pub vector_time_ms: u64,
    pub graph_time_ms: u64,
    pub sql_hit: bool,
    pub vector_hit: bool,
    pub graph_hit: bool,
    pub cache_hit: bool,
}

impl Default for QueryStats {
    fn default() -> Self {
        Self {
            query_id: Uuid::new_v4(),
            total_time_ms: 0,
            sql_time_ms: 0,
            vector_time_ms: 0,
            graph_time_ms: 0,
            sql_hit: false,
            vector_hit: false,
            graph_hit: false,
            cache_hit: false,
        }
    }
}

pub struct QueryStatsBuilder {
    stats: QueryStats,
}

impl QueryStatsBuilder {
    pub fn new() -> Self {
        Self {
            stats: QueryStats::default(),
        }
    }

    pub fn with_sql_time(mut self, ms: u64) -> Self {
        self.stats.sql_time_ms = ms;
        self
    }

    pub fn with_vector_time(mut self, ms: u64) -> Self {
        self.stats.vector_time_ms = ms;
        self
    }

    pub fn with_graph_time(mut self, ms: u64) -> Self {
        self.stats.graph_time_ms = ms;
        self
    }

    pub fn with_cache_hit(mut self, hit: bool) -> Self {
        self.stats.cache_hit = hit;
        self
    }

    pub fn build(mut self) -> QueryStats {
        self.stats.total_time_ms =
            self.stats.sql_time_ms + self.stats.vector_time_ms + self.stats.graph_time_ms;
        self.stats
    }
}

impl Default for QueryStatsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_stats_default() {
        let stats = QueryStats::default();
        assert!(!stats.cache_hit);
        assert_eq!(stats.total_time_ms, 0);
    }

    #[test]
    fn test_query_stats_builder() {
        let stats = QueryStatsBuilder::new()
            .with_sql_time(10)
            .with_vector_time(5)
            .with_graph_time(3)
            .with_cache_hit(true)
            .build();

        assert_eq!(stats.sql_time_ms, 10);
        assert_eq!(stats.vector_time_ms, 5);
        assert_eq!(stats.graph_time_ms, 3);
        assert!(stats.cache_hit);
    }
}
