import { test, expect } from 'bun:test';
import { formatSeseragiCode, removeExtraWhitespace, normalizeOperatorSpacing } from '../src/formatter/index.js';

test('removeExtraWhitespace removes multiple spaces', () => {
  const input = 'fn   add  a   :Int -> b:Int->Int = a+b';
  const expected = 'fn add a :Int -> b:Int->Int = a+b';
  expect(removeExtraWhitespace(input)).toBe(expected);
});

test('removeExtraWhitespace removes trailing whitespace', () => {
  const input = 'fn add a :Int -> Int = a + b   \n';
  const expected = 'fn add a :Int -> Int = a + b\n';
  expect(removeExtraWhitespace(input)).toBe(expected);
});

test('normalizeOperatorSpacing adds proper spacing', () => {
  const input = 'fn add a:Int->b:Int->Int=a+b';
  const expected = 'fn add a: Int -> b: Int -> Int = a + b';
  expect(normalizeOperatorSpacing(input)).toBe(expected);
});

test('normalizeOperatorSpacing handles pipe operators', () => {
  const input = 'x|double|square';
  const expected = 'x | double | square';
  expect(normalizeOperatorSpacing(input)).toBe(expected);
});

test('simple function formatting', () => {
  const input = 'fn add a :Int -> b :Int -> Int = a + b';
  const result = formatSeseragiCode(input);
  expect(result).toContain('fn add');
  expect(result).toContain('->');
});

test('basic whitespace cleanup', () => {
  const input = `

fn   add   a   :Int   ->   b:Int->Int   =   a+b


let   x   :Int   =   42   

`;
  const result = removeExtraWhitespace(input);
  expect(result).not.toContain('   ');
  expect(result).not.toMatch(/\n{3,}/);
});