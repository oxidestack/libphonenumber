# Upgrading

<!-- languages:begin -->
**English** · [Русский](../i18n/ru/guide/upgrading.md) · [中文](../i18n/zh/guide/upgrading.md) ·
[Español](../i18n/es/guide/upgrading.md)
<!-- languages:end -->

How to move a project from the libphonenumber release it depends on to a newer one. Find the section of every release newer
than yours, and apply them oldest first: going from 0.1.0 to a later release means the section of each release in
between, then the one of the release you want. A release with nothing to change says so. Then
[upgrade](#how-to-upgrade) to the newest release in one step: a release never needs an intermediate one first.

Each section lists what the release asks of your code: under **Action required**, a change to make before or with the
upgrade; under **Deprecated**, something that still works and will stop in a later release; under **Removed**,
something that has stopped. The [changelog](https://github.com/oxidestack/libphonenumber/blob/main/CHANGELOG.md) says
everything else each release changed. Only releases have a section: a release candidate (`-rc.N`) is described on its
GitHub release, and what it changes is written here under the release that ships it.

The documentation of the release you use is on this site too: the version picker at the top of each page switches
between releases, and a page of an older release links the same page in the latest.

## How to upgrade

The versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html). Before 1.0, a new minor version
(0.1 to 0.2) may change the API; a new patch version (0.1.0 to 0.1.1) does not.

1. Read the sections below for every release newer than yours, and make their changes to your code.
2. Change the version in your `Cargo.toml` (`libphonenumber = "0.2"`), or run `cargo update -p libphonenumber` for a patch release.
3. Run `cargo build` and your tests. A change listed under **Action required** that the compiler can catch shows up
   here; the others are behaviour, and the section says how to check for them.
4. A release that moves to newer libphonenumber metadata says so under **Action required**: a number's validity, type or format can change with the metadata, as it does in Java, so tests that pin such a result may need the new one.

## Upgrading to X.Y.Z (first release, version and date not yet set)

<!-- Placeholder: replace "X.Y.Z" in this heading and the Unreleased heading of CHANGELOG.md with the first release's
     number when it is cut (expected 0.1.0). Every later release gets its own section above this one. -->

The first release of libphonenumber: the parts of libphonenumber 9.0.41 (Java) that Datafaker 2.7.0 reaches, with libphonenumber 9.0.41's metadata. There is nothing to upgrade from.

### Action required

Nothing.

### Deprecated

Nothing.

### Removed

Nothing.
