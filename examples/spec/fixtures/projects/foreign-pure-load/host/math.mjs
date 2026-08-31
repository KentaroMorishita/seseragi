export const VERSION = "1.0.0";
export const UNKNOWN_VALUE = Symbol("unknown");
export const OBJECT_VALUE = { ready: true };
export const NUMBER_VALUE = Number.NaN;
export const STRING_VALUE = "raw";
export const NULL_VALUE = null;
export const UNDEFINED_VALUE = undefined;
export const MUTABLE_VALUES = [1, 2];
export const RAW_CALLBACK = (value) => `[${value}]`;

export function add(left, right) {
  return left + right;
}

export class Counter {
  constructor(value) {
    this.value = value;
  }

  add(delta) {
    return this.value + delta;
  }
}

export function mutateAndSum(values) {
  values.push(3);
  return values.reduce((sum, value) => sum + value, 0);
}

export function callTwice(callback) {
  return callback(callback("go"));
}

export function inspectBoundaries(
  unknown,
  object,
  number,
  string,
  nullValue,
  undefinedValue,
  nullOrValue,
  nullableValue,
  undefinedOrValue,
) {
  return (
    typeof unknown === "symbol" &&
    object === OBJECT_VALUE &&
    Number.isNaN(number) &&
    string === "raw" &&
    nullValue === null &&
    undefinedValue === undefined &&
    nullOrValue === null &&
    nullableValue === null &&
    undefinedOrValue === undefined
  );
}

export function mutateHostArray(values) {
  values.push(3);
  return values.length;
}

export function invokeRawCallback(callback) {
  return callback("raw");
}

export const Operations = {
  multiply(left, right) {
    return left * right;
  },
  Format: {
    label(value) {
      return `[${value}]`;
    },
  },
};
