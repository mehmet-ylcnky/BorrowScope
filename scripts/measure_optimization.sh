#!/bin/bash
# Measure optimization impact of tracking feature

set -e

echo "=== BorrowScope Optimization Measurement ==="
echo ""

# Clean build
echo "Cleaning previous builds..."
cargo clean

# Build without tracking
echo ""
echo "=== Building WITHOUT tracking feature ==="
time cargo build --release 2>&1 | grep "Finished"

if [ -f "target/release/borrowscope-cli" ]; then
    SIZE_WITHOUT=$(ls -lh target/release/borrowscope-cli | awk '{print $5}')
    echo "Binary size without tracking: $SIZE_WITHOUT"
fi

# Clean for next build
cargo clean

# Build with tracking
echo ""
echo "=== Building WITH tracking feature ==="
time cargo build --release --features borrowscope-runtime/track 2>&1 | grep "Finished"

if [ -f "target/release/borrowscope-cli" ]; then
    SIZE_WITH=$(ls -lh target/release/borrowscope-cli | awk '{print $5}')
    echo "Binary size with tracking: $SIZE_WITH"
fi

# Run benchmarks
echo ""
echo "=== Running benchmarks WITHOUT tracking ==="
cargo bench --bench optimization -- --noplot 2>&1 | grep -E "(time:|track_)"

echo ""
echo "=== Running benchmarks WITH tracking ==="
cargo bench --bench optimization --features borrowscope-runtime/track -- --noplot 2>&1 | grep -E "(time:|track_)"

echo ""
echo "=== Optimization measurement complete ==="
