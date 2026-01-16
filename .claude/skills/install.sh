#!/bin/bash
# Install COSMIC UI Design Skill for Claude Code

set -e

SKILL_NAME="cosmic-ui-design-skill"
SKILL_SOURCE="$(dirname "$0")/$SKILL_NAME"
SKILL_DEST="$HOME/.config/claude-code/skills/$SKILL_NAME"

echo "🚀 Installing COSMIC UI Design Skill for Claude Code..."
echo ""

# Check if source directory exists
if [ ! -d "$SKILL_SOURCE" ]; then
    echo "❌ Error: Skill directory not found at $SKILL_SOURCE"
    exit 1
fi

# Create skills directory if it doesn't exist
mkdir -p "$HOME/.config/claude-code/skills"

# Remove existing installation if present
if [ -d "$SKILL_DEST" ]; then
    echo "🔄 Removing existing installation..."
    rm -rf "$SKILL_DEST"
fi

# Copy skill to Claude Code skills directory
echo "📦 Copying skill files..."
cp -r "$SKILL_SOURCE" "$SKILL_DEST"

echo ""
echo "✅ Skill successfully installed to: $SKILL_DEST"
echo ""
echo "📚 The skill includes 7 specialized agents:"
echo "   • @cosmic-architect           - Application architecture review"
echo "   • @cosmic-theme-expert        - Theming & styling audit"
echo "   • @cosmic-applet-specialist   - Panel applet expertise"
echo "   • @cosmic-widget-builder      - Widget composition"
echo "   • @cosmic-error-handler       - Error handling safety"
echo "   • @cosmic-performance-optimizer - Performance optimization"
echo "   • @cosmic-code-reviewer       - Comprehensive code review"
echo ""
echo "🎯 Quick Start:"
echo "   @cosmic-code-reviewer /pre-commit-check"
echo "   @cosmic-architect review this application"
echo "   @cosmic-theme-expert /audit-theming"
echo ""
echo "📖 Full documentation: .claude/skills/cosmic-ui-design-skill/README.md"
echo ""
echo "⚠️  Important: Restart Claude Code to activate the skill"
echo ""
