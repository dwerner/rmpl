## Alternative timeline rust build tool

Research/playground tool that replaces cargo for rust builds.

```
rmpl - YAML-based monorepo build tool

Usage: rmpl <command> [options]

Commands:
  build [debug|release]  Build the workspace (default: debug)
  install [debug|release]  Install binaries to ~/.rmpl/bin
                         Options: --force, -f
  test [--filter NAME]   Run tests (supports --filter for test name)
  help                   Show this help message
```
