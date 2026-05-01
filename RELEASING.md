# 🚀 Releasing Spotlight

This guide explains how to use GitHub Actions to automatically build and release executables for Windows and Linux.

## Prerequisites

- Git and GitHub CLI (or web access)
- Commit rights to the repository

## Release Process

### 1. **Prepare Your Code**

Ensure all changes are committed and pushed to the repository:

```bash
git add .
git commit -m "feat: my new feature"
git push origin main
```

### 2. **Create a Release Tag**

Tags trigger automatic builds and releases. Use semantic versioning (v1.0.0, v1.1.0, etc.):

```bash
# Locally
git tag -a v1.0.0 -m "Initial release"
git push origin v1.0.0
```

Or via GitHub:
1. Go to your repository
2. Click "Releases" → "Create a new release"
3. Enter version: `v1.0.0`
4. Add release notes
5. Click "Publish release"

### 3. **Automatic Build Triggers**

Once you push a tag, GitHub Actions automatically:

1. ✅ **Builds for Linux** (Ubuntu latest, x86_64)
2. ✅ **Builds for Windows** (Windows latest, x86_64)
3. ✅ **Runs Tests** (CI on all commits)
4. ✅ **Checks Code Quality** (Clippy, Rustfmt)
5. 📦 **Creates Release** with downloadable binaries

### 4. **Monitor the Build**

1. Go to your repository → **Actions** tab
2. Click the latest workflow run
3. Watch as jobs complete:
   - `build` (Linux & Windows in parallel)
   - `test` (Quality checks)
   - `create-release` (Package artifacts)

### 5. **Download Executables**

After successful build:

1. Go to **Releases** tab
2. Find your release (v1.0.0)
3. Download:
   - `spotlight-linux-x86_64` (Linux executable)
   - `spotlight-windows-x86_64.exe` (Windows executable)

---

## What Each Workflow Does

### `release.yml` - Production Releases

**Triggers on:**
- Tag push (e.g., `git push origin v1.0.0`)
- Manual trigger (workflow_dispatch)

**Jobs:**
- **build**: Compiles for Linux and Windows (parallel)
- **test**: Runs tests and linting
- **create-release**: Packages binaries and creates GitHub Release

**Outputs:**
- Release artifacts on GitHub Releases page
- Auto-generated release notes
- Downloadable executables for both platforms

### `ci.yml` - Continuous Integration

**Triggers on:**
- Push to `main` or `develop`
- Pull requests to `main` or `develop`

**Jobs:**
- **check**: `cargo check`
- **test**: `cargo test`
- **fmt**: Rust formatting check
- **clippy**: Linting checks
- **cross-compile**: Verify Linux and Windows builds compile

**Purpose:** Ensure code quality on every push

---

## Version Numbering (Semantic Versioning)

Use [Semantic Versioning](https://semver.org/):

```
v1.2.3
 | | └─ Patch (bug fixes)
 | └─── Minor (new features, backward compatible)
 └───── Major (breaking changes)
```

Examples:
- `v0.1.0` - First release
- `v1.0.0` - Stable release
- `v1.1.0` - New feature
- `v1.1.1` - Bug fix
- `v2.0.0` - Breaking change

---

## Typical Release Workflow

```bash
# 1. Development cycle
git checkout -b feature/my-feature
# ... make changes ...
git add .
git commit -m "feat: add cool feature"
git push origin feature/my-feature

# 2. Merge to main
git checkout main
git merge feature/my-feature
git push origin main

# 3. Tag and release
git tag -a v1.1.0 -m "Add cool feature"
git push origin v1.1.0

# 4. GitHub Actions automatically builds
# 5. Download from Releases page
```

---

## Manual Build (Local)

If you need to build manually:

```bash
# Linux
cargo build --release --target x86_64-unknown-linux-gnu

# Windows (from Windows or with cross-compilation tools)
cargo build --release --target x86_64-pc-windows-msvc

# Outputs
# target/x86_64-unknown-linux-gnu/release/spotlight
# target/x86_64-pc-windows-msvc/release/spotlight.exe
```

---

## Troubleshooting

### Build Fails on Windows
- Windows MSVC toolchain might not be installed
- Ensure you run: `rustup target add x86_64-pc-windows-msvc`

### Build Fails on Linux
- Ensure Linux dependencies are installed
- The workflow handles this automatically via Ubuntu latest

### Release Not Created
- Tag must start with `v` (e.g., `v1.0.0`)
- Build must pass all steps
- Check Actions tab for error messages

---

## GitHub Actions Caching

Workflows use Cargo caching to speed up builds:
- Registry cache: `~/.cargo/registry`
- Git cache: `~/.cargo/git`
- Build cache: `target/`

Caches persist between runs for faster builds.

---

## Environment Variables

Workflows set:
- `CARGO_TERM_COLOR=always` - Colored output

---

## Next Steps

1. Commit these workflow files
2. Create your first tag: `git tag -a v0.1.0 -m "Initial release"`
3. Push: `git push origin v0.1.0`
4. Watch GitHub Actions build
5. Download from Releases! 🎉

---

For more info on GitHub Actions, see [GitHub Actions Documentation](https://docs.github.com/en/actions)
