import { existsSync, readdirSync, readFileSync } from "node:fs";
import { basename, dirname, join, resolve } from "node:path";

const root = resolve(import.meta.dir, "../examples/spec");
const lessonsDir = join(root, "lessons");
const errors: string[] = [];

const lessons = readdirSync(lessonsDir)
  .filter((name) => /^\d{2}-.*\.ssrg$/.test(name))
  .sort();

lessons.forEach((name, index) => {
  const expectedNumber = String(index + 1).padStart(2, "0");
  const actualNumber = name.slice(0, 2);
  const path = join(lessonsDir, name);
  const source = readFileSync(path, "utf8");
  const lines = source.split("\n");

  if (actualNumber !== expectedNumber) {
    errors.push(`${name}: expected lesson number ${expectedNumber}`);
  }
  if (!lines[0]?.startsWith(`// Lesson ${actualNumber}:`)) {
    errors.push(`${name}: first line must declare its lesson number and goal`);
  }
  if (!lines[1]?.startsWith("// Prerequisite:")) {
    errors.push(`${name}: second line must declare prerequisites`);
  }

  const marker = lines.findIndex((line) =>
    line.startsWith("// Expected stdout:"),
  );
  if (marker < 0) {
    errors.push(`${name}: missing Expected stdout marker`);
    return;
  }

  const snapshot = lines[marker]?.slice("// Expected stdout:".length).trim();
  if (snapshot) {
    const snapshotPath = resolve(dirname(path), snapshot);
    if (!existsSync(snapshotPath)) {
      errors.push(`${name}: missing stdout snapshot ${snapshot}`);
    }
  } else if (!lines.slice(marker + 1).some((line) => line.startsWith("// "))) {
    errors.push(`${name}: inline Expected stdout is empty`);
  }
});

const lessonReadme = readFileSync(join(lessonsDir, "README.md"), "utf8");
for (const name of lessons) {
  const number = basename(name).slice(0, 2);
  if (!new RegExp(`\\|\\s*${number}\\s*\\|`).test(lessonReadme)) {
    errors.push(`${name}: missing from lessons/README.md learning path`);
  }
}

if (errors.length > 0) {
  console.error(errors.join("\n"));
  process.exit(1);
}

console.log(`Spec lessons: ${lessons.length} checked`);
