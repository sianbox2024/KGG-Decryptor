# Third-party notices

## Unlock Music algorithms

The offline NCM, KWM, QMC and KGM format implementations are compatible with
the algorithms published by the Unlock Music CLI/Web projects. Those projects
are distributed under the MIT License. This repository keeps the format
implementation self-contained in `tauri-app/src-tauri/src/formats.rs` and does
not bundle their runtime or any account/session data.

## FFmpeg

MP3/M4A/FLAC/WAV/OGG transcoding is performed by invoking an external
[FFmpeg](https://ffmpeg.org/) binary (not linked into this application).
Distribute only an **LGPL-licensed** FFmpeg build with this application; a
GPL-licensed build carries additional source-distribution obligations that must
be fulfilled separately. The encoder used for MP3 output is libmp3lame.

Use this application only with files and playback rights you already possess.
