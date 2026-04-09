#!/bin/bash

# Script to create demo projects test-lib and test-app

set -e  # Stop script on any error

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Setting up demo projects ===${NC}"

# 1. Remove .temp folder if exists and create fresh
TEMP_DIR=".temp"

if [ -d "$TEMP_DIR" ]; then
    echo -e "${BLUE}🗑️  Removing existing $TEMP_DIR folder...${NC}"
    rm -rf "$TEMP_DIR"
fi

echo -e "${BLUE}📁 Creating $TEMP_DIR folder...${NC}"
mkdir -p "$TEMP_DIR"

# Move into .temp folder
cd "$TEMP_DIR"

# 2. Create npm project test-lib
echo -e "${BLUE}📦 Creating test-lib project...${NC}"
mkdir -p test-lib
cd test-lib

# Initialize npm project with default settings
npm init -y > /dev/null 2>&1

# Create index.mjs
cat > index.mjs << 'EOF'
export function print() {
  console.log('test-lib');
}
EOF

# Go back
cd ..

# 3. Create npm project test-app
echo -e "${BLUE}📦 Creating test-app project...${NC}"
mkdir -p test-app
cd test-app

# Initialize npm project
npm init -y > /dev/null 2>&1

# Create index.mjs
cat > index.mjs << 'EOF'
import {print} from "test-lib/index.mjs";

console.log('test-app');

print();
EOF

# Go back
cd ..

echo -e "${GREEN}✅ Projects created successfully!${NC}"
echo ""
echo -e "${GREEN}📁 Folder structure:${NC}"
echo ".temp/"
echo "├── test-lib/"
echo "│   ├── index.mjs"
echo "│   └── package.json"
echo "└── test-app/"
echo "    ├── index.mjs"
echo "    └── package.json"
echo ""
echo -e "${BLUE}💡 To run test-app:${NC}"
echo "  cd .temp/test-app"
echo "  node index.mjs"
