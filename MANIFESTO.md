# rmpl Manifesto

## We Build With Purpose

Rust gave us memory safety without garbage collection. Yet our tooling remains shackled to conventions from another era. Cargo solved problems of 2015. We need tooling for 2025 and beyond.

**rmpl exists because we refused to accept that "this is how it's done" is a valid answer.**

## The Problems We Reject

### Over-Engineering
Cargo's complexity grows with every feature. Proc-macros, build scripts, custom targets, features flags cascading into dependency hell. We wanted to build a simple workspace, not negotiate with a build system.

### External Dependencies
You cannot build a language toolchain that depends on the language's package registry. It's circular. It's fragile. It's unnecessary. rmpl parses YAML with code written in Rust, not by calling out to external parsers.

### Monorepo Blindness
Cargo works beautifully for single crates. Ask it to coordinate a workspace with interdependent packages, proc-macros, and multiple targets, and you're managing workarounds. We work in monorepos. Our tools should too.

### Binary Distribution
Installing a Rust binary means downloading a tarball or compiling from source. Where's the `cargo install` for local workspaces? Why can't my workspace's binaries live in my PATH?

## What We Build Instead

### Simplicity First
```yaml
workspace:
  members: [rmpl, mylib, hello_macro]
  profiles:
    debug: { opt-level: 0 }
    release: { opt-level: 3 }
```

One file. One workspace. No ambiguity.

### Direct Control
rmpl calls `rustc` directly. No intermediary. No hidden steps. When you build with rmpl, you know exactly what commands run and why.

```
proc-macro → rustc --crate-type proc-macro → .so
library    → rustc --crate-type lib → .rlib
binary     → rustc --crate-type bin → executable
test       → rustc --test → run → report
```

### Zero Dependencies
The YAML parser? Hand-written. The dependency resolver? Our own. We don't pull in serde_yaml, clap, or any other crate just to build our own code. This is how tooling should be built—foundational, not dependent.

### Monorepo-Native
- Resolve workspace members and their dependencies
- Build in correct order (proc-macros first)
- Link packages together with `--extern`
- Test everything with a single command

### Developer Experience
```bash
rmpl build debug        # Build everything
rmpl test               # Run all tests
rmpl install            # Put binaries in ~/.rmpl/bin
rmpl test --filter foo  # Run specific test
```

Simple commands. Predictable behavior. No configuration needed.

## Our Philosophy

**Tooling should disappear.** You should think about your code, not your build system. rmpl gets out of your way.

**Simplicity is sophistication.** We don't avoid features—we avoid complexity. Every feature must earn its place.

**Build tools should be self-hosting.** rmpl builds itself. It's not a promise; it's proof the system works.

**We build for ourselves first.** If we can't use rmpl to build rmpl, it's not ready.

## The Future We're Building

- **Incremental compilation**: Don't rebuild what didn't change
- **Cross-package linking**: Binaries use libraries from workspace members
- **Foundation libraries**: rmpl-syn, rmpl-quote for macro programming
- **Benchmark support**: Measure performance, don't just guess
- **Plugin system**: Extend rmpl without modifying its core

## Join Us

rmpl is not finished. It's not even mostly finished. But it works. It builds our code. It runs our tests. It installs our binaries.

And it proves we can do better.

If you've ever stared at a Cargo.toml and wondered "why is this so complicated?"—this is for you.

If you work in a monorepo and wish your build tool understood that—this is for you.

If you believe tooling should be simple, transparent, and under your control—this is for you.

**Build with purpose.**

```bash
$ rmpl build debug
```

---

*rmpl: A YAML-based monorepo build tool for Rust*
