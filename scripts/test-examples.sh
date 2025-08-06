#!/bin/bash

# Seseragi Examples Test Script
# すべてのexamplesファイルをコンパイル・実行してテストします

set -e  # エラーで停止

# 色付きの出力
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔧 Seseragi Examples Test Runner${NC}"
echo "================================"

# 統計情報
TOTAL=0
PASSED=0
FAILED=0
FAILED_FILES=()

# テスト関数
test_file() {
    local file="$1"
    local name=$(basename "$file" .ssrg)

    echo -e "${YELLOW}📝 Testing: $file${NC}"

    # コンパイル
    if seseragi "$file" -o "temp_${name}.ts" >/dev/null 2>&1; then
        echo -e "   ✅ Compile: OK"

        # TypeScript実行
        if bun run "temp_${name}.ts" >/dev/null 2>&1; then
            echo -e "   ✅ Execute: OK"
            echo -e "${GREEN}   ✅ PASSED${NC}"
            ((PASSED++))
        else
            echo -e "   ❌ Execute: FAILED"
            echo -e "${RED}   ❌ FAILED${NC}"
            ((FAILED++))
            FAILED_FILES+=("$file")
        fi

        # 一時ファイル削除
        rm -f "temp_${name}.ts"
    else
        echo -e "   ❌ Compile: FAILED"
        echo -e "${RED}   ❌ FAILED${NC}"
        ((FAILED++))
        FAILED_FILES+=("$file")
    fi

    ((TOTAL++))
    echo ""
}

# basicsテスト
echo -e "${BLUE}📚 Testing Basics${NC}"
echo "=================="
for file in examples/basics/*.ssrg; do
    test_file "$file"
done

# intermediateテスト
echo -e "${BLUE}🔧 Testing Intermediate${NC}"
echo "======================="
for file in examples/intermediate/*.ssrg; do
    test_file "$file"
done

# advancedテスト
echo -e "${BLUE}🚀 Testing Advanced${NC}"
echo "==================="
for file in examples/advanced/*.ssrg; do
    test_file "$file"
done

# tmpテスト
echo -e "${BLUE}🚀 Testing Tmp${NC}"
echo "==================="
for file in tmp/*.ssrg; do
    test_file "$file"
done
for file in tmp/need/*.ssrg; do
    test_file "$file"
done

# playgroundテスト
echo -e "${BLUE}🎮 Testing Playground${NC}"
echo "===================="
if [ -f "examples/playground.ssrg" ]; then
    test_file "examples/playground.ssrg"
fi

# 結果サマリー
echo -e "${BLUE}📊 Test Results Summary${NC}"
echo "======================="
echo -e "Total files: ${TOTAL}"
echo -e "${GREEN}Passed: ${PASSED}${NC}"
echo -e "${RED}Failed: ${FAILED}${NC}"

if [ ${FAILED} -gt 0 ]; then
    echo ""
    echo -e "${RED}❌ Failed files:${NC}"
    printf '%s\n' "${FAILED_FILES[@]}"
    exit 1
else
    echo -e "${GREEN}🎉 All examples passed!${NC}"
    exit 0
fi