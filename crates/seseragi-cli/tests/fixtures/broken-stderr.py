import json
import os
import subprocess
import sys

read, write = os.pipe()
os.close(read)
try:
    result = subprocess.run(
        sys.argv[1:], stdout=subprocess.PIPE, stderr=write, text=True, timeout=20
    )
finally:
    os.close(write)
print(json.dumps({"code": result.returncode, "stdout": result.stdout}))
