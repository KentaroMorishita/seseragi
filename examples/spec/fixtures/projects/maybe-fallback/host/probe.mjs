const events = [];
export function record(value) { events.push(value); return value; }
export function trace() { return events.join(","); }
export function recordUnit() { events.push(99); }
