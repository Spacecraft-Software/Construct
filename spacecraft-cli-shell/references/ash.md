# ash — Almquist Shell (BusyBox / dash family)

ash is a small POSIX shell. You meet it as **BusyBox `ash`** (Alpine Linux,
Docker base images, embedded/initramfs) and, closely related, as **`dash`**
(Debian/Ubuntu `/bin/sh`). The skill detects it from `#!/bin/ash`; on Alpine,
`/bin/sh` *is* ash, so a `#!/bin/sh` script there runs under ash too.

> **ash is POSIX — so `references/posix-safe.md` is the real reference.** Write
> POSIX sh and it runs unchanged. This file records only the ash-specific
> quirks and the bash features ash lacks (which bite when someone drops a
> "`/bin/sh`" script that was secretly relying on bash).

Docs: BusyBox ash (<https://busybox.net/downloads/BusyBox.html#ash>), dash
manual (<https://manpages.debian.org/dash>).

## What ash gives you beyond strict POSIX

- **`local` in functions** — both BusyBox ash and dash support `local var`,
  so you can scope variables. (Strict POSIX `sh` does not guarantee it, but for
  an ash target it's safe.)
- **`test` / `[ ]`, `case`, `$(( ))`, `$( )`, heredocs, `trap`** — all present
  and behave as POSIX specifies.
- **`set -e`, `set -u`** — supported. `set -o pipefail` is **not** in dash and
  is only in newer BusyBox ash builds — do not rely on it; check exit status
  explicitly instead.

## What ash does NOT have (the bash trap)

These parse in bash but fail — often silently — under ash. If a script needs
one, either rewrite in POSIX or change the target to Bash:

- **`[[ … ]]`** — use `[ … ]` / `test`.
- **Arrays** (`arr=(a b c)`, `${arr[@]}`) — none. Use positional params, a
  space-separated string with `set --`, or iterate with `for`.
- **`${var^^}` / `${var,,}` / `${var//a/b}`** — use `tr` / `sed`.
- **Brace expansion** `{1..10}` and `{a,b,c}` — use `seq` or enumerate.
- **`function name { … }`** keyword form — use `name() { … }`.
- **`<<<` here-strings** and **`<(…)` process substitution** — use a pipe,
  `printf … |`, or a tempfile.
- **`**` globstar** — use `find`.
- **`$RANDOM` / `$SECONDS` / `$PIPESTATUS`** — not in dash; presence in BusyBox
  ash depends on build options. Use `awk`/`date +%s`/explicit status checks.

## BusyBox-specific gotchas

- **`echo` is unreliable** — BusyBox `echo` interprets backslash escapes and
  its `-e`/`-n` handling differs from GNU. Always use `printf '%s\n' "$x"` for
  anything but the most trivial literal output.
- **Applets, not coreutils** — `sed`, `grep`, `find`, `awk`, etc. are BusyBox
  reimplementations with a reduced flag set. A GNU-only flag (`grep -P`,
  `sed -i` with a backup suffix, `find -printf`) may be missing. Tool choice is
  `spacecraft-cli-preference`'s job; here, just prefer POSIX-defined options.
- **`/bin/sh` symlink** — on Alpine it points at BusyBox ash, not bash. A
  `#!/bin/sh` script authored against bash will break on Alpine. This is the
  single most common "works on my Ubuntu, fails in the Alpine container" cause.

## Shebang

```sh
#!/bin/sh
set -eu
```

Use `#!/bin/sh` for portability (runs under ash, dash, bash-in-POSIX-mode
alike). Use `#!/bin/ash` only when you specifically need BusyBox ash and know
it is installed at that path.

## Verify

`shellcheck -s sh script.sh` catches the bashisms above. For the strongest
signal, run the script under `dash` (or in a `busybox sh`) — both reject
bashisms that `bash` silently tolerates.
