"""Read-only CoreIr facts for possible shared backend analyses."""
from collections import Counter
import json


def walk(value):
    if isinstance(value, dict):
        yield value
        for child in value.values():
            yield from walk(child)
    elif isinstance(value, list):
        for child in value:
            yield from walk(child)


def free_names(expr, globals):
    if not isinstance(expr, dict):
        return set()
    kind = expr.get("kind")
    if kind == "variable":
        result = {expr["name"]} if expr["name"] not in globals and "::" not in expr["name"] else set()
        return result | free_names({"evidence": expr.get("evidence", [])}, globals)
    if kind == "parameter":
        return {f"$dictionary:{expr['index']}"} if "index" in expr else set()
    if kind == "lambda":
        return free_names(expr["body"], globals) - {expr["parameter"]["id"]}
    if kind == "sequence":
        result = free_names(expr["result"], globals)
        for statement in reversed(expr["statements"]):
            if statement["kind"] != "pure-let":
                raise ValueError("analysis slice supports only pure let sequences")
            result.discard(statement["name"])
            result |= free_names(statement["value"], globals)
        return result
    if kind == "decision":
        result = free_names(expr["scrutinee"], globals)
        for branch in expr["branches"]:
            branch_free = free_names(branch["value"], globals) | free_names(branch.get("guard"), globals)
            result |= branch_free - {binding["name"] for binding in branch["bindings"]}
        return result
    result = set()
    if kind == "call" and expr["callee"] not in globals and "::" not in expr["callee"]:
        result.add(expr["callee"])
    for child in expr.values():
        if isinstance(child, dict):
            result |= free_names(child, globals)
        elif isinstance(child, list):
            for item in child:
                result |= free_names(item, globals)
    return result


def dependency(evidence):
    if evidence["kind"] == "parameter":
        return "parameter-dependent"
    if evidence["kind"] == "imported":
        return "external-unproven"
    children = [dependency(e["evidence"]) for e in evidence.get("evidenceArguments", [])]
    return "parameter-dependent" if "parameter-dependent" in children else "external-unproven" if "external-unproven" in children else "closed"


def analyze(module):
    functions = {function["symbol"]: function for function in module["functions"]}
    globals = set(functions) | {variant["symbol"] for adt in module.get("adts", []) for variant in adt["variants"]}
    factories = {instance["identity"] for instance in module.get("instances", [])
                 if instance.get("typeParameters") or instance.get("constraints")}
    result = {"schema": 1, "module": module["module"], "calls": [], "lambdas": [], "dictionary_factories": []}
    bodies = [(function["symbol"], function["body"]) for function in module["functions"]]
    bodies += [(instance["identity"] + "::" + method["name"], method["body"])
               for instance in module.get("instances", []) if instance["implementation"]["kind"] == "user-defined"
               for method in instance["implementation"]["methods"]]
    for owner, body in bodies:
        for node in walk(body):
            if node.get("kind") == "call" and node["callee"] in functions:
                arity = len(functions[node["callee"]]["parameters"])
                supplied = len(node["arguments"])
                result["calls"].append({"owner": owner, "callee": node["callee"], "source_arity": arity,
                                        "supplied": supplied, "class": "saturated" if arity == supplied else "partial" if supplied < arity else "returned-application",
                                        "evidence_argument_index": node.get("evidenceArgumentIndex"),
                                        "deferred_evidence_parameters": len(node.get("deferredEvidenceParameters", []))})
            if node.get("kind") == "lambda":
                result["lambdas"].append({"owner": owner, "origin": node["origin"], "free_names": sorted(free_names(node, globals))})
            if node.get("kind") == "local" and node.get("identity") in factories:
                result["dictionary_factories"].append({"owner": owner, "identity": node["identity"],
                                                      "dependency": dependency(node), "evidence_key": json.dumps(node, sort_keys=True)})
    result["call_classes"] = dict(Counter(call["class"] for call in result["calls"]))
    result["factory_dependencies"] = dict(Counter(factory["dependency"] for factory in result["dictionary_factories"]))
    return result


def self_check():
    variable = lambda name: {"kind": "variable", "name": name}
    expr = {"kind": "lambda", "parameter": {"id": "x"}, "body": {"kind": "sequence", "statements": [
        {"kind": "pure-let", "name": "y", "value": variable("outer")}], "result": variable("y")}}
    assert free_names(expr, set()) == {"outer"}
    assert dependency({"kind": "local", "evidenceArguments": [{"evidence": {"kind": "parameter", "index": 0}}]}) == "parameter-dependent"
    assert dependency({"kind": "local", "evidenceArguments": []}) == "closed"
