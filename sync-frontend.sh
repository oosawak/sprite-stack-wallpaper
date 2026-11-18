#!/bin/sh
# ゲーム本体（sprite-stack-proto の index.html）をアプリへ取り込む。
# Web版に入っている Cloudflare Web Analytics のビーコンは、アプリでは不要なので外す。
set -e
cp ../sprite-stack-proto/index.html frontend/index.html
python3 - <<'PY'
import re
p = 'frontend/index.html'
s = open(p).read()
s2 = re.sub(r'<script[^>]*cloudflareinsights[^>]*></script>', '', s)
assert s2 != s, 'beacon not found'
open(p, 'w').write(s2)
print('beacon stripped')
PY
