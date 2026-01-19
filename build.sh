#!/usr/bin/env bash
set -e

cd "$(dirname "$0")"

echo "=== Agent Session Recorder Build ==="
echo

# Run tests first
echo "Running tests..."
docker build -f docker/Dockerfile --target test . || {
    echo "Tests failed!"
    exit 1
}
echo "Tests passed!"
echo

# Build release
echo "Building release..."
mkdir -p dist
docker build -f docker/Dockerfile --target export -o dist/ .

if [ -f "dist/asr" ]; then
    echo
    echo "Build successful!"
    echo "Binary: dist/asr"
    ls -lh dist/asr
else
    echo "Build failed - binary not found"
    exit 1
fi
