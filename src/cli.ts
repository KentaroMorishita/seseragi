#!/usr/bin/env node
import { Command } from 'commander';
import { formatCommand } from './cli/format.js';

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

program.parse();