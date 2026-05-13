#![allow(deprecated)]

//! Event executor tests

use sqlrustgo_catalog::{Catalog, Event, EventSchedule};
use sqlrustgo_executor::event::EventExecutor;
use sqlrustgo_types::SqlResult;
use std::sync::{Arc, RwLock};

fn create_catalog() -> Arc<RwLock<Catalog>> {
    Arc::new(RwLock::new(Catalog::new("test_db".to_string())))
}

fn create_enabled_event(name: &str, body: &str, schedule: EventSchedule) -> Event {
    Event {
        name: name.to_string(),
        schema: "public".to_string(),
        schedule,
        body: body.to_string(),
        enable: true,
        comment: None,
        created: "2024-01-01 00:00:00".to_string(),
        last_altered: "2024-01-01 00:00:00".to_string(),
        definer: "root@localhost".to_string(),
        sql_mode: "ONLY_FULL_GROUP_BY,STRICT_TRANS_TABLES".to_string(),
        status: "ENABLED".to_string(),
        on_completion: "PRESERVE".to_string(),
        starts: None,
        ends: None,
    }
}

fn create_disabled_event(name: &str, body: &str, schedule: EventSchedule) -> Event {
    Event {
        name: name.to_string(),
        schema: "public".to_string(),
        schedule,
        body: body.to_string(),
        enable: false,
        comment: None,
        created: "2024-01-01 00:00:00".to_string(),
        last_altered: "2024-01-01 00:00:00".to_string(),
        definer: "root@localhost".to_string(),
        sql_mode: "ONLY_FULL_GROUP_BY,STRICT_TRANS_TABLES".to_string(),
        status: "DISABLED".to_string(),
        on_completion: "PRESERVE".to_string(),
        starts: None,
        ends: None,
    }
}

mod should_run_event_tests {
    use super::*;

    #[test]
    fn test_should_run_event_enabled_one_time() {
        let event = create_enabled_event("test", "SELECT 1", EventSchedule::OneTime);
        assert!(EventExecutor::should_run_event(&event));
    }

    #[test]
    fn test_should_run_event_enabled_interval() {
        let event = create_enabled_event(
            "test",
            "SELECT 1",
            EventSchedule::Interval {
                interval_value: "5".to_string(),
                interval_unit: "SECOND".to_string(),
            },
        );
        assert!(EventExecutor::should_run_event(&event));
    }

    #[test]
    fn test_should_run_event_disabled_one_time() {
        let event = create_disabled_event("test", "SELECT 1", EventSchedule::OneTime);
        assert!(!EventExecutor::should_run_event(&event));
    }

    #[test]
    fn test_should_run_event_disabled_interval() {
        let event = create_disabled_event(
            "test",
            "SELECT 1",
            EventSchedule::Interval {
                interval_value: "5".to_string(),
                interval_unit: "SECOND".to_string(),
            },
        );
        assert!(!EventExecutor::should_run_event(&event));
    }
}

mod execute_event_tests {
    use super::*;

    #[test]
    fn test_execute_event_valid_sql() {
        let catalog = create_catalog();
        let executor = EventExecutor::new(catalog);

        let event = create_enabled_event("test", "SELECT 1", EventSchedule::OneTime);
        let result: SqlResult<()> = executor.execute_event(&event);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_event_disabled_event() {
        let catalog = create_catalog();
        let executor = EventExecutor::new(catalog);

        let event = create_disabled_event("test", "SELECT 1", EventSchedule::OneTime);
        let result: SqlResult<()> = executor.execute_event(&event);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_event_invalid_sql() {
        let catalog = create_catalog();
        let executor = EventExecutor::new(catalog);

        let event = create_enabled_event("test", "INVALID SQL SYNTAX", EventSchedule::OneTime);
        let result: SqlResult<()> = executor.execute_event(&event);
        assert!(result.is_err());
    }
}

mod get_due_events_tests {
    use super::*;

    #[test]
    fn test_get_due_events_empty() {
        let catalog = create_catalog();
        let executor = EventExecutor::new(catalog);
        let due = executor.get_due_events();
        assert!(due.is_empty());
    }

    #[test]
    fn test_get_due_events_with_enabled_events() {
        let catalog = create_catalog();

        {
            let mut cat = catalog.write().unwrap();
            cat.add_event(create_enabled_event(
                "event1",
                "SELECT 1",
                EventSchedule::OneTime,
            ))
            .unwrap();
            cat.add_event(create_enabled_event(
                "event2",
                "SELECT 2",
                EventSchedule::Interval {
                    interval_value: "5".to_string(),
                    interval_unit: "MINUTE".to_string(),
                },
            ))
            .unwrap();
        }

        let executor = EventExecutor::new(catalog);
        let due = executor.get_due_events();
        assert_eq!(due.len(), 2);
    }

    #[test]
    fn test_get_due_events_with_disabled_events() {
        let catalog = create_catalog();

        {
            let mut cat = catalog.write().unwrap();
            cat.add_event(create_enabled_event(
                "event1",
                "SELECT 1",
                EventSchedule::OneTime,
            ))
            .unwrap();
            cat.add_event(create_disabled_event(
                "event2",
                "SELECT 2",
                EventSchedule::OneTime,
            ))
            .unwrap();
        }

        let executor = EventExecutor::new(catalog);
        let due = executor.get_due_events();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].name, "event1");
    }
}

