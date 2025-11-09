#!/bin/bash

# Example build script with custom FreeRTOS configurations
# Usage: ./build_examples.sh

set -e

echo "=== Build Examples for OSAL-RS with FreeRTOS ==="
echo

# Initial cleanup
echo "🧹 Cleaning build cache..."
cargo clean
echo

# Build 1: Default configuration (auto-detection)
echo "📦 Build 1: Default configuration"
echo "  - Port: Auto-detection (ThirdParty/GCC/Posix on Linux)"
echo "  - Heap: 4 (default)"
cargo build --features freertos
echo "✅ Build completed"
echo

# Build 2: Custom heap
echo "📦 Build 2: Custom heap"
echo "  - Port: Auto-detection"
echo "  - Heap: 3 (malloc/free wrapper)"
cargo clean
FREERTOS_HEAP="3" cargo build --features freertos
echo "✅ Build completed"
echo

# Build 3: Limited memory heap
echo "📦 Build 3: For systems with limited memory"
echo "  - Port: Auto-detection"
echo "  - Heap: 1 (no deallocation)"
cargo clean
FREERTOS_HEAP="1" cargo build --features freertos
echo "✅ Build completed"
echo

# Build 4: Advanced coalescence heap
echo "📦 Build 4: Heap with multiple regions"
echo "  - Port: Auto-detection"
echo "  - Heap: 5 (multiple regions)"
cargo clean
FREERTOS_HEAP="5" cargo build --features freertos
echo "✅ Build completed"
echo

# Build 5: Combined configuration
echo "📦 Build 5: Complete explicit configuration"
echo "  - Port: ThirdParty/GCC/Posix (explicit)"
echo "  - Heap: 2 (simple allocation/deallocation)"
cargo clean
FREERTOS_PORT="ThirdParty/GCC/Posix" FREERTOS_HEAP="2" cargo build --features freertos
echo "✅ Build completed"
echo

echo "🎉 All builds completed successfully!"
echo
echo "💡 Tips:"
echo "  - Use FREERTOS_HEAP=1 for systems with low RAM"
echo "  - Use FREERTOS_HEAP=3 if you already have a custom allocator"
echo "  - Use FREERTOS_HEAP=4 (default) for most cases"
echo "  - Use FREERTOS_HEAP=5 for systems with multiple memory regions"
echo
echo "📚 See FREERTOS_CONFIG.md for complete details"