use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[path = "../../test-support/temporary.rs"]
mod temporary;

struct SourceDirectory(PathBuf);

impl SourceDirectory {
    fn new(source: &str) -> Self {
        let directory = temporary::temporary_directory("seseragi-tail-recursive-currying");
        fs::write(directory.join("main.ssrg"), source).unwrap();
        Self(directory)
    }

    fn run(&self, profile: &str) -> String {
        let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["run", "main.ssrg", "--profile", profile])
            .current_dir(&self.0)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{profile}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap()
    }
}

impl Drop for SourceDirectory {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn tail_recursive_partial_functions_are_reusable_with_independent_arguments() {
    let source = SourceDirectory::new(
        r#"
fn advance count: Int -> value: Int -> Int =
  if count == 0 then value else advance (count - 1) (value + 1)
fn rotate count: Int -> left: Int -> right: Int -> (Int, Int) =
  if count == 0 then (left, right) else rotate (count - 1) right left
fn repeat<A> count: Int -> value: A -> A =
  if count == 0 then value else repeat (count - 1) value
pub effect fn main = do {
  let advanceTwice = advance 2
  println (show (advanceTwice 0))
  println (show (advanceTwice 0))
  println (show (advanceTwice 10))
  let rotated = rotate 100001 1
  let (firstLeft, firstRight) = rotated 2
  let (secondLeft, secondRight) = rotated 3
  println (show firstLeft)
  println (show firstRight)
  println (show secondLeft)
  println (show secondRight)
  let repeated: Int -> Int = repeat 100000
  println (show (repeated 42))
  println (show (repeated 43))
}
"#,
    );
    for profile in ["development", "release"] {
        assert_eq!(
            source.run(profile),
            "2\n2\n12\n2\n1\n3\n1\n42\n43\n",
            "{profile}"
        );
    }
}

#[test]
fn closures_keep_the_parameters_of_the_tail_iteration_that_created_them() {
    let source = SourceDirectory::new(
        r#"
fn capture count: Int -> saved: (Unit -> Int) -> (Unit -> Int) =
  if count == 0 then saved else capture (count - 1) (\_ -> count)
fn local start: Int -> (Unit -> Int) = {
  fn loop count: Int -> saved: (Unit -> Int) -> (Unit -> Int) =
    if count == 0 then saved else loop (count - 1) (\_ -> count + start)
  loop start (\_ -> 99)
}
pub effect fn main = do {
  let captured = capture 3 (\_ -> 99)
  println (show (captured ()))
  println (show (captured ()))
  let fromLocal = local 3
  println (show (fromLocal ()))
  let partial = capture 2
  let first = partial (\_ -> 98)
  let second = partial (\_ -> 97)
  println (show (first ()))
  println (show (second ()))
}
"#,
    );
    for profile in ["development", "release"] {
        assert_eq!(source.run(profile), "1\n1\n4\n1\n1\n", "{profile}");
    }
}
