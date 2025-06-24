#!/usr/bin/env bun
import { Command } from 'commander';
import { formatCommand } from './cli/format.js';
import { compileCommand } from './cli/compile.js';
import { runCommand } from './cli/run.js';

const program = new Command();

program
  .name('seseragi')
  .description('Seseragi functional programming language tools')
  .version('1.0.0');

program
  .command('format')
  .description('Format Seseragi source files')
  .argument('<file>', 'File to format')
  .option('-o, --output <file>', 'Output file (default: stdout)')
  .option('-i, --in-place', 'Edit file in place')
  .option('-c, --check', 'Check if file is formatted correctly')
  .option('--remove-whitespace', 'Remove extra whitespace', true)
  .option('--normalize-spacing', 'Normalize operator spacing', true)
  .action(async (file, options) => {
    await formatCommand({
      input: file,
      output: options.output,
      inPlace: options.inPlace,
      check: options.check,
      removeWhitespace: options.removeWhitespace,
      normalizeSpacing: options.normalizeSpacing,
    });
  });

program
  .command('compile')
  .description('Compile Seseragi source to TypeScript')
  .argument('<file>', 'Seseragi source file to compile')
  .option('-o, --output <file>', 'Output TypeScript file')
  .option('-w, --watch', 'Watch for file changes and recompile')
  .option('--no-comments', 'Do not generate comments in output')
  .option('--function-declarations', 'Use function declarations instead of arrow functions')
  .action(async (file, options) => {
    await compileCommand({
      input: file,
      output: options.output,
      watch: options.watch,
      generateComments: !options.noComments,
      useArrowFunctions: !options.functionDeclarations,
    });
  });

program
  .command('run')
  .description('Run Seseragi source file directly')
  .argument('<file>', 'Seseragi source file to run')
  .option('--temp-dir <dir>', 'Directory for temporary files (default: system temp)')
  .option('--keep-temp', 'Keep temporary TypeScript file for debugging')
  .action(async (file, options) => {
    try {
      await runCommand({
        input: file,
        tempDir: options.tempDir,
        keepTemp: options.keepTemp,
      });
    } catch (error) {
      console.error('Execution failed:', error instanceof Error ? error.message : error);
      process.exit(1);
    }
  });

program.parse();