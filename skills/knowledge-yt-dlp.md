---
description: A YouTube video's title, channel, description, chapters or spoken words (auto subtitles) are needed for research, and the video must not be downloaded or watched.
---

`yt-dlp` is on PATH (witnessed 2026.07.04). Without it, `nix run nixpkgs#yt-dlp -- <args>` runs the same tool. It warns that a newer version exists; the warning is harmless.

Write every output into the scratchpad, never the repository: `cd` there first.

Metadata, one JSON object, no download:

    yt-dlp --skip-download --dump-single-json URL > meta.json

Read `title`, `channel`, `description`, `duration_string`, `webpage_url` and `chapters` (a list of `start_time`, `end_time`, `title`; empty when the video has none). `automatic_captions` lists the available subtitle languages.

Auto subtitles, written as `<id>.en.vtt` in the current directory:

    yt-dlp --skip-download --write-auto-subs --sub-langs en --sub-format vtt -o '%(id)s.%(ext)s' URL

Plain text from the vtt; auto captions repeat each line as they roll, so drop repeats:

    grep -vE '^(WEBVTT|Kind:|Language:|NOTE)|-->|^\s*$' ID.en.vtt | sed -E 's/<[^>]+>//g' | awk '!seen[$0]++' > transcript.txt

Description alone: `jq -r .description meta.json`.
