use std::fs;
use std::path::Path;
use std::process::Command;

pub(super) fn check_typed_failure_boundary(root: &Path) -> Result<(), String> {
    let output = Command::new("bun")
        .arg("--eval")
        .arg(
            "import { effectApplicative, effectFunctor, effectMonad, fail, flatMap, mapError, run, succeed } from \"./src/effect.ts\";\n\
             let continued = false;\n\
             const effect = flatMap(fail({ kind: \"expected\" }), () => { continued = true; return succeed(1); });\n\
             const cold = continued === false;\n\
             const result = await run(effect, {});\n\
             let mapped = 0;\n\
             const mappedEffect = mapError(error => { mapped += 1; return { kind: \"mapped\", source: error }; }, fail({ kind: \"source\" }));\n\
             const mapCold = mapped === 0;\n\
             const mappedResult = await run(mappedEffect, {});\n\
             const successResult = await run(mapError(() => { mapped += 100; return { kind: \"impossible\" }; }, succeed(7)), {});\n\
             let defect = false;\n\
             try { await run(mapError(() => ({ kind: \"wrong\" }), () => { throw new Error(\"defect\"); }), {}); } catch (error) { defect = error instanceof Error && error.message === \"defect\"; }\n\
             let mapperDefect = false;\n\
             try { await run(mapError(() => { throw new Error(\"mapper-defect\"); }, fail({ kind: \"source\" })), {}); } catch (error) { mapperDefect = error instanceof Error && error.message === \"mapper-defect\"; }\n\
             let dictionaryEvaluations = 0;\n\
             const dictionarySource = () => { dictionaryEvaluations += 1; return 41; };\n\
             const mappedValue = effectFunctor.map((value) => value + 1)(dictionarySource);\n\
             const appliedValue = effectApplicative.apply(effectApplicative.pure((value) => value + 1))(dictionarySource);\n\
             const flatMappedValue = effectMonad.flatMap((value) => succeed(value + 1))(dictionarySource);\n\
             const dictionaryCold = dictionaryEvaluations === 0;\n\
             const dictionaryResults = [await run(mappedValue, {}), await run(appliedValue, {}), await run(flatMappedValue, {})];\n\
             const dictionaryFailure = await run(effectMonad.flatMap(() => { throw new Error(\"continued-after-failure\"); })(fail(\"stopped\")), {});\n\
             process.stdout.write(JSON.stringify({ cold, continued, result, mapCold, mapped, mappedResult, successResult, defect, mapperDefect, dictionaryCold, dictionaryEvaluations, dictionaryResults, dictionaryFailure }));\n",
        )
        .current_dir(root.join("runtime/ts"))
        .output()
        .map_err(|error| format!("failed to run TypeScript typed failure probe: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "TypeScript typed failure probe failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let expected = b"{\"cold\":true,\"continued\":false,\"result\":{\"kind\":\"failure\",\"error\":{\"kind\":\"expected\"}},\"mapCold\":true,\"mapped\":1,\"mappedResult\":{\"kind\":\"failure\",\"error\":{\"kind\":\"mapped\",\"source\":{\"kind\":\"source\"}}},\"successResult\":{\"kind\":\"success\",\"value\":7},\"defect\":true,\"mapperDefect\":true,\"dictionaryCold\":true,\"dictionaryEvaluations\":3,\"dictionaryResults\":[{\"kind\":\"success\",\"value\":42},{\"kind\":\"success\",\"value\":42},{\"kind\":\"success\",\"value\":42}],\"dictionaryFailure\":{\"kind\":\"failure\",\"error\":\"stopped\"}}";
    if output.stdout != expected {
        return Err(format!(
            "TypeScript typed failure probe returned unexpected result: {}",
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    Ok(())
}

pub(super) fn check_from_either_boundary(root: &Path) -> Result<(), String> {
    check_from_either_inference(root)?;

    let output = Command::new("bun")
        .arg("--eval")
        .arg(
            "import { fromEither, run } from \"./src/effect.ts\";\n\
             import { Left, Right } from \"./src/sum.ts\";\n\
             let evaluated = 0;\n\
             const makeRight = () => { evaluated += 1; return Right(7); };\n\
             const rightEffect = fromEither(makeRight());\n\
             const cold = evaluated === 1;\n\
             const rightResult = await run(rightEffect, {});\n\
             const rightAgain = await run(rightEffect, {});\n\
             const evaluatedOnce = evaluated === 1;\n\
             const error = { kind: \"invalid\" };\n\
             const leftResult = await run(fromEither(Left(error)), {});\n\
             process.stdout.write(JSON.stringify({ cold, evaluatedOnce, rightResult, rightAgain, leftResult, sameError: leftResult.kind === \"failure\" && leftResult.error === error }));\n",
        )
        .current_dir(root.join("runtime/ts"))
        .output()
        .map_err(|error| format!("failed to run TypeScript fromEither probe: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "TypeScript fromEither probe failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let expected = b"{\"cold\":true,\"evaluatedOnce\":true,\"rightResult\":{\"kind\":\"success\",\"value\":7},\"rightAgain\":{\"kind\":\"success\",\"value\":7},\"leftResult\":{\"kind\":\"failure\",\"error\":{\"kind\":\"invalid\"}},\"sameError\":true}";
    if output.stdout != expected {
        return Err(format!(
            "TypeScript fromEither probe returned unexpected result: {}",
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    Ok(())
}

fn check_from_either_inference(root: &Path) -> Result<(), String> {
    let probe_dir = root.join("target/seseragi-conformance/runtime-package");
    fs::create_dir_all(&probe_dir)
        .map_err(|error| format!("failed to create fromEither type probe directory: {error}"))?;
    let probe_path = probe_dir.join("from-either-inference.ts");
    fs::write(
        &probe_path,
        "import { fromEither, type Effect } from \"../../../runtime/ts/src/effect.ts\";\n\
         import type { Either } from \"../../../runtime/ts/src/sum.ts\";\n\
         type AppError = { readonly tag: \"InvalidInput\"; readonly input: string };\n\
         type Hand = { readonly tag: \"Rock\" } | { readonly tag: \"Paper\" };\n\
         declare const parsed: Either<AppError, Hand>;\n\
         const inferred = fromEither(parsed);\n\
         type Equal<Left, Right> = (<Value>() => Value extends Left ? 1 : 2) extends\n\
           (<Value>() => Value extends Right ? 1 : 2)\n\
           ? true\n\
           : false;\n\
         type Assert<Condition extends true> = Condition;\n\
         type InferenceIsExact = Assert<\n\
           Equal<typeof inferred, Effect<unknown, AppError, Hand>>\n\
         >;\n\
         const exact: Effect<unknown, AppError, Hand> = inferred;\n\
         void (null as unknown as InferenceIsExact);\n\
         void exact;\n",
    )
    .map_err(|error| format!("failed to write fromEither type probe: {error}"))?;

    let output = Command::new("bunx")
        .arg("tsc")
        .arg("--noEmit")
        .arg("--strict")
        .arg("--target")
        .arg("ES2022")
        .arg("--module")
        .arg("ESNext")
        .arg("--moduleResolution")
        .arg("bundler")
        .arg("--allowImportingTsExtensions")
        .arg("--skipLibCheck")
        .arg("--types")
        .arg("node")
        .arg(&probe_path)
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to type-check fromEither inference: {error}"))?;
    if output.status.success() {
        return Ok(());
    }
    Err(format!(
        "TypeScript fromEither inference probe failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    ))
}
