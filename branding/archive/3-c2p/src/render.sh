#!/bin/sh
# render.sh in.svg size out.png  — transparent-background PNG via headless Edge
svg=$(cygpath -m "$(realpath "$1")"); out=$(cygpath -m "$(realpath -m "$3")"); s=$2
html=$(mktemp -p . r.XXXX.html)
printf '<html><body style="margin:0;background:transparent"><img src="file:///%s" style="display:block;width:%spx;height:%spx"></body></html>' "$svg" "$s" "$s" > "$html"
"/c/Program Files (x86)/Microsoft/Edge/Application/msedge.exe" --headless=new --disable-gpu --hide-scrollbars --default-background-color=00000000 --force-device-scale-factor=1 --window-size=$s,$s --screenshot="$out" "file:///$(cygpath -m "$(realpath "$html")")" >/dev/null 2>&1
rm -f "$html"