mod has_event_tests {
    use super::*;

    #[test]
    fn test_has_event_exists() {
        let catalog = create_catalog();

        {
            let mut cat = catalog.write().unwrap();
            cat.add_event(create_enabled_event(
                "myevent",
                "SELECT 1",
                EventSchedule::OneTime,
            ))
            .unwrap();
        }

        let executor = EventExecutor::new(catalog);
        assert!(executor.has_event("myevent"));
    }

    #[test]
    fn test_has_event_not_exists() {
        let catalog = create_catalog();
        let executor = EventExecutor::new(catalog);
        assert!(!executor.has_event("nonexistent"));
    }
}

mod get_event_tests {
    use super::*;

    #[test]
    fn test_get_event_exists() {
        let catalog = create_catalog();

        {
            let mut cat = catalog.write().unwrap();
            cat.add_event(create_enabled_event(
                "myevent",
                "SELECT 1",
                EventSchedule::OneTime,
            ))
            .unwrap();
        }

        let executor = EventExecutor::new(catalog);
        let event = executor.get_event("myevent");
        assert!(event.is_some());
        assert_eq!(event.unwrap().name, "myevent");
    }

    #[test]
    fn test_get_event_not_exists() {
        let catalog = create_catalog();
        let executor = EventExecutor::new(catalog);
        let event = executor.get_event("nonexistent");
        assert!(event.is_none());
    }
}

mod run_due_events_tests {
    use super::*;

    #[test]
    fn test_run_due_events_empty() {
        let catalog = create_catalog();
        let executor = EventExecutor::new(catalog);
        let results = executor.run_due_events();
        assert!(results.is_empty());
    }

    #[test]
    fn test_run_due_events_single_event() {
        let catalog = create_catalog();

        {
            let mut cat = catalog.write().unwrap();
            cat.add_event(create_enabled_event(
                "event1",
                "SELECT 1",
                EventSchedule::OneTime,
            ))
            .unwrap();
        }

        let executor = EventExecutor::new(catalog);
        let results = executor.run_due_events();
        assert_eq!(results.len(), 1);
        assert!(results[0].is_ok());
    }

    #[test]
    fn test_run_due_events_multiple_events() {
        let catalog = create_catalog();

        {
            let mut cat = catalog.write().unwrap();
            cat.add_event(create_enabled_event(
                "event1",
                "SELECT 1",
                EventSchedule::OneTime,
            ))
            .unwrap();
            cat.add_event(create_enabled_event(
                "event2",
                "SELECT 2",
                EventSchedule::Interval {
                    interval_value: "5".to_string(),
                    interval_unit: "MINUTE".to_string(),
                },
            ))
            .unwrap();
        }

        let executor = EventExecutor::new(catalog);
        let results = executor.run_due_events();
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r: &Result<(), String>| r.is_ok()));
    }

    #[test]
    fn test_run_due_events_disabled_not_run() {
        let catalog = create_catalog();

        {
            let mut cat = catalog.write().unwrap();
            cat.add_event(create_enabled_event(
                "event1",
                "SELECT 1",
                EventSchedule::OneTime,
            ))
            .unwrap();
            cat.add_event(create_disabled_event(
                "event2",
                "SELECT 2",
                EventSchedule::OneTime,
            ))
            .unwrap();
        }

        let executor = EventExecutor::new(catalog);
        let results = executor.run_due_events();
        assert_eq!(results.len(), 1);
    }
}
