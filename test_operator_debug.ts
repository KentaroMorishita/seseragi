// Generated TypeScript code from Seseragi

// Seseragi minimal runtime

type Maybe<T> = { tag: 'Just'; value: T } | { tag: 'Nothing' };
const Just = <T>(value: T): Maybe<T> => ({ tag: 'Just', value });
const Nothing: Maybe<never> = { tag: 'Nothing' };


// Struct method and operator dispatch tables
let __structMethods: Record<string, Record<string, Function>> = {};
let __structOperators: Record<string, Record<string, Function>> = {};

// Method dispatch helper
function __dispatchMethod(obj: any, methodName: string, ...args: any[]): any {
  // 構造体のフィールドアクセスの場合は直接返す
  if (args.length === 0 && obj.hasOwnProperty(methodName)) {
    return obj[methodName];
  }
  const structName = obj.constructor.name;
  const structMethods = __structMethods[structName];
  if (structMethods && structMethods[methodName]) {
    return structMethods[methodName](obj, ...args);
  }
  throw new Error(`Method '${methodName}' not found for struct '${structName}'`);
}

// Operator dispatch helper
function __dispatchOperator(left: any, operator: string, right: any): any {
  const structName = left.constructor.name;
  const structOperators = __structOperators[structName];
  if (structOperators && structOperators[operator]) {
    return structOperators[operator](left, right);
  }
  // Fall back to native JavaScript operator
  switch (operator) {
    case '+': return left + right;
    case '-': return left - right;
    case '*': return left * right;
    case '/': return left / right;
    case '%': return left % right;
    case '==': return left == right;
    case '!=': return left != right;
    case '<': return left < right;
    case '>': return left > right;
    case '<=': return left <= right;
    case '>=': return left >= right;
    case '&&': return left && right;
    case '||': return left || right;
    default: throw new Error(`Unknown operator: ${operator}`);
  }
}


class Point {
  constructor(public x: number, public y: number) {}
}

// Point implementation
function __ssrg_Point_f4pl4mu_op_add(self: Point, other: Point): Point {
  return new Point((self.x + other.x), (self.y + other.y));
}

// Initialize dispatch tables immediately
(() => {
  // Initialize method dispatch table
  __structMethods = {
    "Point": {

    },
  };

  // Initialize operator dispatch table
  __structOperators = {
    "Point": {
      "+": __ssrg_Point_f4pl4mu_op_add
    },
  };

})();
