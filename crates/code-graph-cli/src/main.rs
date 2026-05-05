//! CLI tool for Code Intelligence Graph
//!
//! Usage:
//!   code-graph build <repo_root> [--output <graph.json>]
//!   code-graph query <graph.json> node <node_id>
//!   code-graph query <graph.json> file <file_path>
//!   code-graph query <graph.json> neighbors <node_id>

use sqlrustgo_code_graph::ast::Indexer;
use sqlrustgo_code_graph::graph::builder::build_graph;
use sqlrustgo_code_graph::graph::NodeType;
use sqlrustgo_code_graph::runtime::GraphRuntime;
use sqlrustgo_code_graph::store;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "code-graph")]
#[command(version = "0.1.0")]
#[command(about = "Code Intelligence Graph CLI for SQLRustGo")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build a code graph from a repository
    Build {
        #[arg(default_value = ".")]
        repo_root: String,
        #[arg(short, long, default_value = "graph.json")]
        output: String,
    },
    /// Query a previously built graph
    Query {
        graph_path: String,
        #[command(subcommand)]
        subcommand: QueryCommands,
    },
    /// Print graph statistics
    Stats { graph_path: String },
}

#[derive(Subcommand)]
enum QueryCommands {
    Node { node_id: String },
    File { file_path: String },
    Neighbors { node_id: String },
    Functions,
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { repo_root, output } => {
            println!("Building code graph from: {}", repo_root);
            let indexer = Indexer::new(&repo_root)?;
            let nodes = indexer.index_all()?;
            println!("Extracted {} symbols", nodes.len());
            let graph = build_graph(nodes);
            println!(
                "Built graph: {} nodes, {} edges",
                graph.node_count(),
                graph.edge_count()
            );
            store::save_graph(&graph, &output)?;
            println!("Saved to: {}", output);
        }
        Commands::Query { graph_path, subcommand } => {
            let graph = store::load_graph(&graph_path)?;
            let runtime = GraphRuntime::new(graph);

            match subcommand {
                QueryCommands::Node { node_id } => {
                    if let Some(node) = runtime.get_node(&node_id) {
                        println!("Node: {} ({})", node.name, node.id);
                        println!("  Type: {}", node.node_type);
                        println!(
                            "  File: {}:{}:{}",
                            node.file_path, node.line_start, node.line_end
                        );
                        if let Some(sig) = &node.signature {
                            println!("  Signature: {}", sig);
                        }
                    } else {
                        println!("Node not found: {}", node_id);
                    }
                }
                QueryCommands::File { file_path } => {
                    let nodes = runtime.locate_by_file(&file_path);
                    if nodes.is_empty() {
                        println!("No nodes found in: {}", file_path);
                    } else {
                        println!("{} nodes in {}:", nodes.len(), file_path);
                        for node in nodes {
                            println!(
                                "  [{}] {} ({}) :{}-{}",
                                node.id, node.name, node.node_type, node.line_start, node.line_end
                            );
                        }
                    }
                }
                QueryCommands::Neighbors { node_id } => {
                    let neighbors = runtime.get_neighbors(&node_id);
                    if neighbors.is_empty() {
                        println!("No neighbors for: {}", node_id);
                    } else {
                        println!("{} neighbors:", neighbors.len());
                        for node in neighbors {
                            println!("  [{}] {} ({})", node.id, node.name, node.node_type);
                        }
                    }
                }
                QueryCommands::Functions => {
                    let funcs = runtime.get_nodes_by_type(NodeType::Function);
                    println!("{} functions:", funcs.len());
                    for node in funcs.iter().take(20) {
                        println!(
                            "  [{}] {} — {}:{}-{}",
                            node.id, node.name, node.file_path, node.line_start, node.line_end
                        );
                    }
                    if funcs.len() > 20 {
                        println!("  ... and {} more", funcs.len() - 20);
                    }
                }
            }
        }
        Commands::Stats { graph_path } => {
            let graph = store::load_graph(&graph_path)?;
            let runtime = GraphRuntime::new(graph);
            let stats = runtime.stats();
            println!("Graph Statistics:");
            println!("  Nodes: {}", stats.total_nodes);
            println!("  Edges: {}", stats.total_edges);
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
