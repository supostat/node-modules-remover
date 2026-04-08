#!/bin/bash
# Creates fake project directories for demo recording
set -e

DEMO_DIR="/tmp/dev-cleaner-demo"
rm -rf "$DEMO_DIR"
mkdir -p "$DEMO_DIR"

create_node_project() {
    local name="$1"
    local size_mb="$2"
    local project_dir="$DEMO_DIR/$name"
    mkdir -p "$project_dir/node_modules"
    echo '{}' > "$project_dir/package.json"
    dd if=/dev/urandom of="$project_dir/node_modules/bundle.dat" bs=1048576 count="$size_mb" 2>/dev/null
}

create_rust_project() {
    local name="$1"
    local size_mb="$2"
    local project_dir="$DEMO_DIR/$name"
    mkdir -p "$project_dir/target"
    echo '[package]' > "$project_dir/Cargo.toml"
    dd if=/dev/urandom of="$project_dir/target/build.dat" bs=1048576 count="$size_mb" 2>/dev/null
}

create_node_project "react-dashboard"    180
create_node_project "api-server"         95
create_node_project "mobile-app"         250
create_node_project "landing-page"       45
create_node_project "old-prototype"      320
create_node_project "shared-lib"         12

create_rust_project "rust-api"           400
create_rust_project "cli-tool"           150
create_rust_project "shared-crate"       60

echo "Demo data created in $DEMO_DIR"
ls -la "$DEMO_DIR"
