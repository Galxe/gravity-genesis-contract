#!/bin/bash
set -e

# Color definitions
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Script configuration
REPO_ROOT=$(pwd)
SRC_DIR="$REPO_ROOT/src"
AUDIT_DIR="$REPO_ROOT/audit"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

# Tool selection from command line argument
TOOL="${1:-all}"

# Print usage
print_usage() {
    echo "Usage: $0 [tool]"
    echo "Tools: slither, mythril, 4naly3er, aderyn, all"
    echo "Default: all (runs all tools)"
}

# Check if src directory exists
check_src_dir() {
    if [ ! -d "$SRC_DIR" ]; then
        echo -e "${RED}Error: src/ directory not found!${NC}"
        echo "Please make sure you're running this script from the project root directory."
        exit 1
    fi
}

# Create audit directory if it doesn't exist
create_audit_dir() {
    mkdir -p "$AUDIT_DIR"
}

# Common header for all reports
generate_report_header() {
    local tool_name=$1
    local report_file=$2
    
    cat > "$report_file" << EOF
# $tool_name Security Analysis Report

Generated on: $(date)
Repository: $(basename "$REPO_ROOT")

---

EOF
}

# Run Slither analysis
run_slither() {
    echo -e "${BLUE}Running Slither security analysis...${NC}"
    
    # Check if Slither is installed
    if ! command -v slither &> /dev/null; then
        echo -e "${YELLOW}Slither not found. Attempting to install...${NC}"
        if command -v pip3 &> /dev/null; then
            pip3 install slither-analyzer
        elif command -v pip &> /dev/null; then
            pip install slither-analyzer
        else
            echo -e "${RED}Error: pip not found. Please install Slither manually.${NC}"
            exit 1
        fi
    fi
    
    local REPORT_FILE="$AUDIT_DIR/slither_report_$TIMESTAMP.md"
    local JSON_FILE="$AUDIT_DIR/slither_report_$TIMESTAMP.json"
    
    generate_report_header "Slither" "$REPORT_FILE"
    
    # Check for config file
    local CONFIG_ARGS=""
    if [ -f "$REPO_ROOT/slither.config.json" ]; then
        CONFIG_ARGS="--config-file slither.config.json"
    fi
    
    # Run analysis
    echo -e "${YELLOW}Analyzing contracts...${NC}"
    set +e
    slither . $CONFIG_ARGS --json "$JSON_FILE" 2>&1 | tee -a "$REPORT_FILE"
    local SLITHER_EXIT_CODE=$?
    set -e
    
    # Parse results
    if [ -f "$JSON_FILE" ]; then
        local HIGH_COUNT=$(jq '[.results.detectors[] | select(.impact == "High")] | length' "$JSON_FILE" 2>/dev/null || echo "0")
        local MEDIUM_COUNT=$(jq '[.results.detectors[] | select(.impact == "Medium")] | length' "$JSON_FILE" 2>/dev/null || echo "0")
        local LOW_COUNT=$(jq '[.results.detectors[] | select(.impact == "Low")] | length' "$JSON_FILE" 2>/dev/null || echo "0")
        local INFO_COUNT=$(jq '[.results.detectors[] | select(.impact == "Informational")] | length' "$JSON_FILE" 2>/dev/null || echo "0")
        
        echo -e "\n${GREEN}Slither Analysis Complete!${NC}"
        echo -e "Report saved to: ${BLUE}$REPORT_FILE${NC}"
        echo -e "JSON report saved to: ${BLUE}$JSON_FILE${NC}"
        echo -e "\n${YELLOW}Summary:${NC}"
        echo -e "  High Impact: ${RED}$HIGH_COUNT${NC}"
        echo -e "  Medium Impact: ${YELLOW}$MEDIUM_COUNT${NC}"
        echo -e "  Low Impact: ${BLUE}$LOW_COUNT${NC}"
        echo -e "  Informational: $INFO_COUNT"
    fi
}

