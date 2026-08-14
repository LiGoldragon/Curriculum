Repo: `/git/github.com/LiGoldragon/lojix`. Ordinary contract: `signal-lojix`. Owner contract: `meta-signal-lojix`.

One daemon (`lojix-daemon`), two thin CLIs (`lojix`, `meta-lojix`), one per authority tier — ordinary socket for queries/watches, owner/meta socket for deploy/pin/unpin/retire/test. This triad is the standard component architecture.

Each CLI takes exactly one positional argument: an inline DOTOS/NOTA object. No flags, no subcommands — `--help` is rejected. The request type is the discriminator. `dotos-text` must be compiled in.

`lojix '(Query ...)'` goes to the ordinary socket.
`meta-lojix '(Deploy ...)'` goes to the owner/meta socket.
`DeployAccepted` is admission, not completion.

Socket paths have no defaults. `LOJIX_ORDINARY_SOCKET` and `LOJIX_OWNER_SOCKET` come from environment. CriomOS configures them through `services.lojix`.

`lojix-write-configuration` is the only DOTOS-to-startup boundary — writes an rkyv archive. The daemon rejects inline DOTOS at startup.

Store uses SEMA tables, schema v4. Refuses earlier schemas; no migration. `lojix-reset-store` resets v2/v3 to v4 (removes and recreates). Past database is disposable.

Deploy transport is explicit: `nix_store_uri` and `ssh_destination` per request. Lojix never derives addresses from names. Proposals are non-symlink absolute regular `.dotos` files.

`lojix-bootstrap` is the daemon-free surface for initial setup. Strict SSH policy — caller-owned identity and known-hosts, no ambient SSH.

Lojix belongs only in OS configuration, not home-environment. The interface is `lojix` and `meta-lojix` CLI only — no setup-specific scripts. Meta-signal is not optional.
