# Bundled FFmpeg (ADR-017, ADR-022)

CI fills this folder with `bin/` (ffmpeg, ffprobe), `lib/` (dylibs) and `LICENSES/`
from `tools/build-ffmpeg/build.sh`. Nothing here is committed except this file, so
non-macOS development builds still find the resource path.
