import { describe, test, expect } from "bun:test";
import { UsageAnalyzer } from "../src/usage-analyzer";
import { parse } from "../src/parser";
import { lex } from "../src/lexer";
import { generateTypeScript } from "../src/codegen";

describe("Show Function Detection", () => {
  function analyzeShowUsage(source: string) {
    const tokens = lex(source);
    const parseResult = parse(tokens);
    
    if (!parseResult.statements || parseResult.statements.length === 0) {
      throw new Error("No statements found");
    }
    
    const analyzer = new UsageAnalyzer();
    const usage = analyzer.analyze(parseResult.statements);
    
    return {
      showNeeded: usage.needsBuiltins.show,
      toStringNeeded: usage.needsBuiltins.toString,
      functionApplicationNeeded: usage.needsFunctionApplication
    };
  }

  function verifyShowInRuntime(source: string) {
    const tokens = lex(source);
    const parseResult = parse(tokens);
    
    if (!parseResult.statements || parseResult.statements.length === 0) {
      throw new Error("No statements found");
    }
    
    const generatedCode = generateTypeScript(parseResult.statements, { runtimeMode: 'minimal' });
    return generatedCode.includes('const show = ');
  }

  test("should detect show function in function application operator syntax (show $ expr)", () => {
    const usage = analyzeShowUsage('show $ Just 42');
    
    expect(usage.showNeeded).toBe(true);
    expect(usage.toStringNeeded).toBe(true); // show depends on toString
    expect(usage.functionApplicationNeeded).toBe(true);
  });

  test("should include show function in runtime for function application operator", () => {
    const showIncluded = verifyShowInRuntime('show $ Just 42');
    expect(showIncluded).toBe(true);
  });

  test("should detect show function in function call syntax (show(expr))", () => {
    const usage = analyzeShowUsage('show(Just 42)');
    
    expect(usage.showNeeded).toBe(true);
    expect(usage.toStringNeeded).toBe(true);
  });

  test("should include show function in runtime for function call", () => {
    const showIncluded = verifyShowInRuntime('show(Just 42)');
    expect(showIncluded).toBe(true);
  });

  test("should detect show function in function application syntax (show expr)", () => {
    const usage = analyzeShowUsage('show data');
    
    expect(usage.showNeeded).toBe(true);
    expect(usage.toStringNeeded).toBe(true);
  });

  test("should include show function in runtime for function application", () => {
    const showIncluded = verifyShowInRuntime('show data');
    expect(showIncluded).toBe(true);
  });

  test("should detect show function in nested expressions", () => {
    const usage = analyzeShowUsage('print $ show $ Just 42');
    
    expect(usage.showNeeded).toBe(true);
    expect(usage.toStringNeeded).toBe(true);
    expect(usage.functionApplicationNeeded).toBe(true);
  });

  test("should include show function in runtime for nested expressions", () => {
    const showIncluded = verifyShowInRuntime('print $ show $ Just 42');
    expect(showIncluded).toBe(true);
  });

  test("should detect show function in double show expressions", () => {
    const usage = analyzeShowUsage('show $ show $ Just 42');
    
    expect(usage.showNeeded).toBe(true);
    expect(usage.toStringNeeded).toBe(true);
    expect(usage.functionApplicationNeeded).toBe(true);
  });

  test("should include show function in runtime for double show expressions", () => {
    const showIncluded = verifyShowInRuntime('show $ show $ Just 42');
    expect(showIncluded).toBe(true);
  });

  test("should NOT detect show function when not used", () => {
    const usage = analyzeShowUsage('print "Hello"');
    
    expect(usage.showNeeded).toBe(false);
  });

  test("should NOT include show function in runtime when not used", () => {
    const showIncluded = verifyShowInRuntime('print "Hello"');
    expect(showIncluded).toBe(false);
  });
});