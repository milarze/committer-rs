# committer-rs

A Rust implementation of [committer](https://github.com/Hyper-Unearthing/committer), a CLI tool that generates commit messages using Claude AI.

## Features

- Generates high-quality commit messages using the Claude API
- Opens your preferred text editor to review and edit the suggested commit message
- Supports customizable commit message scopes
- Follows common git editor patterns, respecting your git configuration

## Installation

```bash
# Clone the repository
git clone https://github.com/milarze/committer-rs.git
cd committer-rs

# Build the project
cargo build --release

# Move the binary to your PATH (optional)
cp target/release/committer-rs ~/.local/bin/
```

## Configuration

Configuration is stored in `~/.committer-rs/config.yml`:

```yaml
api_key: your_anthropic_api_key  # Optional: can also use ANTHROPIC_API_KEY environment variable
model: claude-3-7-sonnet-20250219  # Default model
scopes:  # Optional list of scopes for your commit messages
  - feat
  - fix
  - docs
  - style
  - refactor
  - test
  - chore
```

## Usage

1. Stage your changes with `git add`
2. Run `committer-rs` to generate a commit message
3. Review and edit the suggested message in your preferred text editor
4. Save and close the editor to complete the commit

## Requirements

- Rust 2021 edition or later
- An Anthropic API key

## License

See the [LICENSE](LICENSE) file for details.
