#!/usr/bin/env bash
#
# Build the FFmpeg that Timelapse Studio ships (ADR-017, ADR-022): FFmpeg 9.0.2,
# LGPL-2.1-or-later only, shared libraries, with SVT-AV1 v4.2.0 for AV1 and
# VideoToolbox for H.264/HEVC. macOS (aarch64) only.
#
# Usage: tools/build-ffmpeg/build.sh <output-dir>
#   <output-dir>/bin/{ffmpeg,ffprobe}, <output-dir>/lib/*.dylib,
#   <output-dir>/LICENSES/*, <output-dir>/sources/*.tar.* (exact sources, LGPL §6)
#
# Every input is pinned by sha256; a mismatch aborts the build.
set -euo pipefail

OUT="${1:?usage: build.sh <output-dir>}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=pins.env
source "$HERE/pins.env"

[ "$(uname -s)" = "Darwin" ] || { echo "build.sh: macOS only (VideoToolbox)" >&2; exit 1; }

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
mkdir -p "$OUT"
OUT="$(cd "$OUT" && pwd)"
PREFIX="$WORK/prefix"
JOBS="$(sysctl -n hw.ncpu)"

fetch() { # url file sha256
  curl -sSfL --retry 3 -o "$WORK/$2" "$1"
  echo "$3  $WORK/$2" | shasum -a 256 -c -
}

fetch "$FFMPEG_URL" "$FFMPEG_TARBALL" "$FFMPEG_SHA256"
fetch "$SVTAV1_URL" "$SVTAV1_TARBALL" "$SVTAV1_SHA256"

# ---- SVT-AV1: shared library only, no command-line apps -------------------------
tar -xzf "$WORK/$SVTAV1_TARBALL" -C "$WORK"
cmake -S "$WORK/SVT-AV1-v$SVTAV1_VERSION" -B "$WORK/svt-build" \
  -DCMAKE_BUILD_TYPE=Release \
  -DBUILD_SHARED_LIBS=ON \
  -DBUILD_APPS=OFF \
  -DBUILD_TESTING=OFF \
  -DCMAKE_INSTALL_PREFIX="$PREFIX" \
  -DCMAKE_INSTALL_NAME_DIR=@rpath \
  -DCMAKE_OSX_DEPLOYMENT_TARGET="$MACOS_MIN"
cmake --build "$WORK/svt-build" -j "$JOBS"
cmake --install "$WORK/svt-build"

# ---- FFmpeg: LGPL only (no --enable-gpl / --enable-nonfree), nothing autodetected --
tar -xJf "$WORK/$FFMPEG_TARBALL" -C "$WORK"
CONFIGURE_ARGS=(
  --prefix="$PREFIX"
  --enable-shared --disable-static
  --disable-autodetect
  --disable-doc --disable-debug
  --disable-everything
  --enable-ffmpeg --enable-ffprobe --disable-ffplay
  --enable-swscale --enable-avfilter
  --enable-videotoolbox
  --enable-libsvtav1
  --enable-protocol=file,pipe
  --enable-demuxer=rawvideo,mov
  --enable-decoder=rawvideo
  --enable-muxer=mp4,mov
  --enable-encoder=libsvtav1,h264_videotoolbox,hevc_videotoolbox
  --enable-filter=scale,format,fps,null
  --install-name-dir=@rpath
  --extra-ldflags="-Wl,-rpath,@executable_path/../lib -mmacosx-version-min=$MACOS_MIN"
  --extra-cflags="-mmacosx-version-min=$MACOS_MIN"
)
(
  cd "$WORK/ffmpeg-$FFMPEG_VERSION"
  PKG_CONFIG_PATH="$PREFIX/lib/pkgconfig" ./configure "${CONFIGURE_ARGS[@]}"
  make -j "$JOBS"
  make install
)

# ---- Collect: binaries, dylibs, licences, exact sources ---------------------------
rm -rf "$OUT/bin" "$OUT/lib" "$OUT/LICENSES" "$OUT/sources"
mkdir -p "$OUT/bin" "$OUT/lib" "$OUT/LICENSES" "$OUT/sources"
cp "$PREFIX/bin/ffmpeg" "$PREFIX/bin/ffprobe" "$OUT/bin/"
# One real file per library, under the install name the binaries load (@rpath/<name>),
# so the bundle carries no symlinks and no duplicate copies.
for lib in "$PREFIX"/lib/*.dylib; do
  [ -L "$lib" ] && continue
  install_name="$(otool -D "$lib" | tail -n 1)"
  cp "$lib" "$OUT/lib/$(basename "$install_name")"
done
cp "$WORK/ffmpeg-$FFMPEG_VERSION/COPYING.LGPLv2.1" "$OUT/LICENSES/FFMPEG-COPYING.LGPLv2.1"
cp "$WORK/SVT-AV1-v$SVTAV1_VERSION/LICENSE.md" "$OUT/LICENSES/SVT-AV1-LICENSE.md"
cp "$WORK/SVT-AV1-v$SVTAV1_VERSION/PATENTS.md" "$OUT/LICENSES/SVT-AV1-PATENTS.md"
{
  echo "FFmpeg $FFMPEG_VERSION, configured with:"
  printf '  %s\n' "${CONFIGURE_ARGS[@]}" | sed "s|$PREFIX|<prefix>|g"
  echo
  echo "SVT-AV1 $SVTAV1_VERSION: cmake -DBUILD_SHARED_LIBS=ON -DBUILD_APPS=OFF -DCMAKE_BUILD_TYPE=Release"
} > "$OUT/LICENSES/CONFIGURE.txt"
{
  echo "Exact sources of the FFmpeg shipped with Timelapse Studio (unmodified):"
  echo "  $FFMPEG_URL"
  echo "    sha256 $FFMPEG_SHA256"
  echo "  $SVTAV1_URL"
  echo "    sha256 $SVTAV1_SHA256"
  echo "Copies of both tarballs are published with each release of the app."
} > "$OUT/LICENSES/SOURCES.txt"
cp "$WORK/$FFMPEG_TARBALL" "$WORK/$SVTAV1_TARBALL" "$OUT/sources/"

echo "build.sh: FFmpeg $FFMPEG_VERSION + SVT-AV1 $SVTAV1_VERSION in $OUT"
