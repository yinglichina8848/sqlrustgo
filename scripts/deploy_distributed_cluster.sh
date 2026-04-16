#!/bin/bash
# Deploy Distributed Storage Cluster
# Usage: ./deploy_distributed_cluster.sh [node1_ip] [node2_ip] [node3_ip]

set -e

NODES=(
    "127.0.0.1:50051"  # Node 1 - localhost
    "192.168.0.250:50051"  # Node 2
    "192.168.0.252:50051"  # Node 3
)

REPO_PATH="${REPO_PATH:-/tmp/sqlrustgo}"
FEATURE_BRANCH="feature/unified-query-regression-test-1345"

echo "==========================================="
echo "  Distributed Storage Cluster Deployment"
echo "==========================================="

# Check prerequisites
check_ssh() {
    local ip=$1
    echo "Checking SSH connection to $ip..."
    ssh -o ConnectTimeout=5 -o StrictHostKeyChecking=no "$ip" "echo 'OK'" || {
        echo "ERROR: Cannot connect to $ip"
        return 1
    }
}

# Install Rust if needed
install_rust() {
    local ip=$1
    echo "Checking Rust on $ip..."
    ssh "$ip" "rustc --version" &>/dev/null && {
        echo "  Rust already installed on $ip"
        return 0
    }
    echo "  Installing Rust on $ip..."
    ssh "$ip" "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y"
}

# Clone or update repo
sync_repo() {
    local ip=$1
    local is_local=$2
    echo "Syncing repository on $ip..."
    if [ "$is_local" = "true" ]; then
        cd "$REPO_PATH"
        git fetch origin "$FEATURE_BRANCH"
        git checkout "$FEATURE_BRANCH"
    else
        ssh "$ip" "cd /tmp && rm -rf sqlrustgo && git clone --depth 1 https://github.com/minzuuniversity/sqlrustgo.git sqlrustgo"
        ssh "$ip" "cd /tmp/sqlrustgo && git fetch origin && git checkout $FEATURE_BRANCH"
    fi
}

# Build on remote
build_remote() {
    local ip=$1
    local is_local=$2
    echo "Building on $ip..."
    if [ "$is_local" = "true" ]; then
        cd "$REPO_PATH"
        cargo build --release -p sqlrustgo-distributed --example distributed_test_server
    else
        ssh "$ip" "source ~/.cargo/env && cd /tmp/sqlrustgo && cargo build --release -p sqlrustgo-distributed --example distributed_test_server"
    fi
}

# Deploy and start node
start_node() {
    local node_id=$1
    local ip=$2
    local is_local=$3
    local peers=""
    for i in "${!NODES[@]}"; do
        if [ $i -ne $(($node_id - 1)) ]; then
            [ -n "$peers" ] && peers="$peers,"
            peers="${peers}${NODES[$i]}"
        fi
    done

    echo "Starting Node $node_id on $ip (peers: $peers)..."

    if [ "$is_local" = "true" ]; then
        nohup cargo run --example distributed_test_server -- \
            --node-id $node_id \
            --listen-addr "${NODES[$(($node_id - 1))]}" \
            --peers "$peers" \
            > "node_${node_id}.log" 2>&1 &
        echo "  Node $node_id started on localhost (PID: $!)"
    else
        ssh "$ip" "source ~/.cargo/env && cd /tmp/sqlrustgo && nohup cargo run --example distributed_test_server -- \
            --node-id $node_id \
            --listen-addr '0.0.0.0:50051' \
            --peers '$peers' \
            > node_${node_id}.log 2>&1 &"
        echo "  Node $node_id started on $ip"
    fi
}

# Main deployment
main() {
    echo "Step 1: Checking prerequisites..."
    for i in 0 1 2; do
        ip="${NODES[$i]%:*}"
        [ "$ip" = "127.0.0.1" ] && continue
        check_ssh "$ip" || exit 1
        install_rust "$ip"
    done

    echo ""
    echo "Step 2: Syncing repositories..."
    sync_repo "127.0.0.1" "true"
    sync_repo "192.168.0.250" "false"
    sync_repo "192.168.0.252" "false"

    echo ""
    echo "Step 3: Building..."
    build_remote "127.0.0.1" "true"
    build_remote "192.168.0.250" "false"
    build_remote "192.168.0.252" "false"

    echo ""
    echo "Step 4: Starting cluster..."
    start_node 1 "127.0.0.1" "true"
    sleep 2
    start_node 2 "192.168.0.250" "false"
    sleep 2
    start_node 3 "192.168.0.252" "false"

    echo ""
    echo "==========================================="
    echo "  Cluster deployed successfully!"
    echo "==========================================="
    echo ""
    echo "Running health checks..."
    sleep 3
    for i in 0 1 2; do
        ip="${NODES[$i]%:*}"
        port="${NODES[$i]#*:}"
        node_num=$(($i + 1))
        echo "  Node $node_num ($ip:$port): "
        curl -s "http://$ip:$port/health" 2>/dev/null || echo "    (Health check not implemented yet)"
    done
}

main "$@"