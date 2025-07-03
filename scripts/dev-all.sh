#!/bin/bash
set -e

echo "🚀 Full development rebuild..."

./scripts/build.sh
./scripts/check.sh

echo "🎉 Everything rebuilt! Reload VS Code to apply changes."