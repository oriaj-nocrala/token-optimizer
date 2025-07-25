#!/bin/bash
# Quick indexing script - run indexing in background and monitor progress

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

SERVICE_NAME="claude-indexer@${USER}"
PROJECT_DIR="/home/${USER}/Prog/Rust/Ts-tools/claude-ts-tools"

echo -e "${BLUE}Claude Token Optimizer - Quick Background Indexing${NC}"
echo "=================================================="

# Check if service is installed
if [ ! -f "${HOME}/.config/systemd/user/claude-indexer@.service" ]; then
    echo -e "${YELLOW}Service not installed. Running setup script...${NC}"
    "${PROJECT_DIR}/scripts/setup-indexer-service.sh"
    echo
fi

# Start the indexing service
echo -e "${YELLOW}Starting background indexing...${NC}"
systemctl --user start "${SERVICE_NAME}"

echo -e "${GREEN}Indexing started in background!${NC}"
echo
echo "Monitor progress with:"
echo -e "${BLUE}journalctl --user -u ${SERVICE_NAME} -f${NC}"
echo
echo "Check status with:"
echo -e "${BLUE}systemctl --user status ${SERVICE_NAME}${NC}"
echo
echo "Stop indexing with:"
echo -e "${BLUE}systemctl --user stop ${SERVICE_NAME}${NC}"

# Show immediate status
echo
echo -e "${YELLOW}Current status:${NC}"
systemctl --user status "${SERVICE_NAME}" --no-pager -l