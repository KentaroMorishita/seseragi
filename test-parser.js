#!/usr/bin/env bun

import { readFileSync } from "fs";
import { Parser } from "./src/parser.ts";

const filename = process.argv[2];
if (!filename) {
  console.log("Usage: bun test-parser.js <filename>");
  process.exit(1);
}

try {
  const source = readFileSync(filename, "utf-8");
  console.log("=== Parsing:", filename, "===");
  console.log("Source:");
  console.log(source);
  console.log("\n=== Parse Result ===");
  
  const parser = new Parser(source);
  const result = parser.parse();
  
  if (result.errors && result.errors.length > 0) {
    console.log("Parse Errors:");
    result.errors.forEach(err => console.log("  ", err.message));
  } else {
    console.log("Parse Success!");
    console.log("Statements:", result.statements.length);
    result.statements.forEach((stmt, i) => {
      console.log(`  ${i}: ${stmt.kind}`);
    });
  }
} catch (error) {
  console.error("Error:", error.message);
}