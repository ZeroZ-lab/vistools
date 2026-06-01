# vistools Skills

AI agent skill definitions for vistools — so agents know **when** and **how** to use the CLI.

## Installation

### Claude Code (Plugin - Recommended)

Install as a Claude Code plugin (auto-installs CLI binary):

```bash
/plugin install vistools
```

Or from GitHub:

```bash
/plugin install https://github.com/zhengjianqiao/vistolls
```

Or from local path:

```bash
/plugin install ./path/to/vistolls
```

**Usage:**
```
/vistools screenshot.png
/vistools large-image.jpg "focus on the header"
/vistools design.png "extract the login button"
```

### Cursor (Manual)

Copy the rule file to `.cursor/rules/`:

```bash
cp skills/cursor/vistools.mdc .cursor/rules/
```

### Codex (Manual)

Append to your project's AGENTS.md:

```bash
cat skills/codex/AGENTS.md >> AGENTS.md
```

## What's Included

### Claude Code Plugin (`skills/claude-code/`)

Single skill that handles all vistools workflows:

| Invocation | Use Case |
|-----------|----------|
| `/vistools <path>` | Auto-detect: inspect → overview (if needed) → analyze |
| `/vistools <path> "focus on X"` | Targeted analysis with focus area |
| `/vistools <path> "extract region"` | Crop and coordinate mapping |

**Features:**
- ✅ Auto-installs CLI binary via Setup hook
- ✅ `vistools` added to PATH via bin/
- ✅ Structured JSON output with coordinate mapping
- ✅ Error handling with fix suggestions

### Cursor Rule (`skills/cursor/`)

Auto-attached rule that activates when working with images:
- Triggers on: `.png`, `.jpg`, `.jpeg`, `.webp`, `.gif`, `.bmp`, `.tiff`
- Provides command reference and workflow guidance

### Codex Instructions (`skills/codex/`)

Project-level instructions for Codex CLI:
- Append to AGENTS.md for persistent context
- Includes command reference and best practices

## Plugin Structure

```
.claude-plugin/
└── plugin.json          # Plugin manifest

skills/
├── claude-code/
│   └── vistools/
│       └── SKILL.md    # Unified skill (all workflows)
├── cursor/
│   └── vistools.mdc    # Cursor rule
└── codex/
    └── AGENTS.md       # Codex instructions
```

## How It Works

1. **Plugin install** → Claude Code downloads the repo
2. **User installs CLI** → `git clone ... && cargo install --path crates/cli`
3. **User invokes** → `/vistools screenshot.png`
4. **Skill loads** → Agent runs `vistools inspect` → decides workflow
5. **Structured output** → JSON with coordinate mapping for back-reference

## Manual CLI Installation

```bash
git clone https://github.com/zhengjianqiao/vistolls
cd vistolls
cargo install --path crates/cli   # installs to ~/.cargo/bin/vistools
vistools --version
```

## Troubleshooting

### "vistools: command not found"

Install the CLI manually:

```bash
cargo install --path /path/to/vistolls/crates/cli
```

### "Build failed" errors

Ensure Rust is installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### "Permission denied" on install

The script tries `/usr/local/bin/` first (may need sudo), then falls back to `~/.local/bin/`. Add to PATH if needed:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

## Uninstall

### Claude Code Plugin

```bash
/plugin uninstall vistools
```

### Manual Cleanup

```bash
# Remove CLI binary
cargo uninstall vistools
```

## Development

To test the plugin locally:

```bash
# Create a test project
mkdir test-project && cd test-project

# Install plugin from local path
/plugin install ../path/to/vistolls

# Test invocation
/vistools ../path/to/test-image.png
```

## License

MIT OR Apache-2.0
