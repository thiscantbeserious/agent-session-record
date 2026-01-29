#!/usr/bin/env bash
# Generate changelog grouped by src/ module based on actual file paths changed
# Each commit appears in only ONE module (the one with most files touched)
# Usage: ./scripts/generate-changelog.sh [--unreleased | --tag vX.Y.Z]
# Requires: git-cliff, git

set -e

MODE="${1:---unreleased}"
TAG="${2:-}"

# Module groups for changelog (consolidated for readability)
# Maps individual modules to display groups
declare_module_group() {
    case "$1" in
        tui) echo "tui" ;;
        player|terminal) echo "player" ;;
        asciicast) echo "asciicast" ;;
        cli|commands|shell) echo "cli" ;;
        analyzer|branding|config|recording|storage|lib|main) echo "core" ;;
        *) echo "$1" ;;
    esac
}

# All src modules to check (for file detection)
MODULES="tui player terminal asciicast commands analyzer branding cli config recording shell storage"

# Display groups in order
DISPLAY_GROUPS="tui player asciicast cli core"

# Format group name for display
format_group() {
    case "$1" in
        tui) echo "TUI" ;;
        cli) echo "CLI" ;;
        core) echo "Core" ;;
        player) echo "Player" ;;
        asciicast) echo "Asciicast" ;;
        tests) echo "Tests" ;;
        docs) echo "Documentation" ;;
        *) echo "$1" | awk '{print toupper(substr($0,1,1)) tolower(substr($0,2))}' ;;
    esac
}

# Get the primary module for a commit based on file count
# Priority: src/ modules > commit scope fallback > tests/docs
get_primary_module() {
    local sha="$1"
    local scope="$2"
    local max_count=0
    local primary=""

    # Get files changed in this commit
    local files=$(git diff-tree --no-commit-id --name-only -r "$sha" 2>/dev/null)

    # First pass: count src/ modules (directories and single files)
    for mod in $MODULES; do
        local count=0
        # Check if it's a directory module
        if [[ -d "src/$mod" ]]; then
            local dir_count=$(echo "$files" | grep -c "^src/$mod/" 2>/dev/null)
            count=${dir_count:-0}
        fi
        # Also check for single-file module (src/mod.rs)
        local file_count=$(echo "$files" | grep -c "^src/$mod.rs$" 2>/dev/null)
        file_count=${file_count:-0}
        count=$((count + file_count))

        if [[ $count -gt $max_count ]]; then
            max_count=$count
            primary="$mod"
        fi
    done

    # Fallback: use commit scope if no src/ module found
    if [[ -z "$primary" && -n "$scope" ]]; then
        # Only use if it's a valid module
        if echo "$MODULES" | grep -qw "$scope"; then
            primary="$scope"
        fi
    fi

    # Last resort: tests/docs only if still nothing
    if [[ -z "$primary" ]]; then
        local test_count=$(echo "$files" | grep -c "^tests/" 2>/dev/null || echo 0)
        if [[ $test_count -gt 0 ]]; then
            primary="tests"
        fi

        local docs_count=$(echo "$files" | grep -c "^docs/" 2>/dev/null || echo 0)
        if [[ $docs_count -gt $test_count ]]; then
            primary="docs"
        fi
    fi

    echo "$primary"
}

# Get all relevant commits
get_commits() {
    local cliff_args=""
    if [[ "$MODE" == "--unreleased" ]]; then
        cliff_args="--unreleased"
    elif [[ "$MODE" == "--tag" ]]; then
        cliff_args="--tag $TAG"
    fi

    # Get commits with their SHAs, scopes, and messages
    # Format: sha|scope|message
    # Filter: only lines starting with a commit SHA (40 hex chars)
    git cliff $cliff_args --body "{% for commit in commits %}{{ commit.id }}|{{ commit.scope | default(value='') }}|{{ commit.message | upper_first }}
{% endfor %}" 2>/dev/null | grep -E "^[a-f0-9]{40}\|" || true
}

# Build module -> commits mapping
declare_module_commits() {
    local commits="$1"

    # Temp files for each display group
    for group in $DISPLAY_GROUPS tests docs; do
        echo "" > "/tmp/changelog_$group.txt"
    done

    # Process each commit (format: sha|scope|message)
    while IFS='|' read -r sha scope message; do
        [[ -z "$sha" ]] && continue
        local primary=$(get_primary_module "$sha" "$scope")
        if [[ -n "$primary" ]]; then
            # Map to display group
            local group=$(declare_module_group "$primary")
            echo "- $message" >> "/tmp/changelog_$group.txt"
        fi
    done <<< "$commits"
}

# Generate changelog
generate_changelog() {
    echo "# Changelog"
    echo ""
    echo "All notable changes to this project will be documented in this file."
    echo "Grouped by module based on files changed (each commit in primary module only)."
    echo ""

    if [[ "$MODE" == "--unreleased" ]]; then
        echo "## [Unreleased]"
    elif [[ "$MODE" == "--tag" ]]; then
        echo "## [$TAG] - $(date +%Y-%m-%d)"
    fi
    echo ""

    # Get and process commits
    local commits=$(get_commits)
    declare_module_commits "$commits"

    # Output each group's commits
    for group in $DISPLAY_GROUPS tests docs; do
        local content=$(cat "/tmp/changelog_$group.txt" 2>/dev/null | grep -v "^$" | sort -u)
        if [[ -n "$content" ]]; then
            echo "### $(format_group "$group")"
            echo "$content"
            echo ""
        fi
    done

    # Cleanup
    rm -f /tmp/changelog_*.txt
}

generate_changelog
echo "Done." >&2
