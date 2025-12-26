#!/bin/bash
# Thermite Development Environment Verification Script
# Verifies all required tools and services are correctly configured

set -e

echo "========================================="
echo "Thermite Development Environment Check"
echo "========================================="

echo ""
echo "=== Mise Tools ==="
mise ls --current | grep -E "rust|python|uv"

echo ""
echo "=== Rust Toolchain ==="
mise x -- cargo --version
mise x -- rustc --version

echo ""
echo "=== Python & uv ==="
mise x -- python --version
uv --version

echo ""
echo "=== PostgreSQL ==="
psql --version
psql -U delorenj -d thermite -c "SELECT 1 AS test" > /dev/null && echo "✓ thermite database accessible"

echo ""
echo "=== RabbitMQ ==="
systemctl is-active rabbitmq-server > /dev/null && echo "✓ RabbitMQ running"
curl -s http://localhost:15672/ > /dev/null && echo "✓ Management UI accessible at http://localhost:15672"

echo ""
echo "=== Docker ==="
docker --version
docker compose version
docker ps > /dev/null && echo "✓ Docker daemon running"

echo ""
echo "========================================="
echo "✓ All checks passed!"
echo "========================================="
echo ""
echo "Development environment is ready for Thermite."
echo "Next step: Review docs/sprint-plan-thermite-2025-12-26.md"
