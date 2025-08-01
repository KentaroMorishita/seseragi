#!/usr/bin/env bun
import { Command } from "commander"
import { compileCommand } from "./cli/compile.js"
import { formatCommand } from "./cli/format.js"
import { runCommand } from "./cli/run.js"

const program = new Command()

program
  .name("seseragi")
  .description("Seseragi functional programming language tools")
  .version("1.0.0")

// デフォルトコマンド：コンパイル
program
  .argument("[file]", "Seseragi source file to compile")
  .option("-o, --output <file>", "Output TypeScript file")
  .option("--auto", "Auto-recompile on file changes")
  .option("--no-comments", "Do not generate comments in output")
  .option(
    "--function-declarations",
    "Use function declarations instead of arrow functions"
  )
  .option(
    "--runtime <mode>",
    "Runtime mode: embedded, import (default: import)",
    "import"
  )
  .action(async (file, options) => {
    if (!file) {
      program.help()
      return
    }
    await compileCommand({
      input: file,
      output: options.output,
      watch: options.auto,
      generateComments: !options.noComments,
      useArrowFunctions: !options.functionDeclarations,
      runtimeMode: options.runtime,
    })
  })

// フォーマットコマンド
program
  .command("fmt")
  .description("Format Seseragi source files")
  .argument("<file>", "File to format")
  .option("-o, --output <file>", "Output file (default: overwrite)")
  .option("-c, --check", "Check if file is formatted correctly")
  .option("--remove-whitespace", "Remove extra whitespace", false)
  .option("--normalize-spacing", "Normalize operator spacing", false)
  .action(async (file, options) => {
    await formatCommand({
      input: file,
      output: options.output,
      inPlace: !options.output,
      check: options.check,
      removeWhitespace: options.removeWhitespace,
      normalizeSpacing: options.normalizeSpacing,
    })
  })

// runコマンド（実行機能）
program
  .command("run")
  .description("Run Seseragi source file directly")
  .argument("<file>", "Seseragi source file to run")
  .option(
    "--temp-dir <dir>",
    "Directory for temporary files (default: system temp)"
  )
  .option("--keep-temp", "Keep temporary TypeScript file for debugging")
  .option("-w, --watch", "Watch for file changes and re-run")
  .action(async (file, options) => {
    try {
      await runCommand({
        input: file,
        tempDir: options.tempDir,
        keepTemp: options.keepTemp,
        watch: options.watch,
      })
    } catch (error) {
      console.error(
        "Execution failed:",
        error instanceof Error ? error.message : error
      )
      process.exit(1)
    }
  })

program.parse()