# Run Mythril analysis
run_mythril() {
    echo -e "${BLUE}Running Mythril security analysis...${NC}"
    
    # Check if Mythril is installed
    if ! command -v myth &> /dev/null; then
        echo -e "${YELLOW}Mythril not found. Attempting to install...${NC}"
        if command -v pip3 &> /dev/null; then
            pip3 install mythril
        elif command -v pip &> /dev/null; then
            pip install mythril
        else
            echo -e "${RED}Error: pip not found. Please install Mythril manually.${NC}"
            exit 1
        fi
    fi
    
    local REPORT_FILE="$AUDIT_DIR/mythril_report_$TIMESTAMP.md"
    generate_report_header "Mythril" "$REPORT_FILE"
    
    # Find all Solidity files
    local FILES=$(find "$SRC_DIR" -name "*.sol" -type f)
    local TOTAL_FILES=$(echo "$FILES" | wc -l | tr -d ' ')
    local CURRENT_FILE=0
    local TOTAL_VULNS=0
    
    echo -e "${YELLOW}Found $TOTAL_FILES Solidity files to analyze${NC}"
    
    for FILE in $FILES; do
        CURRENT_FILE=$((CURRENT_FILE + 1))
        local RELATIVE_PATH=${FILE#$REPO_ROOT/}
        
        echo -e "\n${BLUE}[$CURRENT_FILE/$TOTAL_FILES] Analyzing: $RELATIVE_PATH${NC}"
        echo -e "\n## File: $RELATIVE_PATH\n" >> "$REPORT_FILE"
        
        set +e
        local OUTPUT=$(timeout 300 myth analyze "$FILE" --execution-timeout 60 2>&1)
        local MYTH_EXIT_CODE=$?
        set -e
        
        if [ $MYTH_EXIT_CODE -eq 124 ]; then
            echo -e "${YELLOW}Analysis timed out for $RELATIVE_PATH${NC}"
            echo "Analysis timed out after 5 minutes." >> "$REPORT_FILE"
        elif [ $MYTH_EXIT_CODE -eq 0 ]; then
            if echo "$OUTPUT" | grep -q "The analysis was completed successfully. No issues were detected."; then
                echo -e "${GREEN}No vulnerabilities found${NC}"
                echo "No vulnerabilities detected." >> "$REPORT_FILE"
            else
                local VULN_COUNT=$(echo "$OUTPUT" | grep -c "SWC ID:" || true)
                TOTAL_VULNS=$((TOTAL_VULNS + VULN_COUNT))
                echo -e "${RED}Found $VULN_COUNT potential vulnerabilities${NC}"
                echo "$OUTPUT" >> "$REPORT_FILE"
            fi
        else
            echo -e "${YELLOW}Analysis completed with warnings${NC}"
            echo "$OUTPUT" >> "$REPORT_FILE"
        fi
    done
    
    echo -e "\n${GREEN}Mythril Analysis Complete!${NC}"
    echo -e "Report saved to: ${BLUE}$REPORT_FILE${NC}"
    echo -e "\n${YELLOW}Summary:${NC}"
    echo -e "  Total files analyzed: $TOTAL_FILES"
    echo -e "  Total vulnerabilities found: ${RED}$TOTAL_VULNS${NC}"
}

# Run 4naly3er analysis
run_4naly3er() {
    echo -e "${BLUE}Running 4naly3er security analysis...${NC}"
    
    local TOOL_DIR="$REPO_ROOT/tools/4naly3er"
    local REPORT_FILE="$AUDIT_DIR/4naly3er_report_$TIMESTAMP.md"
    
    # Clone or update 4naly3er
    if [ ! -d "$TOOL_DIR" ]; then
        echo -e "${YELLOW}Cloning 4naly3er repository...${NC}"
        mkdir -p "$(dirname "$TOOL_DIR")"
        git clone https://github.com/Picodes/4naly3er.git "$TOOL_DIR"
    else
        echo -e "${YELLOW}Updating 4naly3er repository...${NC}"
        cd "$TOOL_DIR"
        git pull
        cd "$REPO_ROOT"
    fi
    
    # Install dependencies
    cd "$TOOL_DIR"
    if command -v yarn &> /dev/null; then
        yarn install
    elif command -v npm &> /dev/null; then
        npm install
    else
        echo -e "${RED}Error: Neither yarn nor npm found. Please install Node.js.${NC}"
        exit 1
    fi
    
    # Copy remappings if exists
    if [ -f "$REPO_ROOT/remappings.txt" ]; then
        cp "$REPO_ROOT/remappings.txt" "$TOOL_DIR/"
    fi
    
    # Apply fix for storage-collision issue
    local TARGET_FILE="$TOOL_DIR/src/issues/H/storageCollision.ts"
    if [ -f "$TARGET_FILE" ]; then
        if [[ "$OSTYPE" == "darwin"* ]]; then
            sed -i '' '/storageCollision,/d' "$TARGET_FILE"
        else
            sed -i '/storageCollision,/d' "$TARGET_FILE"
        fi
    fi
    
    generate_report_header "4naly3er" "$REPORT_FILE"
    
    # Run analysis
    echo -e "${YELLOW}Analyzing contracts...${NC}"
    cd "$TOOL_DIR"
    set +e
    yarn analyze "$SRC_DIR" "$REPO_ROOT" >> "$REPORT_FILE" 2>&1
    local ANALYZE_EXIT_CODE=$?
    set -e
    
    cd "$REPO_ROOT"
    
    echo -e "\n${GREEN}4naly3er Analysis Complete!${NC}"
    echo -e "Report saved to: ${BLUE}$REPORT_FILE${NC}"
}

# Run Aderyn analysis
run_aderyn() {
    echo -e "${BLUE}Running Aderyn static analysis...${NC}"
    
    # Check if Aderyn is installed
    if ! command -v aderyn &> /dev/null; then
        echo -e "${YELLOW}Aderyn not found. Attempting to install...${NC}"
        
        if [[ "$OSTYPE" == "linux-gnu"* ]] || [[ "$OSTYPE" == "darwin"* ]]; then
            curl -L https://raw.githubusercontent.com/Cyfrin/aderyn/dev/cyfrinup/install | bash
            export PATH="$HOME/.cyfrin/bin:$PATH"
        elif command -v npm &> /dev/null; then
            npm install -g aderyn
        elif command -v brew &> /dev/null; then
            brew install aderyn
        else
            echo -e "${RED}Error: Could not install Aderyn. Please install manually.${NC}"
            exit 1
        fi
    fi
    
    local REPORT_FILE="$AUDIT_DIR/aderyn_report_$TIMESTAMP.md"
    local SUMMARY_FILE="$AUDIT_DIR/aderyn_summary_$TIMESTAMP.md"
    
    # Run analysis
    echo -e "${YELLOW}Analyzing contracts...${NC}"
    set +e
    aderyn . --output "$REPORT_FILE" 2>&1 | tee "$SUMMARY_FILE"
    local ADERYN_EXIT_CODE=$?
    set -e
    
    # Parse results
    local HIGH_COUNT=$(grep -c "High:" "$SUMMARY_FILE" 2>/dev/null || echo "0")
    local MEDIUM_COUNT=$(grep -c "Medium:" "$SUMMARY_FILE" 2>/dev/null || echo "0")
    local LOW_COUNT=$(grep -c "Low:" "$SUMMARY_FILE" 2>/dev/null || echo "0")
    local INFO_COUNT=$(grep -c "Informational:" "$SUMMARY_FILE" 2>/dev/null || echo "0")
    
    echo -e "\n${GREEN}Aderyn Analysis Complete!${NC}"
    echo -e "Report saved to: ${BLUE}$REPORT_FILE${NC}"
    echo -e "Summary saved to: ${BLUE}$SUMMARY_FILE${NC}"
    echo -e "\n${YELLOW}Summary:${NC}"
    echo -e "  High Severity: ${RED}$HIGH_COUNT${NC}"
    echo -e "  Medium Severity: ${YELLOW}$MEDIUM_COUNT${NC}"
    echo -e "  Low Severity: ${BLUE}$LOW_COUNT${NC}"
    echo -e "  Informational: $INFO_COUNT"
}

# Main execution
main() {
    check_src_dir
    create_audit_dir
    
    case "$TOOL" in
        slither)
            run_slither
            ;;
        mythril)
            run_mythril
            ;;
        4naly3er)
            run_4naly3er
            ;;
        aderyn)
            run_aderyn
            ;;
        all)
            echo -e "${BLUE}Running comprehensive security audit...${NC}\n"
            run_slither
            echo ""
            run_mythril
            echo ""
            run_4naly3er
            echo ""
            run_aderyn
            echo -e "\n${GREEN}Comprehensive security audit complete!${NC}"
            echo -e "All reports saved in: ${BLUE}$AUDIT_DIR${NC}"
            ;;
        *)
            echo -e "${RED}Error: Unknown tool '$TOOL'${NC}"
            print_usage
            exit 1
            ;;
    esac
    
    echo -e "\n${YELLOW}Recommendations:${NC}"
    echo "1. Review all findings carefully, especially High and Medium severity issues"
    echo "2. Cross-reference findings between different tools"
    echo "3. Consider the context and business logic when evaluating issues"
    echo "4. Run tests after implementing fixes"
    echo -e "\n${BLUE}Happy auditing! 🛡️${NC}"
}

# Run main function
main