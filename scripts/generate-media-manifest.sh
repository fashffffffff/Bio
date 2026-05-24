#!/usr/bin/env bash
# Scans media/ and writes media/manifest.json (runs automatically before trunk build).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MEDIA="$ROOT/media"

mkdir -p "$MEDIA"/{ava,video,audio,background}

json_escape() {
  local s="$1"
  s="${s//\\/\\\\}"
  s="${s//\"/\\\"}"
  printf '%s' "$s"
}

json_list() {
  local subdir="$1"
  shift
  local -a allowed=("$@")
  local -a items=()

  shopt -s nullglob
  for f in "$MEDIA/$subdir"/*; do
    [[ -f "$f" ]] || continue
    local base="${f##*/}"
  local ext="${base##*.}"
    local lower
    lower="$(printf '%s' "$ext" | tr '[:upper:]' '[:lower:]')"
    local ok=0
    for a in "${allowed[@]}"; do
      if [[ "$lower" == "$a" ]]; then
        ok=1
        break
      fi
    done
    [[ "$ok" -eq 1 ]] || continue
    items+=("$(json_escape "/media/$subdir/$base")")
  done
  shopt -u nullglob

  if ((${#items[@]} == 0)); then
    printf '[]'
    return
  fi

  local out="\"${items[0]}\""
  local i
  for ((i = 1; i < ${#items[@]}; i++)); do
    out+=", \"${items[$i]}\""
  done
  printf '[%s]' "$out"
}

{
  printf '{\n'
  printf '  "ava": %s,\n' "$(json_list ava png jpg jpeg webp gif avif)"
  printf '  "video": %s,\n' "$(json_list video mp4 webm mov m4v)"
  printf '  "audio": %s,\n' "$(json_list audio mp3 ogg wav m4a flac aac)"
  printf '  "background": %s\n' "$(json_list background png jpg jpeg webp gif avif)"
  printf '}\n'
} >"$MEDIA/manifest.json"

count_json() {
  local n
  n="$(grep -o '"/media/' "$MEDIA/manifest.json" | wc -l | tr -d ' ')"
  printf '%s' "${n:-0}"
}

echo "media manifest: $(count_json) file(s) -> $MEDIA/manifest.json"
