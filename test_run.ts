// Generated TypeScript from Seseragi
type Shape = { type: 'Circle', data0: number } | { type: 'Rectangle', data0: number, data1: number };

const Circle = (data0: number) => ({ type: 'Circle' as const, data0 });
const Rectangle = (data0: number, data1: number) => ({ type: 'Rectangle' as const, data0, data1 });

const circle = Circle(5);
const rect = Rectangle(10, 20);

console.log("Circle:", circle);
console.log("Rectangle:", rect);