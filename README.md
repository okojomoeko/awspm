# AWS Profile Manager (awspm)

![Build Status](https://img.shields.io/github/actions/workflow/status/okojomoeko/awspm/ci.yml?branch=main)
![License](https://img.shields.io/github/license/okojomoeko/awspm)

> 🚀 **Created via Vibe Coding by Gemini & Antigravity**

`awspm` is a Rust-based CLI tool to manage, search, and identify AWS profiles efficiently. It allows attaching metadata (tags, aliases, notes) to your profiles without modifying `~/.aws/config`.

## Features

- **Metadata Management**: Store tags, aliases, and notes in `~/.awspm.yaml`.
- **Sync**: Automatically detect new profiles in `~/.aws/config` and register them.
- **Search**: Interactive fuzzy search using `skim` (Unix) or `dialoguer` (Windows) to find profiles by name, tag, or alias.
- **Reverse Lookup**: Find the profile name by searching for its attributes.
- **Context Awareness**: Easily get the current profile name for scripts.
- **Command Execution**: Run single commands within a profile context `awspm exec`.
- **Region Override**: Force specific regions for legacy tools via `AWS_REGION` injection.
- **Team Configuration**: Share alias and tag settings via `.awspm.yaml` committed to your repo.
- **Production Guardrails**: Protect sensitive profiles ("production", "prod") with confirmation prompts.

## Installation

### Options

#### 1. Homebrew (macOS / Linux)
Coming soon! You will be able to install via our tap:
```bash
brew install okojomoeko/tap/awspm
```

### Shell (Linux / macOS)
```bash
curl -fsSL https://raw.githubusercontent.com/okojomoeko/awspm/main/install.sh | sh
```

### Windows Support
Windows support is currently in development and planned for a future release.

### Cargo
```bash
cargo install --git https://github.com/okojomoeko/awspm
```

#### 2. Binary Download (Manual)
Download the latest binary for your platform from [GitHub Releases](https://github.com/okojomoeko/awspm/releases).
1. Download the file (e.g., `awspm-macos-amd64`).
2. Make it executable: `chmod +x awspm-macos-amd64`
3. Move it to your PATH: `sudo mv awspm-macos-amd64 /usr/local/bin/awspm`

#### 3. From Source (Rust Developers)
```bash
cargo install awspm
# or for local development:
cargo install --path .
```

*Note: The package and binary are now named `awspm`.*

## Usage

### 1. Initialize

First, create the metadata storage file:

```bash
awspm init
```

### 2. Sync with AWS Config

Read your existing `~/.aws/config` and populate the metadata store. This is safe to run multiple times; it adds new profiles but keeps your existing metadata.

```bash
awspm sync
```

**New in v0.1.0:**
- `awspm` now loads profiles from `~/.aws/credentials` and `AWS_PROFILE` env var, in addition to `~/.aws/config`.
- `awspm sync --check`: Check for sync status without modifying metadata.
- `awspm sync`: Interactively add "untracked" profiles (in AWS but not metadata) and remove "orphaned" profiles (in metadata but not AWS).

**Environment Variables Precedence:**
If you set `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, etc. directly in your shell, `awspm` detects this as a special **`env-vars`** profile. This takes precedence over `AWS_PROFILE` or `awspm switch`.

```bash
export AWS_ACCESS_KEY_ID=...
awspm current
# Output: env-vars
```

### 3. Switch Profile (Subshell)

Launch a new shell with the `AWS_PROFILE` environment variable set. This is the recommended way to "enter" a profile context.

```bash
# Interactive selection
awspm switch

# Direct switch
awspm switch production-db
```

*(Type `exit` or press `Ctrl-D` to return to your previous shell)*

### 4. Execute Command

Execute a command with the specified profile's credentials.
This command unsets `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, and `AWS_SESSION_TOKEN` from the child process to ensure the specified profile takes precedence.

```bash
# S3 list using 'dev' profile
awspm exec dev -- aws s3 ls

# With additional arguments
awspm exec dev -- echo "Running in $(aws configure get region)"
```

**Safety Check:** If the target profile is sensitive (e.g., tagged `production`), `awspm` will ask for confirmation.

### 5. Smart Profile Switching (direnv)

If you use `direnv`, you can **pin** a profile to the current directory. This creates (or updates) an `.envrc` file.

```bash
awspm pin my-api-profile
# Then run: direnv allow
```

Now, whenever you `cd` into this directory, `AWS_PROFILE` will be automatically set.

### 6. List Profiles

List all profiles in a rich table with colors:

```bash
awspm list
```

**Filtering:**
You can filter the list using keywords or specific field targets:

```bash
# Simple fuzzy match (searches name, alias, tags, notes)
awspm list --query "prod"

# Filter by Tag
awspm list --query "tag:production"

# Filter by Alias
awspm list --query "alias:db"

# Filter by Name
awspm list --query "name:admin"
```

**Compact Mode:**
For a cleaner view showing only names and aliases:

```bash
awspm list --short
```

### 7. Search & Select (Interactive)

Launch an interactive fuzzy finder to select a profile. You can pass an initial query to pre-filter results.
**Note:** `awspm` automatically sorts results by "Last Used" time, placing your most frequently used profiles at the top.

```bash
# Interactive search
awspm search

# Search specifically in tags
awspm search --target tags
```

**Production Guardrails:**
If you select a profile tagged with `production`, `prod`, or `live`, `awspm` will ask for confirmation before printing the result or switching. This acts as a safety net against accidental modification of critical environments.

**Select Alias:**
`awspm select` is an alias for `awspm search`, convenient for shell integration.

```bash
export AWS_PROFILE=$(awspm select "prod")
```

### 8. Update Metadata

Add tags, aliases, or notes to a profile directly from the CLI:

```bash
# Add a tag
awspm update my-profile --add-tag production

# Add an alias
awspm update my-profile --add-alias prod-db

# Set a note
awspm update my-profile --set-note "Do not touch"

# Unset a note
awspm update my-profile --unset-note
```

### 9. Shell Completion

Generate completion scripts for your shell (bash, zsh, fish, etc.):

```bash
# For Zsh (add to .zshrc)
source <(awspm completion zsh)

# For Bash
source <(awspm completion bash)
```

### 10. Get Current Profile

Get the active profile name (from `AWS_PROFILE`):

```bash
awspm current
```

Example usage:

```bash
git clone codecommit://$(awspm current)@MyRepo
```

### 11. Region Override (Default Region)

You can specify a default region for a profile that overrides `~/.aws/config` settings. This is useful when you want to work in a specific region without modifying the global configuration.

```bash
# Set a region override
awspm update my-profile --set-region us-west-2

# The override is applied automatically
awspm exec my-profile -- env | grep AWS_REGION
# AWS_REGION=us-west-2
```

### 12. Project Configuration (Shareable Config)

You can share project-specific profile settings (like aliases and tags) using a `.awspm.yaml` file in your project directory.

**Secure Sharing with Aliases:**

To avoid committing sensitive Account IDs or Profile Names to git, use **Alias Resolution**:

1. **Global Setup**: Map your real profile to a shared alias (once).
    ```bash
    awspm update my-real-profile-id --add-alias project-prod
    ```

2. **Project Config (`./.awspm.yaml`)**: Use the alias as the key.
    ```yaml
    profiles:
      project-prod:
        region: eu-central-1
        tags: ["team-shared"]
    ```

3. **Usage**: When you run `awspm` in this directory, it resolves `project-prod` to `my-real-profile-id` and applies the settings.
    ```bash
    # You can use the alias 'project-prod' directly in commands
    awspm exec project-prod -- aws s3 ls
    ```

**Note on Alias Resolution:**

All commands that accept a profile name (`current`, `exec`, `pin`, `switch`, `update`) now support **Alias Resolution**.
`awspm` prioritizes exact profile name matches. If no match is found, it searches for a unique alias.
If multiple profiles share the same alias, `awspm` will report an ambiguity error.

### 13. Debugging

If you encounter issues or want to see what `awspm` is doing (e.g., config loading details, SSO cache checks):

```bash
awspm --verbose list
```

## SSO Integration

`awspm` automatically checks the AWS SSO cache (`~/.aws/sso/cache`) to display session status in `awspm list`:
- 🟢 **Active**: Session is valid.
- 🔴 **Expired**: Session has expired.
- ⚪ **Not Configured**: No SSO session defined for this profile.
- ❓ **Unknown**: Cache file missing or unreadable.

## Configuration

Metadata is stored in `~/.awspm.yaml`. You can verify it manually or edit it if needed (though `awspm update` is recommended).

```yaml
profiles:
  my-profile:
    aliases:
      - "prod-db"
    tags:
      - "production"
      - "database"
    note: "Do not touch"
```

## Contributing

We welcome contributions! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our development process, coding standards, and testing requirements.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
