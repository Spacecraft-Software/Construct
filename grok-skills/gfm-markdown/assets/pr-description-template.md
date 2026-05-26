## Summary

<!-- One paragraph summary of the change. What problem does this solve? -->

This PR adds support for GFM alerts in the Markdown renderer and updates all documentation templates.

## Motivation

<!-- Why is this change needed? Link to issue if applicable. -->

Closes #123
Related to #456

## Changes

<!-- Bullet list of what was changed. Use task list for checklist items. -->

- [x] Added alert parsing for `> [!NOTE]`, `> [!TIP]`, etc.
- [x] Updated `README-template.md` with new examples
- [x] Added unit tests for all five alert types
- [ ] Updated user-facing docs (will be done in follow-up PR)

## Testing

<!-- How was this tested? Include commands or screenshots if relevant. -->

```bash
npm test
# All 142 tests passing
```

**Screenshots / Demos**

> [!TIP]
> Before/after comparison available in the attached video (internal link).

## Checklist

- [x] Code follows project style guide
- [x] Self-review completed
- [x] No new dependencies introduced
- [x] Documentation updated where needed
- [ ] CI pipeline green (pending final review)

## Breaking Changes

<!-- List any breaking changes or say "None" -->

None.

## Additional Context

<!-- Any other relevant information for reviewers. -->

Thanks to @reviewer1 for the initial feedback on alert syntax!