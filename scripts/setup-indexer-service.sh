#!/bin/bash
# Setup script for Claude Token Optimizer Indexer Service

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SERVICE_NAME="claude-indexer"
PROJECT_DIR="/home/${USER}/Prog/Rust/Ts-tools/claude-ts-tools"
BINARY_PATH="${HOME}/bin/token-optimizer"
FALLBACK_BINARY="${PROJECT_DIR}/target/release/token-optimizer"

echo -e "${BLUE}Claude Token Optimizer Indexer Service Setup${NC}"
echo "=============================================="

# Check if running as root
if [ "$EUID" -eq 0 ]; then
    echo -e "${RED}Error: Do not run this script as root${NC}"
    echo "This script should be run as your regular user"
    exit 1
fi

# Check if project directory exists
if [ ! -d "${PROJECT_DIR}" ]; then
    echo -e "${RED}Error: Project directory not found: ${PROJECT_DIR}${NC}"
    exit 1
fi

# Check for binary in ~/bin first, then build if needed
if [ -f "${BINARY_PATH}" ]; then
    echo -e "${GREEN}Binary found in ~/bin: ${BINARY_PATH}${NC}"
elif [ -f "${FALLBACK_BINARY}" ]; then
    echo -e "${YELLOW}Binary found in project, installing to ~/bin...${NC}"
    mkdir -p "${HOME}/bin"
    cp "${FALLBACK_BINARY}" "${BINARY_PATH}"
    echo -e "${GREEN}Binary installed: ${BINARY_PATH}${NC}"
else
    echo -e "${YELLOW}Building release binary...${NC}"
    cd "${PROJECT_DIR}"
    cargo build --release
    if [ $? -ne 0 ]; then
        echo -e "${RED}Error: Failed to build release binary${NC}"
        exit 1
    fi
    echo -e "${YELLOW}Installing binary to ~/bin...${NC}"
    mkdir -p "${HOME}/bin"
    cp "${FALLBACK_BINARY}" "${BINARY_PATH}"
    echo -e "${GREEN}Binary built and installed: ${BINARY_PATH}${NC}"
fi

# Create systemd user directory if it doesn't exist
SYSTEMD_USER_DIR="${HOME}/.config/systemd/user"
mkdir -p "${SYSTEMD_USER_DIR}"

# Copy service file to user systemd directory
echo -e "${YELLOW}Installing service file...${NC}"
cp "${PROJECT_DIR}/claude-indexer.service" "${SYSTEMD_USER_DIR}/${SERVICE_NAME}@.service"

# Replace placeholders in service file
sed -i "s|%i|${USER}|g" "${SYSTEMD_USER_DIR}/${SERVICE_NAME}@.service"

echo -e "${GREEN}Service file installed: ${SYSTEMD_USER_DIR}/${SERVICE_NAME}@.service${NC}"

# Reload systemd user daemon
echo -e "${YELLOW}Reloading systemd user daemon...${NC}"
systemctl --user daemon-reload

# Enable linger for user (allows user services to run without being logged in)
echo -e "${YELLOW}Enabling linger for user ${USER}...${NC}"
sudo loginctl enable-linger "${USER}"

echo -e "${GREEN}Setup complete!${NC}"
echo
echo "Available commands:"
echo -e "${BLUE}Start indexing:${NC}    systemctl --user start ${SERVICE_NAME}@${USER}"
echo -e "${BLUE}Stop indexing:${NC}     systemctl --user stop ${SERVICE_NAME}@${USER}"
echo -e "${BLUE}Check status:${NC}      systemctl --user status ${SERVICE_NAME}@${USER}"
echo -e "${BLUE}View logs:${NC}         journalctl --user -u ${SERVICE_NAME}@${USER} -f"
echo -e "${BLUE}Enable auto-start:${NC} systemctl --user enable ${SERVICE_NAME}@${USER}"
echo
echo "Example usage:"
echo -e "${YELLOW}# Start background indexing${NC}"
echo "systemctl --user start ${SERVICE_NAME}@${USER}"
echo
echo -e "${YELLOW}# Monitor progress${NC}"
echo "journalctl --user -u ${SERVICE_NAME}@${USER} -f"
echo
echo -e "${YELLOW}# Check if indexing is complete${NC}"
echo "systemctl --user status ${SERVICE_NAME}@${USER}"