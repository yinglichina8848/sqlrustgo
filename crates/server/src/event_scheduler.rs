//! Event Scheduler Background Service
//!
//! This module provides background scheduling for database events.
//! Events are checked and executed periodically according to their schedule.

use sqlrustgo_executor::event::EventExecutor;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time;

#[derive(Debug, Clone)]
pub enum EventSchedulerCommand {
    Stop,
}

pub struct EventSchedulerService {
    executor: Arc<EventExecutor>,
    interval: Duration,
}

impl EventSchedulerService {
    pub fn new(executor: Arc<EventExecutor>, interval_secs: u64) -> Self {
        Self {
            executor,
            interval: Duration::from_secs(interval_secs),
        }
    }

    pub async fn run(&self, mut rx: mpsc::Receiver<EventSchedulerCommand>) {
        let mut ticker = time::interval(self.interval);

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    if let Err(e) = self.check_and_execute_events() {
                        log::error!("Event scheduler error: {}", e);
                    }
                }
                Some(cmd) = rx.recv() => {
                    match cmd {
                        EventSchedulerCommand::Stop => {
                            log::info!("Event scheduler received stop command");
                            break;
                        }
                    }
                }
            }
        }
    }

    fn check_and_execute_events(&self) -> Result<(), String> {
        let results = self.executor.run_due_events();
        for result in results {
            if let Err(e) = result {
                log::error!("Event execution failed: {}", e);
            }
        }
        Ok(())
    }
}