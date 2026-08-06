; SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
; SPDX-License-Identifier: GPL-3.0-or-later
;
; GENERATED FROM steelbore.toml — DO NOT EDIT.
;
; Regenerate with:
;     python3 .github/generate-steelbore-scm.py
; CI fails if this file drifts from the TOML (--check).
;
; The Guile mirror of the canonical Steelbore palette contract. It exists
; because Guix evaluates system configuration in plain Guile, which has no
; TOML reader — so Standard §11.6.5's obligations on Steelbore OS (GNU Guix
; System / Ginx) would otherwise require retyping values that §11.4 says are
; read, never retyped. Editing this file instead of the TOML defeats the
; entire point: there would be two sources, and they would drift.
;
; Palette contract version 3.2.0 — The Steelbore Standard §11 (v1.45).

(define-module (steelbore)
  #:export (steelbore-meta
            steelbore-themes
            steelbore-resolution
            steelbore-theme-ref
            steelbore-polarity
            steelbore-counterpart))

;;; Metadata (§11 [meta]).
(define steelbore-meta
  `((version . "3.2.0")
    (date . "2026-08-06T00:00:00Z")
    (standard . "The Steelbore Standard §11 (v1.45)")
    (default-theme . "steelbore")
    (default-dark-theme . "steelbore")
    (default-light-theme . "steelbore-navywhite")
    (mono-theme . "steelbore-mono")
    (palette-family . ("steelbore" "steelbore-classic" "steelbore-blue" "steelbore-blackpinkpanther" "steelbore-matrixgreen" "steelbore-navywhite" "tokyonight"))
    (fidelity-palettes . ("solarized-dark" "solarized-light"))
    ;; §11.6.1 — the themes every application MUST register.
    (registered-set . ("steelbore" "steelbore-high-contrast" "steelbore-blue" "steelbore-blue-high-contrast" "steelbore-blackpinkpanther" "steelbore-blackpinkpanther-high-contrast" "steelbore-matrixgreen" "steelbore-matrixgreen-high-contrast" "steelbore-navywhite" "steelbore-navywhite-high-contrast" "tokyonight" "tokyonight-high-contrast" "steelbore-mono"))))

;;; Every theme, as slug -> role alist. Role names are the §11.1 tokens
;;; verbatim; steelbore-mono binds 4-bit ANSI names rather than hex, and
;;; steelbore-classic binds only its legacy six roles (§11.2).
(define steelbore-themes
  `(
    ("solarized-dark"
     .
     (
      ("background" . "#002B36")
      ("surface" . "#073642")
      ("surface-alt" . "#073642")
      ("foreground" . "#839496")
      ("accent" . "#268BD2")
      ("structure" . "#6C71C4")
      ("success" . "#859900")
      ("error" . "#DC322F")
      ("warning" . "#B58900")
      ("focus" . "#2AA198")
      ("border" . "#6C71C4")
      ))
    ("solarized-light"
     .
     (
      ("background" . "#FDF6E3")
      ("surface" . "#EEE8D5")
      ("surface-alt" . "#EEE8D5")
      ("foreground" . "#657B83")
      ("accent" . "#268BD2")
      ("structure" . "#6C71C4")
      ("success" . "#859900")
      ("error" . "#DC322F")
      ("warning" . "#B58900")
      ("focus" . "#2AA198")
      ("border" . "#6C71C4")
      ))
    ("steelbore"
     .
     (
      ("background" . "#000027")
      ("surface" . "#0E2A47")
      ("surface-alt" . "#0B1A12")
      ("foreground" . "#D9DEE5")
      ("accent" . "#FF5E00")
      ("structure" . "#8A6CFF")
      ("success" . "#B4FF00")
      ("error" . "#FF3B3B")
      ("warning" . "#E445FF")
      ("focus" . "#B4FF00")
      ("border" . "#8A6CFF")
      ))
    ("steelbore-blackpinkpanther"
     .
     (
      ("background" . "#141418")
      ("surface" . "#1E1E22")
      ("surface-alt" . "#28282D")
      ("foreground" . "#FFFFFF")
      ("accent" . "#E445FF")
      ("structure" . "#FC8AFF")
      ("success" . "#5BE49B")
      ("error" . "#FF6B6B")
      ("warning" . "#FFC857")
      ("focus" . "#FDBBFF")
      ("border" . "#FC8AFF")
      ))
    ("steelbore-blackpinkpanther-high-contrast"
     .
     (
      ("background" . "#141418")
      ("surface" . "#1E1E22")
      ("surface-alt" . "#28282D")
      ("foreground" . "#FFFFFF")
      ("accent" . "#F07BFF")
      ("structure" . "#FC8AFF")
      ("success" . "#5BE49B")
      ("error" . "#FF8F8F")
      ("warning" . "#FFC857")
      ("focus" . "#FDBBFF")
      ("border" . "#FC8AFF")
      ))
    ("steelbore-blue"
     .
     (
      ("background" . "#0A1024")
      ("surface" . "#0F1728")
      ("surface-alt" . "#1B2436")
      ("foreground" . "#E6F0FF")
      ("accent" . "#0066FF")
      ("structure" . "#3390FF")
      ("success" . "#28C76F")
      ("error" . "#FF6B6B")
      ("warning" . "#FFC857")
      ("focus" . "#FF8A4B")
      ("border" . "#3390FF")
      ))
    ("steelbore-blue-high-contrast"
     .
     (
      ("background" . "#0A1024")
      ("surface" . "#0F1728")
      ("surface-alt" . "#1B2436")
      ("foreground" . "#E6F0FF")
      ("accent" . "#79B4FF")
      ("structure" . "#66A3FF")
      ("success" . "#28C76F")
      ("error" . "#FF8F8F")
      ("warning" . "#FFC857")
      ("focus" . "#FF8A4B")
      ("border" . "#66A3FF")
      ))
    ("steelbore-classic"
     .
     (
      ("background" . "#000027")
      ("foreground" . "#D98E32")
      ("accent" . "#4B7EB0")
      ("success" . "#50FA7B")
      ("error" . "#FF5C5C")
      ("info" . "#8BE9FD")
      ))
    ("steelbore-classic-high-contrast"
     .
     (
      ("background" . "#000027")
      ("foreground" . "#D98E32")
      ("accent" . "#7FAEDC")
      ("success" . "#50FA7B")
      ("error" . "#FF8080")
      ("info" . "#8BE9FD")
      ))
    ("steelbore-high-contrast"
     .
     (
      ("background" . "#000027")
      ("surface" . "#0E2A47")
      ("surface-alt" . "#0B1A12")
      ("foreground" . "#D9DEE5")
      ("accent" . "#FF8A3D")
      ("structure" . "#B3A1FF")
      ("success" . "#B4FF00")
      ("error" . "#FF7A7A")
      ("warning" . "#EE7BFF")
      ("focus" . "#B4FF00")
      ("border" . "#B3A1FF")
      ))
    ("steelbore-matrixgreen"
     .
     (
      ("background" . "#0C1A2B")
      ("surface" . "#1B2630")
      ("surface-alt" . "#05070A")
      ("foreground" . "#C7D2D9")
      ("accent" . "#B6FF3B")
      ("structure" . "#00F0FF")
      ("success" . "#00B39A")
      ("error" . "#FF6B6B")
      ("warning" . "#FFC857")
      ("focus" . "#B6FF3B")
      ("border" . "#00F0FF")
      ))
    ("steelbore-matrixgreen-high-contrast"
     .
     (
      ("background" . "#0C1A2B")
      ("surface" . "#1B2630")
      ("surface-alt" . "#05070A")
      ("foreground" . "#C7D2D9")
      ("accent" . "#B6FF3B")
      ("structure" . "#00F0FF")
      ("success" . "#2FD3BB")
      ("error" . "#FF8F8F")
      ("warning" . "#FFC857")
      ("focus" . "#B6FF3B")
      ("border" . "#00F0FF")
      ))
    ("steelbore-mono"
     .
     (
      ("background" . "default")
      ("foreground" . "default")
      ("accent" . "bright-white")
      ("structure" . "blue")
      ("success" . "green")
      ("error" . "red")
      ("warning" . "yellow")
      ("focus" . "reverse-video")
      ("border" . "default")
      ))
    ("steelbore-navywhite"
     .
     (
      ("background" . "#E7E5E0")
      ("surface" . "#F4F3F1")
      ("surface-alt" . "#D4D2CD")
      ("foreground" . "#111827")
      ("accent" . "#2A5580")
      ("structure" . "#1C2433")
      ("success" . "#2A6349")
      ("error" . "#93211F")
      ("warning" . "#6F4E0C")
      ("focus" . "#2A5580")
      ("border" . "#1C2433")
      ))
    ("steelbore-navywhite-high-contrast"
     .
     (
      ("background" . "#E7E5E0")
      ("surface" . "#F4F3F1")
      ("surface-alt" . "#D4D2CD")
      ("foreground" . "#111827")
      ("accent" . "#1F4A73")
      ("structure" . "#1C2433")
      ("success" . "#16452F")
      ("error" . "#7E1C1A")
      ("warning" . "#5A3F09")
      ("focus" . "#1F4A73")
      ("border" . "#1C2433")
      ))
    ("tokyonight"
     .
     (
      ("background" . "#1A1B26")
      ("surface" . "#24283B")
      ("surface-alt" . "#16161E")
      ("foreground" . "#C0CAF5")
      ("accent" . "#7AA2F7")
      ("structure" . "#BB9AF7")
      ("success" . "#9ECE6A")
      ("error" . "#F7768E")
      ("warning" . "#E0AF68")
      ("focus" . "#7DCFFF")
      ("border" . "#BB9AF7")
      ))
    ("tokyonight-high-contrast"
     .
     (
      ("background" . "#1A1B26")
      ("surface" . "#24283B")
      ("surface-alt" . "#16161E")
      ("foreground" . "#C0CAF5")
      ("accent" . "#97B6F9")
      ("structure" . "#BB9AF7")
      ("success" . "#9ECE6A")
      ("error" . "#F998AA")
      ("warning" . "#E0AF68")
      ("focus" . "#7DCFFF")
      ("border" . "#BB9AF7")
      ))
    ))

