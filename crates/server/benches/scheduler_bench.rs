//! Scheduler Benchmarks
//!
//! Performance benchmarks for the scheduler.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use sqlrustgo_server::scheduler::{
    Agent, AgentStatus, ExecutionRecord, SchedulerState, Task, TaskPriority, Workflow, WorkflowStep,
};

/// Create a fresh scheduler state for benchmarking
fn create_benchmark_state() -> SchedulerState {
    SchedulerState::new()
}

/// Create a sample task
fn create_sample_task(i: usize) -> Task {
    let mut task = Task::new(
        format!("task_{}", i),
        serde_json::json!({
            "index": i,
            "data": format!("value_{}", i),
            "nested": {"key": "val"}
        }),
    );
    task.priority = match i % 4 {
        0 => TaskPriority::Low,
        1 => TaskPriority::Medium,
        2 => TaskPriority::High,
        _ => TaskPriority::Critical,
    };
    task
}

/// Create a sample workflow
fn create_sample_workflow(i: usize) -> Workflow {
    let steps = vec![
        WorkflowStep::Task {
            task_template: format!("template_{}", i),
            params: serde_json::json!({"index": i}),
        },
        WorkflowStep::Task {
            task_template: format!("template_{}_2", i),
            params: serde_json::json!({"index": i * 2}),
        },
    ];
    Workflow::new(format!("workflow_{}", i), steps)
}

/// Create a sample agent
fn create_sample_agent(i: usize) -> Agent {
    Agent {
        id: format!("agent_{}", i),
        name: format!("Agent {}", i),
        status: if i % 3 == 0 {
            AgentStatus::Idle
        } else if i % 3 == 1 {
            AgentStatus::Busy
        } else {
            AgentStatus::Offline
        },
        capabilities: vec!["sql".to_string(), "rag".to_string(), "vector".to_string()],
        current_task_id: if i % 2 == 0 {
            Some(format!("task_{}", i))
        } else {
            None
        },
        tasks_completed: (i * 10) as u64,
        created_at: 1000,
    }
}

// ============================================================================
// Task Benchmarks
// ============================================================================

fn create_task_benchmark(c: &mut criterion::Criterion) {
    c.bench_function("scheduler_create_task", |b| {
        let state = create_benchmark_state();
        b.iter(|| {
            let task = create_sample_task(0);
            black_box(state.create_task(task));
        });
    });
}

fn create_1000_tasks_benchmark(c: &mut criterion::Criterion) {
    c.bench_function("scheduler_create_1000_tasks", |b| {
        let state = create_benchmark_state();
        b.iter(|| {
            for i in 0..1000 {
                let task = create_sample_task(i);
                state.create_task(task);
            }
        });
    });
}

fn get_task_benchmark(c: &mut criterion::Criterion) {
    let state = create_benchmark_state();
    let ids: Vec<String> = (0..1000)
        .map(|i| {
            let task = create_sample_task(i);
            state.create_task(task)
        })
        .collect();

    c.bench_function("scheduler_get_task", |b| {
        let mut idx = 0;
        b.iter(|| {
            let id = &ids[idx % 1000];
            black_box(state.get_task(id));
            idx += 1;
        });
    });
}

fn list_tasks_benchmark(c: &mut criterion::Criterion) {
    let state = create_benchmark_state();
    for i in 0..1000 {
        let task = create_sample_task(i);
        state.create_task(task);
    }

    c.bench_function("scheduler_list_tasks_1000", |b| {
        black_box(state.list_tasks());
    });
}

fn delete_task_benchmark(c: &mut criterion::Criterion) {
    let state = create_benchmark_state();
    let ids: Vec<String> = (0..100)
        .map(|i| {
            let task = create_sample_task(i);
            state.create_task(task)
        })
        .collect();

    c.bench_function("scheduler_delete_task", |b| {
        let mut idx = 0;
        b.iter(|| {
            let id = &ids[idx % 100];
            black_box(state.delete_task(id));
            idx += 1;
        });
    });
}

// ============================================================================
// Workflow Benchmarks
// ============================================================================

fn create_workflow_benchmark(c: &mut criterion::Criterion) {
    c.bench_function("scheduler_create_workflow", |b| {
        let state = create_benchmark_state();
        b.iter(|| {
            let workflow = create_sample_workflow(0);
            black_box(state.create_workflow(workflow));
        });
    });
}

fn create_100_workflows_benchmark(c: &mut criterion::Criterion) {
    c.bench_function("scheduler_create_100_workflows", |b| {
        let state = create_benchmark_state();
        b.iter(|| {
            for i in 0..100 {
                let workflow = create_sample_workflow(i);
                state.create_workflow(workflow);
            }
        });
    });
}

