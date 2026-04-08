#!/bin/bash
# Creates fake node_modules directories for demo recording
set -e

DEMO_DIR="/tmp/dev-cleaner-demo"
rm -rf "$DEMO_DIR"
mkdir -p "$DEMO_DIR"

create_project() {
    local name="$1"
    local size_mb="$2"
    local dir="$DEMO_DIR/$name/node_modules"
    mkdir -p "$dir"
    dd if=/dev/urandom of="$dir/bundle.dat" bs=1048576 count="$size_mb" 2>/dev/null
}

create_project "react-dashboard"    180
create_project "api-server"         95
create_project "mobile-app"         250
create_project "landing-page"       45
create_project "old-prototype"      320
create_project "shared-lib"         12

echo "Demo data created in $DEMO_DIR"
ls -la "$DEMO_DIR"
