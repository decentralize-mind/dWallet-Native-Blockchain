#!/bin/bash

# dWallet Native Blockchain - Test Runner
# Run all tests or specific pallet tests

set -e

echo "🚀 dWallet Native Blockchain - Test Suite"
echo "=========================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to run tests for a package
run_tests() {
    local package=$1
    echo -e "${YELLOW}Testing: $package${NC}"
    if cargo test -p "$package" --verbose; then
        echo -e "${GREEN}✓ $package tests passed${NC}\n"
    else
        echo -e "${RED}✗ $package tests failed${NC}\n"
        return 1
    fi
}

# Parse command line arguments
case "${1:-all}" in
    all)
        echo "Running all tests..."
        echo ""
        
        # Test primitives
        run_tests "dwallet-primitives"
        
        # Test pallets
        run_tests "pallet-protocol-registry"
        run_tests "pallet-dwt-token"
        run_tests "pallet-rate-limiter"
        run_tests "pallet-cross-chain"
        run_tests "pallet-security-root"
        run_tests "pallet-governance"
        run_tests "pallet-business-logic"
        run_tests "pallet-protocol-security"
        run_tests "pallet-settlement"
        run_tests "pallet-intelligence"
        run_tests "pallet-lending"
        run_tests "pallet-dex"
        run_tests "pallet-nft-membership"
        run_tests "pallet-staking"
        run_tests "pallet-treasury"
        
        echo -e "${GREEN}=========================================="
        echo "✓ All tests completed!"
        echo "==========================================${NC}"
        ;;
    
    primitives)
        run_tests "dwallet-primitives"
        ;;
    
    registry)
        run_tests "pallet-protocol-registry"
        ;;
    
    token)
        run_tests "pallet-dwt-token"
        ;;
    
    *)
        echo "Usage: $0 {all|primitives|registry|token}"
        echo ""
        echo "Examples:"
        echo "  ./scripts/test.sh all           # Run all tests"
        echo "  ./scripts/test.sh primitives    # Test primitives only"
        echo "  ./scripts/test.sh registry      # Test protocol registry"
        echo "  ./scripts/test.sh token         # Test DWT token"
        exit 1
        ;;
esac