fn list_workflows_benchmark(c: &mut criterion::Criterion) {
    let state = create_benchmark_state();
    for i in 0..100 {
        let workflow = create_sample_workflow(i);
        state.create_workflow(workflow);
    }

    c.bench_function("scheduler_list_workflows_100", |b| {
        black_box(state.list_workflows());
    });
}

// ============================================================================
// Agent Benchmarks
// ============================================================================

fn register_agent_benchmark(c: &mut criterion::Criterion) {
    c.bench_function("scheduler_register_agent", |b| {
        let state = create_benchmark_state();
        b.iter(|| {
            let agent = create_sample_agent(0);
            black_box(state.register_agent(agent));
        });
    });
}

fn register_100_agents_benchmark(c: &mut criterion::Criterion) {
    c.bench_function("scheduler_register_100_agents", |b| {
        let state = create_benchmark_state();
        b.iter(|| {
            for i in 0..100 {
                let agent = create_sample_agent(i);
                state.register_agent(agent);
            }
        });
    });
}

fn list_agents_benchmark(c: &mut criterion::Criterion) {
    let state = create_benchmark_state();
    for i in 0..100 {
        let agent = create_sample_agent(i);
        state.register_agent(agent);
    }

    c.bench_function("scheduler_list_agents_100", |b| {
        black_box(state.list_agents());
    });
}

fn update_agent_status_benchmark(c: &mut criterion::Criterion) {
    let state = create_benchmark_state();
    let ids: Vec<String> = (0..100)
        .map(|i| {
            let agent = create_sample_agent(i);
            state.register_agent(agent)
        })
        .collect();

    c.bench_function("scheduler_update_agent_status", |b| {
        let mut idx = 0;
        b.iter(|| {
            let id = &ids[idx % 100];
            let status = if idx % 2 == 0 {
                AgentStatus::Busy
            } else {
                AgentStatus::Idle
            };
            black_box(state.update_agent_status(id, status, None));
            idx += 1;
        });
    });
}

// ============================================================================
// Execution History Benchmarks
// ============================================================================

fn record_execution_benchmark(c: &mut criterion::Criterion) {
    c.bench_function("scheduler_record_execution", |b| {
        let state = create_benchmark_state();
        b.iter(|| {
            let record = ExecutionRecord::new(format!("workflow_{}", black_box(0)));
            black_box(state.record_execution(record));
        });
    });
}

fn get_execution_history_benchmark(c: &mut criterion::Criterion) {
    let state = create_benchmark_state();
    for i in 0..1000 {
        let record = ExecutionRecord::new(format!("workflow_{}", i));
        state.record_execution(record);
    }

    c.bench_function("scheduler_get_execution_history_1000", |b| {
        black_box(state.get_execution_history(100));
    });
}

// ============================================================================
// Mixed Workload Benchmark
// ============================================================================

fn mixed_workload_benchmark(c: &mut criterion::Criterion) {
    c.bench_function("scheduler_mixed_workload_100_ops", |b| {
        let state = create_benchmark_state();

        b.iter(|| {
            // Create tasks
            for i in 0..20 {
                let task = create_sample_task(i);
                state.create_task(task);
            }

            // Create workflows
            for i in 0..10 {
                let workflow = create_sample_workflow(i);
                state.create_workflow(workflow);
            }

            // Register agents
            for i in 0..10 {
                let agent = create_sample_agent(i);
                state.register_agent(agent);
            }

            // List operations
            black_box(state.list_tasks());
            black_box(state.list_workflows());
            black_box(state.list_agents());

            // Record executions
            for i in 0..5 {
                let record = ExecutionRecord::new(format!("workflow_{}", i));
                state.record_execution(record);
            }
        });
    });
}

// Register all benchmarks
criterion_group!(
    benches,
    // Task benchmarks
    create_task_benchmark,
    create_1000_tasks_benchmark,
    get_task_benchmark,
    list_tasks_benchmark,
    delete_task_benchmark,
    // Workflow benchmarks
    create_workflow_benchmark,
    create_100_workflows_benchmark,
    list_workflows_benchmark,
    // Agent benchmarks
    register_agent_benchmark,
    register_100_agents_benchmark,
    list_agents_benchmark,
    update_agent_status_benchmark,
    // Execution history benchmarks
    record_execution_benchmark,
    get_execution_history_benchmark,
    // Mixed workload
    mixed_workload_benchmark,
);
criterion_main!(benches);
