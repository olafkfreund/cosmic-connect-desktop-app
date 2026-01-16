#!/usr/bin/env python3
"""
Multi-Repository Sync Agent for COSMIC Connect

Analyzes changes in cosmic-connect-desktop-app and identifies what needs
to be synced to cosmic-connect-core and cosmic-connect-android.

Usage:
    python tools/sync-repos.py [--since COMMIT] [--verbose]

Options:
    --since COMMIT    Analyze changes since this commit (default: last sync or HEAD~10)
    --verbose, -v     Show detailed analysis
    --dry-run         Show what would be synced without making changes
"""

import subprocess
import sys
import re
import json
from pathlib import Path
from datetime import datetime
from typing import List, Dict, Set, Tuple
import yaml

class SyncAgent:
    def __init__(self, config_path: Path):
        self.config_path = config_path
        self.config = self.load_config()
        self.desktop_path = Path(self.config['repos']['desktop-app']['path'])
        self.changes = []
        self.sync_recommendations = {
            'core': [],
            'android': []
        }

    def load_config(self) -> dict:
        """Load sync configuration"""
        with open(self.config_path) as f:
            return yaml.safe_load(f)

    def get_recent_commits(self, since: str = None) -> List[str]:
        """Get recent commits to analyze"""
        if since is None:
            # Use last sync or last 10 commits
            last_sync = self.config.get('last_sync', {}).get('commit_hash')
            since = last_sync if last_sync else 'HEAD~10'

        cmd = ['git', '-C', str(self.desktop_path), 'log',
               f'{since}..HEAD', '--oneline']
        result = subprocess.run(cmd, capture_output=True, text=True)

        return [line.split(' ', 1) for line in result.stdout.strip().split('\n') if line]

    def get_changed_files(self, since: str = None) -> Set[str]:
        """Get files changed since commit"""
        if since is None:
            last_sync = self.config.get('last_sync', {}).get('commit_hash')
            since = last_sync if last_sync else 'HEAD~10'

        cmd = ['git', '-C', str(self.desktop_path), 'diff',
               '--name-only', f'{since}..HEAD']
        result = subprocess.run(cmd, capture_output=True, text=True)

        return set(result.stdout.strip().split('\n'))

    def analyze_triggers(self, changed_files: Set[str]) -> List[Dict]:
        """Analyze if any sync triggers are activated"""
        activated_triggers = []

        for trigger_name, trigger_config in self.config['triggers'].items():
            # Check if any changed file matches trigger
            for file_pattern in trigger_config['files']:
                matching_files = [f for f in changed_files
                                if self._matches_pattern(f, file_pattern)]

                if matching_files:
                    # Check if pattern exists in file
                    for file in matching_files:
                        if self._check_pattern_in_file(file, trigger_config['pattern']):
                            activated_triggers.append({
                                'name': trigger_name,
                                'severity': trigger_config['severity'],
                                'files': matching_files,
                                'sync_to': trigger_config['sync_to']
                            })

        return activated_triggers

    def _matches_pattern(self, filepath: str, pattern: str) -> bool:
        """Check if filepath matches glob pattern"""
        import fnmatch
        return fnmatch.fnmatch(filepath, pattern)

    def _check_pattern_in_file(self, filepath: str, pattern: str) -> bool:
        """Check if pattern exists in file"""
        full_path = self.desktop_path / filepath
        if not full_path.exists():
            return False

        try:
            with open(full_path) as f:
                content = f.read()
                return re.search(pattern, content) is not None
        except:
            return False

    def analyze_component_changes(self, changed_files: Set[str]) -> Dict:
        """Analyze which sync components are affected"""
        affected_components = {}

        for comp_name, comp_config in self.config['sync_components'].items():
            matching_files = []

            for file_pattern in comp_config['files']['desktop-app']:
                matches = [f for f in changed_files
                          if self._matches_pattern(f, file_pattern)]
                matching_files.extend(matches)

            if matching_files:
                affected_components[comp_name] = {
                    'description': comp_config['description'],
                    'files': matching_files,
                    'sync_to': comp_config['sync_to']
                }

        return affected_components

    def extract_code_snippets(self, filepath: str, pattern: str = None) -> List[str]:
        """Extract relevant code snippets from changed files"""
        full_path = self.desktop_path / filepath
        if not full_path.exists():
            return []

        try:
            with open(full_path) as f:
                lines = f.readlines()

            snippets = []
            in_relevant_block = False
            current_block = []

            for i, line in enumerate(lines):
                # Detect start of relevant blocks (enums, structs, consts)
                if any(kw in line for kw in ['pub enum', 'pub struct', 'pub const', 'impl']):
                    in_relevant_block = True
                    current_block = [line]
                elif in_relevant_block:
                    current_block.append(line)
                    # End of block
                    if line.strip() == '}' or (line.strip().startswith('//') and not current_block[-2].strip()):
                        snippets.append(''.join(current_block))
                        in_relevant_block = False
                        current_block = []

            return snippets
        except:
            return []

    def generate_sync_report(self, since: str = None) -> str:
        """Generate comprehensive sync report"""
        print("🔍 Analyzing changes in cosmic-connect-desktop-app...")

        # Get changes
        commits = self.get_recent_commits(since)
        changed_files = self.get_changed_files(since)

        print(f"   Found {len(commits)} commits with {len(changed_files)} changed files\n")

        # Analyze triggers
        print("🎯 Checking sync triggers...")
        activated_triggers = self.analyze_triggers(changed_files)

        # Analyze components
        print("📦 Analyzing affected components...")
        affected_components = self.analyze_component_changes(changed_files)

        # Build report
        report = []
        report.append("=" * 80)
        report.append("COSMIC CONNECT - MULTI-REPOSITORY SYNC REPORT")
        report.append("=" * 80)
        report.append(f"\nGenerated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        report.append(f"Analyzing: {len(commits)} recent commits")
        report.append(f"Changed files: {len(changed_files)}\n")

        # Recent commits
        report.append("\n📝 RECENT COMMITS")
        report.append("-" * 80)
        for commit_hash, commit_msg in commits[:5]:
            report.append(f"  {commit_hash} {commit_msg}")
        if len(commits) > 5:
            report.append(f"  ... and {len(commits) - 5} more")

        # Activated triggers
        if activated_triggers:
            report.append("\n\n🚨 ACTIVATED SYNC TRIGGERS")
            report.append("-" * 80)

            for trigger in activated_triggers:
                severity_emoji = {
                    'critical': '🔴',
                    'high': '🟠',
                    'medium': '🟡',
                    'low': '🟢'
                }.get(trigger['severity'], '⚪')

                report.append(f"\n{severity_emoji} {trigger['name'].upper()} [{trigger['severity']}]")
                report.append(f"   Sync to: {', '.join(trigger['sync_to'])}")
                report.append(f"   Files: {', '.join(trigger['files'])}")
        else:
            report.append("\n\n✅ No critical sync triggers activated")

        # Affected components
        if affected_components:
            report.append("\n\n📦 AFFECTED COMPONENTS")
            report.append("-" * 80)

            for comp_name, comp_data in affected_components.items():
                report.append(f"\n🔧 {comp_name.upper()}")
                report.append(f"   Description: {comp_data['description']}")
                report.append(f"   Sync to: {', '.join(comp_data['sync_to'])}")
                report.append(f"   Changed files:")
                for file in comp_data['files']:
                    report.append(f"      - {file}")

        # Generate recommendations
        report.append("\n\n💡 SYNC RECOMMENDATIONS")
        report.append("=" * 80)

        # Group by target repo
        sync_needed = {'core': set(), 'android': set()}

        for trigger in activated_triggers:
            for repo in trigger['sync_to']:
                sync_needed[repo].add(trigger['name'])

        for comp_name, comp_data in affected_components.items():
            for repo in comp_data['sync_to']:
                sync_needed[repo].add(f"{comp_name} component")

        if sync_needed['core']:
            report.append("\n🦀 COSMIC-CONNECT-CORE (Rust)")
            report.append("-" * 80)
            report.append("Changes needed:")
            for item in sync_needed['core']:
                report.append(f"  • {item}")
            report.append("\nRecommended actions:")
            report.append("  1. Review protocol changes in desktop-app")
            report.append("  2. Update core/src/lib.rs with new types")
            report.append("  3. Ensure crypto module is in sync")
            report.append("  4. Run core tests")
            report.append("  5. Update core CHANGELOG.md")

        if sync_needed['android']:
            report.append("\n\n🤖 COSMIC-CONNECT-ANDROID (Kotlin)")
            report.append("-" * 80)
            report.append("Changes needed:")
            for item in sync_needed['android']:
                report.append(f"  • {item}")
            report.append("\nRecommended actions:")
            report.append("  1. Review protocol changes in desktop-app")
            report.append("  2. Update packet type definitions (Kotlin data classes)")
            report.append("  3. Update error types (Kotlin sealed classes)")
            report.append("  4. Update constants (ports, timeouts, versions)")
            report.append("  5. Add new capabilities to manifest")
            report.append("  6. Run Android tests")
            report.append("  7. Update Android CHANGELOG.md")

        if not sync_needed['core'] and not sync_needed['android']:
            report.append("\n✨ No sync needed - changes are desktop-app specific")

        # Code snippets
        if affected_components:
            report.append("\n\n📄 CODE SNIPPETS FOR REFERENCE")
            report.append("=" * 80)

            for comp_name, comp_data in affected_components.items():
                if comp_data['files']:
                    report.append(f"\n--- {comp_name.upper()} ---")
                    for file in comp_data['files'][:2]:  # Limit to 2 files per component
                        snippets = self.extract_code_snippets(file)
                        if snippets:
                            report.append(f"\nFrom {file}:")
                            for snippet in snippets[:3]:  # Limit snippets
                                report.append("```rust")
                                report.append(snippet.rstrip())
                                report.append("```\n")

        report.append("\n" + "=" * 80)
        report.append("END OF SYNC REPORT")
        report.append("=" * 80)

        return '\n'.join(report)

    def save_report(self, report: str, output_path: Path = None):
        """Save report to file"""
        if output_path is None:
            timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
            output_path = self.desktop_path / f'sync-report-{timestamp}.md'

        with open(output_path, 'w') as f:
            f.write(report)

        print(f"\n📄 Report saved to: {output_path}")
        return output_path

    def update_last_sync(self):
        """Update last sync timestamp in config"""
        cmd = ['git', '-C', str(self.desktop_path), 'rev-parse', 'HEAD']
        result = subprocess.run(cmd, capture_output=True, text=True)
        current_commit = result.stdout.strip()

        self.config['last_sync'] = {
            'timestamp': datetime.now().isoformat(),
            'commit_hash': current_commit,
            'synced_changes': []
        }

        with open(self.config_path, 'w') as f:
            yaml.dump(self.config, f, default_flow_style=False)

def main():
    import argparse

    parser = argparse.ArgumentParser(description='COSMIC Connect Multi-Repo Sync Agent')
    parser.add_argument('--since', help='Analyze changes since this commit')
    parser.add_argument('--verbose', '-v', action='store_true', help='Verbose output')
    parser.add_argument('--dry-run', action='store_true', help='Show what would be synced')
    parser.add_argument('--output', help='Output file for report')

    args = parser.parse_args()

    # Find config
    config_path = Path.cwd() / '.sync-config.yaml'
    if not config_path.exists():
        print(f"❌ Config not found: {config_path}")
        print("   Please run from cosmic-connect-desktop-app root directory")
        sys.exit(1)

    # Create agent
    agent = SyncAgent(config_path)

    # Generate report
    report = agent.generate_sync_report(since=args.since)

    # Display report
    print("\n" + report)

    # Save report
    output_path = Path(args.output) if args.output else None
    saved_path = agent.save_report(report, output_path)

    # Update sync tracking
    if not args.dry_run:
        agent.update_last_sync()
        print("\n✅ Last sync timestamp updated")

    print("\n🎉 Sync analysis complete!")
    print(f"\n💡 Next steps:")
    print("   1. Review the generated report")
    print("   2. Apply recommended changes to cosmic-connect-core")
    print("   3. Apply recommended changes to cosmic-connect-android")
    print("   4. Test all repositories")
    print("   5. Commit and push changes")

if __name__ == '__main__':
    main()
