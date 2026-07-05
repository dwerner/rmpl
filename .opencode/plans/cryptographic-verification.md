# Plan: Cryptographic Verification for rmpl

## Overview
Build SHA256 hash verification into the rmpl build/install process to validate binary integrity without external dependencies.

## Use Cases
1. **Build verification**: Generate hash after compilation
2. **Install verification**: Verify hash before installing
3. **Cache validation**: Skip installation if hash matches
4. **Integrity checks**: `rmpl verify` command to check installed binaries

## Architecture

### Hash Storage
```
~/.rmpl/cache/
  <binary-name>.sha256    # Expected hash
  <binary-name>           # Cached binary (optional)

target/<profile>/
  deps/
    <binary>.sha256       # Build hash
```

### Commands

#### `rmpl build` (enhanced)
- Generate SHA256 hash after successful compilation
- Store hash alongside binary in target directory
- Hash file format: `<hash>  <binary-name>`

#### `rmpl install` (enhanced)
- Verify binary hash before copying
- Store expected hash in `~/.rmpl/cache/`
- Support `--verify` flag to check existing installs

#### `rmpl verify` (new)
```bash
rmpl verify              # Verify all installed binaries
rmpl verify <binary>     # Verify specific binary
rmpl verify --workspace  # Verify workspace build hashes
```

## Implementation Steps

### Phase 1: Hash Generation (build.rs)
1. Add `generate_hash()` function using `sha256sum` command
2. After successful compilation, generate hash for binary
3. Write hash to `<target>/<binary>.sha256`
4. Hash format matches sha256sum output: `<64-char-hash>  <filename>`

### Phase 2: Hash Verification (install.rs)
1. Add `verify_hash()` function
2. Before copying binary, verify its hash matches expected
3. Store hash in `~/.rmpl/cache/<binary>.sha256` on install
4. Add `--verify` flag to `rmpl install`

### Phase 3: Verify Command (new verify.rs)
1. New module `verify.rs`
2. `rmpl verify` - Check all installed binaries against cached hashes
3. `rmpl verify --workspace` - Check workspace build hashes
4. Exit code 0 if all pass, 1 if any fail
5. Output format:
   ```
   ✓ rmpl (verified)
   ✓ demo (verified)
   ✗ outdated (hash mismatch)
   ```

### Phase 4: Cache Management (new cache.rs)
1. `rmpl cache clean` - Remove cached hashes
2. `rmpl cache list` - Show cached binaries and hashes
3. Auto-cleanup old hashes on install

## Technical Details

### Hash Generation
```rust
fn generate_hash(binary_path: &Path) -> Result<String, String> {
    let output = Command::new("sha256sum")
        .arg(binary_path)
        .output()?;
    
    let stdout = String::from_utf8(output.stdout)?;
    let hash = stdout.split_whitespace().next().unwrap_or("").to_string();
    Ok(hash)
}
```

### Hash Storage Format
```
# ~/.rmpl/cache/rmpl.sha256
a1b2c3d4...  rmpl

# target/debug/deps/rmpl.sha256
a1b2c3d4...  rmpl
```

### Verification Logic
```rust
fn verify_hash(binary_path: &Path, expected_hash_path: &Path) -> bool {
    let expected = read_hash_file(expected_hash_path)?;
    let actual = generate_hash(binary_path)?;
    expected == actual
}
```

## Dependencies
- **None** - Uses system `sha256sum` (available on all Unix systems)
- No external crates needed

## File Changes

### New Files
- `src/bin/verify.rs` - Verify command implementation
- `src/bin/cache.rs` - Cache management commands

### Modified Files
- `src/bin/build.rs` - Add hash generation after compilation
- `src/bin/install.rs` - Add hash verification and caching
- `src/bin/main.rs` - Add verify/cache commands

## Commands After Implementation

```bash
# Build with hash generation
rmpl build debug
# Creates: target/debug/deps/rmpl.sha256

# Install with verification
rmpl install debug --verify
# Verifies hash, stores in ~/.rmpl/cache/

# Verify installed binaries
rmpl verify
# ✓ rmpl (verified)
# ✓ demo (verified)

# Verify workspace builds
rmpl verify --workspace
# ✓ rmpl (build verified)
# ✓ mylib (build verified)

# Cache management
rmpl cache list
rmpl cache clean
```

## Edge Cases
1. **Missing hash file**: Treat as unverified, generate new hash
2. **Hash mismatch**: Fail install, report error
3. **No sha256sum**: Fallback to manual hash computation (pure Rust)
4. **Binary modified after build**: Detect via verification

## Future Enhancements
- **Reproducible builds**: Compare hashes across builds to detect non-determinism
- **Signature verification**: Add GPG signing for distribution (future)
- **Hash algorithms**: Configurable (SHA256, SHA512, BLAKE3)

## Timeline
- Phase 1: 30 min (hash generation in build)
- Phase 2: 30 min (verification in install)
- Phase 3: 45 min (verify command)
- Phase 4: 15 min (cache management)
- Testing: 30 min
- **Total**: ~2.5 hours

## Questions
1. Should we store hashes in `~/.rmpl/cache/` or `~/.rmpl/hashes/`?
2. Should `rmpl verify` check workspace builds or just installed binaries?
3. Do you want hash generation to be opt-in or always on?