;;; §11.6 — system theme resolution. Slugs and paths only; never a color.
(define steelbore-resolution
  `((env-var . "SPACECRAFT_THEME")
    (system-file . "/etc/steelbore/theme.toml")
    (user-file . "$XDG_CONFIG_HOME/steelbore/theme.toml")
    (registry . "/etc/steelbore/themes.json")
    (polarity
     .
     (
      ("solarized-dark" . "dark")
      ("solarized-light" . "light")
      ("steelbore" . "dark")
      ("steelbore-blackpinkpanther" . "dark")
      ("steelbore-blue" . "dark")
      ("steelbore-classic" . "dark")
      ("steelbore-matrixgreen" . "dark")
      ("steelbore-navywhite" . "light")
      ("tokyonight" . "dark")
      ))
    (pair
     .
     (
      ("solarized-dark" . "solarized-light")
      ("solarized-light" . "solarized-dark")
      ("steelbore" . "steelbore-navywhite")
      ("steelbore-blackpinkpanther" . "steelbore-navywhite")
      ("steelbore-blue" . "steelbore-navywhite")
      ("steelbore-classic" . "steelbore-navywhite")
      ("steelbore-matrixgreen" . "steelbore-navywhite")
      ("steelbore-navywhite" . "steelbore")
      ("tokyonight" . "steelbore-navywhite")
      ))))

;;; Accessors. Each raises rather than returning #f for an unknown slug:
;;; §11.6.5 requires an OS to validate the slug at configuration-evaluation
;;; time, so an unknown theme fails the build rather than the boot.
(define (steelbore-theme-ref slug)
  "Return SLUG's role alist, or raise if it is not a registered theme."
  (or (assoc-ref steelbore-themes slug)
      (error "steelbore: unknown theme slug (see steelbore.toml)" slug)))

(define (steelbore-polarity slug)
  "Return \"light\" or \"dark\" for SLUG's canvas (§11.6.2)."
  (or (assoc-ref (assoc-ref steelbore-resolution 'polarity) slug)
      (error "steelbore: no polarity recorded for slug" slug)))

(define (steelbore-counterpart slug)
  "Return SLUG's counterpart in the other polarity (§11.6.2)."
  (or (assoc-ref (assoc-ref steelbore-resolution 'pair) slug)
      (error "steelbore: no light/dark counterpart recorded for slug" slug)))
