#!/usr/bin/env bash
#
# ADR-017 / ADR-022 build-time assertion: the FFmpeg we ship is LGPL-only and
# carries the encoders we committed to, and nothing GPL. Fails loudly.
#
# Usage: tools/build-ffmpeg/check-licence.sh <dir with bin/ and lib/>
set -uo pipefail

DIR="${1:?usage: check-licence.sh <ffmpeg-dir>}"
FF="$DIR/bin/ffmpeg"
FAIL=0
ok()  { printf '  ok    %s\n' "$1"; }
bad() { printf '  FAIL  %s\n' "$1"; FAIL=$((FAIL + 1)); }

echo "check-licence.sh ($DIR)"
[ -x "$FF" ] && [ -x "$DIR/bin/ffprobe" ] && ok "ffmpeg and ffprobe present" || { bad "ffmpeg/ffprobe missing"; exit 1; }

LICENSE_TEXT="$("$FF" -hide_banner -L 2>&1)"
echo "$LICENSE_TEXT" | grep -q "GNU Lesser General Public" && ok "ffmpeg -L reports the LGPL" || bad "ffmpeg -L does not report the LGPL"
echo "$LICENSE_TEXT" | grep -qi "nonfree\|unredistributable" && bad "ffmpeg -L reports a nonfree build" || ok "not nonfree"

BUILDCONF="$("$FF" -hide_banner -buildconf 2>&1)"
echo "$BUILDCONF" | grep -qE -- '--enable-(gpl|nonfree|version3)' && bad "configured with --enable-gpl/--enable-nonfree/--enable-version3" || ok "no GPL/nonfree configure flags"

ENCODERS="$("$FF" -hide_banner -encoders 2>&1)"
for enc in libsvtav1 h264_videotoolbox hevc_videotoolbox; do
  echo "$ENCODERS" | grep -qE "^ V[^ ]* $enc " && ok "encoder $enc" || bad "encoder $enc missing"
done
echo "$ENCODERS" | grep -qE ' lib(x264|x265|xvid|fdk_aac) ' && bad "GPL/nonfree encoder present" || ok "no libx264/libx265/libxvid/libfdk_aac encoder"

for f in "$DIR"/bin/* "$DIR"/lib/*.dylib; do
  [ -L "$f" ] && continue
  if otool -L "$f" 2>/dev/null | grep -qiE 'x264|x265|libpostproc'; then bad "$(basename "$f") links x264/x265/postproc"; fi
  if strings "$f" 2>/dev/null | grep -qE 'x264 - core|x265 \(build'; then bad "$(basename "$f") contains x264/x265 code"; fi
  # Only system libraries and our own @rpath libraries may be loaded.
  if otool -L "$f" 2>/dev/null | tail -n +2 | awk '{print $1}' | grep -vE '^(/usr/lib/|/System/Library/|@rpath/)' | grep -q .; then
    bad "$(basename "$f") loads a library from outside the bundle"
  fi
done
[ "$FAIL" -eq 0 ] && ok "binaries: no x264/x265, only system and bundled libraries"

for l in FFMPEG-COPYING.LGPLv2.1 SVT-AV1-LICENSE.md SVT-AV1-PATENTS.md CONFIGURE.txt SOURCES.txt; do
  [ -s "$DIR/LICENSES/$l" ] && ok "LICENSES/$l" || bad "LICENSES/$l missing"
done

echo
[ "$FAIL" -eq 0 ] && { echo "check-licence.sh: pass"; exit 0; }
echo "check-licence.sh: $FAIL failure(s)"; exit 1
