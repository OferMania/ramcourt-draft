# Contributing to RamCourt

RamCourt is an early-stage, actively evolving distributed key-value store.
Right now it's primarily developed by a single maintainer, so response
time on issues and PRs may be slow — that's expected at this stage, not
a signal the project is unmaintained.

## Reporting issues
Open a GitHub issue with a clear description and, where relevant, steps
to reproduce.

## Submitting changes
- Open a pull request against `main`.
- New behavior should be covered by an appropriate test suite (see the [clstr-rc](https://github.com/OferMania/clstr-rc)
  test harness currently used for correctness testing). If you decide
  to use a different test suite than clstr-rc, please document which
  one you are using clearly in your PR.
- By submitting a contribution, you agree it's licensed under this
  project's [Apache 2.0 license](./LICENSE), consistent with the rest
  of the codebase.

## Design philosophy
RamCourt prioritizes correctness under concurrent access. See the
README for the version-verification model that's central to its design.
Changes that weaken these guarantees need strong justification.
