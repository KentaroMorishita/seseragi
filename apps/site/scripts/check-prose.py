"""Run an installed yomiyasu lint on rendered Japanese prose, not source code.

Usage: python3 check-prose.py OUTPUT_DIR YOMIYASU_LINT_PATH
The JSON report is advisory. This command starts no server or background process.
"""

import importlib.util
import json
import sys
from html.parser import HTMLParser
from pathlib import Path

sys.dont_write_bytecode = True


class ArticleProse(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.stack = []
        self.main_depth = None
        self.paragraphs = []
        self.current = None
        self.inline_code = False

    def handle_starttag(self, tag, attrs):
        attributes = dict(attrs)
        if tag not in {"area", "base", "br", "col", "embed", "hr", "img",
                       "input", "link", "meta", "param", "source", "track", "wbr"}:
            self.stack.append(tag)
        if tag == "main" and attributes.get("id") == "content":
            self.main_depth = len(self.stack)
        if self.main_depth is None or "pre" in self.stack:
            return
        if tag in {"p", "li"} and self.current is None:
            self.current = [tag, []]
        if tag == "code" and self.current is not None:
            self.current[1].append("`")
            self.inline_code = True
        if tag == "br" and self.current is not None:
            self.current[1].append("\n")

    def handle_endtag(self, tag):
        if tag == "code" and self.inline_code and self.current is not None:
            self.current[1].append("`")
            self.inline_code = False
        if self.current is not None and tag == self.current[0]:
            text = "".join(self.current[1]).strip()
            if text:
                self.paragraphs.append(("- " if tag == "li" else "") + text)
            self.current = None
        if tag == "main":
            self.main_depth = None
        if tag in self.stack:
            position = len(self.stack) - 1 - self.stack[::-1].index(tag)
            del self.stack[position:]

    def handle_data(self, data):
        if self.current is not None and self.main_depth is not None:
            if not any(tag in self.stack for tag in {"pre", "script", "style"}):
                self.current[1].append(data)


def main():
    if len(sys.argv) != 3:
        raise SystemExit(__doc__)
    output = Path(sys.argv[1]).resolve()
    lint_path = Path(sys.argv[2]).resolve()
    spec = importlib.util.spec_from_file_location("yomiyasu_lint", lint_path)
    lint = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(lint)
    pages = []
    for path in sorted((output / "ja").rglob("index.html")):
        parser = ArticleProse()
        parser.feed(path.read_text(encoding="utf-8"))
        prose = "\n\n".join(parser.paragraphs)
        if not prose:
            raise SystemExit(f"No article prose extracted: {path}")
        report = lint.lint_text(prose)
        pages.append({"route": "/" + str(path.parent.relative_to(output)) + "/",
                      "paragraphs": len(parser.paragraphs), **report})
    if not pages:
        raise SystemExit(f"No Japanese pages found in {output}")
    print(json.dumps({"advisory": True, "pages": len(pages),
                      "reports": pages}, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
