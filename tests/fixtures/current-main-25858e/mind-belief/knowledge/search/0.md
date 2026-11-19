---
description: Text or a file is searched for.
dependencies: []
---

Code and text under /git are searched with zoekt, an indexed search whose repeated queries answer in about 0.1 s where `rg` over the same tree takes seconds: `zoekt -index_dir ~/.local/share/zoekt/index 'text'`. A query is a regular expression by default; `f:name` restricts to file names, `r:repo` to a repository, `lang:rust` to a language, `sym:name` to a symbol, `case:yes` makes it case-sensitive, and terms combine with spaces (and), `or` and `-` (not). A timer re-indexes the HEAD of every repository under /git every 15 minutes.

`rg` keeps what zoekt cannot do: files not yet committed (the working tree, the flow directories, anything untracked), a repository changed since the last index run, and any directory outside /git. `rg -n 'text' <directory>`, with `-F` for a literal string and `-l` for file names alone. One repository answers in under 10 ms; the home directory holds a million and a half files and costs a minute of disk for every search.

A type filter narrows an `rg` search: `rg -t rust 'text'`, `rg -g '*.nix' 'text'`; `rg --type-list` names the types. rg reads the ignore files: `.gitignore`, and `.ignore` for rules of its own. It skips hidden files and does not follow symlinks, so `target/`, `.git/` and `result` stay out.

Code by its structure: `ast-grep`. Tracked files alone: `git grep -n 'text'`. A file by name: `fd -g 'name' <directory>`.

A file inside a Nix input is searched at that input's own store path, taken from the flake: `nix flake metadata --json`, or `nix eval` of the input's outPath.
