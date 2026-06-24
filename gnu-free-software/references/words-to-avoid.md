# GNU Vocabulary: Words to Avoid (or Use with Care)

Apply this when writing or auditing any prose a free-software project ships or
publishes: READMEs, manuals, web copy, release announcements, commit messages,
issue replies. Many common technical words are **loaded or confusing** — they
smuggle in a viewpoint the free-software movement rejects, or they blur a
distinction that matters. The fix is almost always a plainer, more precise word.

These corrections are about *the project's own voice*. Quoting someone else who
uses a loaded term is fine; describe and attribute it rather than adopting it.

## The single most important pair

- **Say "free software", never "open source."** "Open source" was coined
  specifically to drop the ethical framing — the user's freedom — and reframe the
  matter as a development methodology. A free-software project that calls itself
  "open source" undercuts its own point. (If you must name the broader category
  neutrally, name both movements explicitly rather than using the umbrella
  abbreviations "FOSS"/"FLOSS," which paper over the disagreement.)
- **Say "GNU/Linux", not "Linux", for the whole system.** "Linux" is the kernel
  Linus Torvalds began in 1991; the operating system is essentially GNU with Linux
  added. Call the kernel "Linux" and the whole system "GNU/Linux" — both to credit
  the GNU Project and to keep the kernel/system distinction clear. (So "LAMP"
  should really be "GLAMP.")

## Quick reference

| Avoid | Use instead | Why in one line |
|---|---|---|
| open source | free software | drops the freedom/ethics framing on purpose |
| closed (software) | nonfree, proprietary | "closed" is the open-source frame's opposite; we don't use it |
| FOSS / FLOSS | free software (or name both movements) | hides the ethical disagreement |
| Linux (the OS) | GNU/Linux | the kernel isn't the system |
| intellectual property | copyright, *or* patents, *or* trademarks | lumps unlike laws under a false property analogy |
| copyright owner | copyright holder | "owner" overstates the right |
| content | works, articles, writing, music, software… | treats works as interchangeable filler |
| creator (of authors) | author, writer, programmer | implicitly likens authors to a deity |
| consumer / consume | user, the public / use, run, read | software isn't used up; users aren't mouths |
| ecosystem | community | implies value-free observation, no ethical judgment |
| monetize | charge for, fund, earn from | "convert into money" implies giving nothing back |
| market (of users) | users, the public, the community | reframes a community as a sales target |
| vendor | supplier, developer, distributor | "vendor" presumes a seller, not e.g. a co-op |
| product (of free sw) | program, package, software | not everything free is a sold product |
| cloud / cloud computing | name the specific activity (hosting, SaaS, backup…) | a buzzword with no coherent meaning |
| SaaS (loosely) | the specific service, or SaaSS where it applies | conflates distinct setups; some mistreat users |
| piracy | unauthorized copying, sharing | equates copying with attacking ships |
| theft / stealing (of copies) | copyright infringement | copying isn't taking something away |
| for free / give away | release as free software | implies the issue is price, not freedom |
| freeware / freemium / free-to-play | free software (if it is) / gratis nonfree | "free" here means price, and usually isn't free |
| hacker (= intruder) | cracker, security breaker | a hacker is a playful clever programmer |
| steward (of free sw) | maintainer, contributor, developer | the EU CRA loads "steward" with legal duties |
| protection (DRM) | restriction, handcuffs, DRM | DRM restricts the user, it doesn't protect them |
| digital locks / digital rights mgmt | DRM, Digital *Restrictions* Management | "rights" inverts who is being restricted |
| trusted computing | treacherous computing | "trusted" = the computer obeys someone else, not you |
| RAND (patent licensing) | uniform-fee-only, or "restricted" | "reasonable/nondiscriminatory" hides that it bars free sw |
| Photoshop / PowerPoint / Skype (as verbs) | edit a photo / a presentation / a video call | don't make nonfree products into generic verbs |
| Google (as a verb) | search the web | same reason |
| MP3 player | audio player, portable player | privileges a patented, vulnerable format |
| ad-blocker | ad filter (or name what it blocks) | over-broad; conflates ads with surveillance/malware |
| alternative (free sw as) | free software (stated as the goal) | "alternative" accepts proprietary software as the norm |

## Entries that need more than a line

- **"Intellectual property."** This phrase glues together copyright, patents,
  trademarks, and more — bodies of law with little in common — and frames them all
  by analogy to physical property, which misleads on the central fact that
  information copies freely. Don't even think in these terms: name the specific
  law (copyright, patent, trademark) you actually mean.
- **"Creative Commons licensed."** CC publishes both free and nonfree licenses, so
  this phrase says nothing about whether a work is free. **Always name the specific
  license** ("licensed under CC BY-SA"), and find out which one if you don't know.
- **"Cloud computing."** It names no single activity — only "uses the internet for
  more than file transfer." Reason and write about the concrete thing instead:
  hosting, a backup service, remote storage, or *Service as a Software Substitute*
  (SaaSS), the setup where a network service does the user's own computing for
  them and thereby takes away their control.
- **"Commercial."** Don't use it as a synonym for "proprietary." Free software can
  be commercial (developed as a business), and nonfree software can be
  noncommercial. The axis that matters is free vs. nonfree, not the developer's
  business model.

## Note on "AI" and other vogue terms

Treat hype words skeptically. "Artificial intelligence" is often used to imply
understanding a program does not have; prefer concrete descriptions (a model, a
generator, a classifier) where accuracy matters. The standing rule generalizes: if
a word imports a marketing assumption or blurs whether software is free, replace it
with a plain, precise one.
