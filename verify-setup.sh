#!/bin/bash

echo "🔍 Verifying dWallet Native Blockchain Setup..."
echo ""

# Check Rust
echo "✓ Checking Rust installation..."
if command -v rustc &> /dev/null; then
    echo "  ✅ rustc: $(rustc --version)"
else
    echo "  ❌ rustc not found"
    exit 1
fi

if command -v cargo &> /dev/null; then
    echo "  ✅ cargo: $(cargo --version)"
else
    echo "  ❌ cargo not found"
    exit 1
fi

# Check Rust targets
echo ""
echo "✓ Checking Rust targets..."
if rustup target list --installed | grep -q wasm32-unknown-unknown; then
    echo "  ✅ wasm32-unknown-unknown target installed"
else
    echo "  ❌ wasm32-unknown-unknown target missing"
    echo "  Run: rustup target add wasm32-unknown-unknown"
fi

# Check tools
echo ""
echo "✓ Checking development tools..."
tools=("git" "cmake" "protoc")
for tool in "${tools[@]}"; do
    if command -v $tool &> /dev/null; then
        echo "  ✅ $tool: $($tool --version | head -n 1)"
    else
        echo "  ⚠️  $tool not found (may be needed later)"
    fi
done

# Check project structure
echo ""
echo "✓ Checking project structure..."
if [ -f "Cargo.toml" ]; then
    echo "  ✅ Workspace Cargo.toml found"
else
    echo "  ❌ Workspace Cargo.toml missing"
fi

pallet_count=$(ls -d pallets/*/ 2>/dev/null | wc -l | tr -d ' ')
if [ "$pallet_count" -gt 0 ]; then
    echo "  ✅ $pallet_count pallets created"
else
    echo "  ❌ No pallets found"
fi

if [ -d "runtime/src" ]; then
    echo "  ✅ Runtime directory found"
else
    echo "  ❌ Runtime directory missing"
fi

if [ -d "primitives/src" ]; then
    echo "  ✅ Primitives directory found"
else
    echo "  ❌ Primitives directory missing"
fi

echo ""
echo "🎉 Setup verification complete!"
echo ""
echo "Next steps:"
echo "  1. cd ~/blockchain/dwallet-native/dwallet-native"
echo "  2. cargo check (verify everything compiles)"
echo "  3. Start implementing Layer 0 (pallet-protocol-registry)"
echo ""
echo "📚 Documentation:"
echo "  - SETUP-GUIDE.md: Environment setup"
echo "  - PROJECT-STRUCTURE.md: Code organization"
echo "  - IMPLEMENTATION-ROADMAP.md: Week-by-week plan"
echo "  - HARDEST_PART.md: Implementation priorities with DSA"
echo ""

