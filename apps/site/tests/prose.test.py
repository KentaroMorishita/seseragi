"""Foreground tests for the Python prose extractor."""

import importlib.util
import subprocess
import sys
import unittest
from pathlib import Path

sys.dont_write_bytecode = True
script = Path(__file__).resolve().parents[1] / "scripts" / "check-prose.py"
spec = importlib.util.spec_from_file_location("check_prose", script)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class ArticleProseTest(unittest.TestCase):
    def test_excludes_chrome_and_code_but_protects_inline_syntax(self):
        parser = module.ArticleProse()
        parser.feed('''<header><p>Navigation</p></header>
<main id="content"><h1>Title</h1><p>名前は<code>x &lt; y</code>です。</p>
<pre><code>fn main -&gt; Unit = {}</code></pre>
<p>一行目<br>二行目</p><ul><li>項目<code>A</code></li></ul>
<p>説明<script>window.secret()</script><style>p{}</style>です。</p></main>
<footer><p>Footer</p></footer>''')
        self.assertEqual(parser.paragraphs, [
            "名前は`x < y`です。",
            "一行目\n二行目",
            "- 項目`A`",
            "説明です。",
        ])

    def test_requires_explicit_output_and_installed_skill_paths(self):
        result = subprocess.run(
            [sys.executable, "-B", str(script)],
            capture_output=True, text=True, timeout=10, check=False,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("YOMIYASU_LINT_PATH", result.stderr)


if __name__ == "__main__":
    unittest.main()
