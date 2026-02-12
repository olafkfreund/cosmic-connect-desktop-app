#!/usr/bin/env bash
# test-plugins.sh - Comprehensive plugin testing script for COSMIC Connect
#
# Usage: ./scripts/test-plugins.sh [options] [device_id]
#        ./scripts/test-plugins.sh --interactive
#        ./scripts/test-plugins.sh --all <device_id>

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# DBus service details
DBUS_SERVICE="io.github.olafkfreund.CosmicExtConnect"
DBUS_PATH="/io/github/olafkfreund/CosmicExtConnect"
DBUS_INTERFACE="io.github.olafkfreund.CosmicExtConnect"

# Get device ID
DEVICE_ID="${1:-}"
INTERACTIVE_MODE=false
RUN_ALL_TESTS=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -i|--interactive)
            INTERACTIVE_MODE=true
            shift
            ;;
        -a|--all)
            RUN_ALL_TESTS=true
            shift
            ;;
        -h|--help)
            echo "COSMIC Connect Plugin Testing Script"
            echo ""
            echo "Usage: $0 [options] [device_id]"
            echo ""
            echo "Options:"
            echo "  -i, --interactive    Run in interactive mode"
            echo "  -a, --all           Run all tests automatically"
            echo "  -h, --help          Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0                           # Auto-detect device and run basic tests"
            echo "  $0 --interactive             # Interactive menu"
            echo "  $0 --all <device_id>         # Run all tests on specific device"
            exit 0
            ;;
        *)
            DEVICE_ID="$1"
            shift
            ;;
    esac
done

if [ -z "$DEVICE_ID" ]; then
    echo -e "${BLUE}🔍 Auto-detecting paired device...${NC}"
    # Try to get first connected device
    DEVICE_ID=$(busctl --user call "$DBUS_SERVICE" "$DBUS_PATH" "$DBUS_INTERFACE" \
        GetDevices 2>/dev/null | grep -oP '[a-f0-9]{32}' | head -1 || echo "")

    if [ -z "$DEVICE_ID" ]; then
        echo -e "${RED}❌ No paired device found. Please pair a device first.${NC}"
        exit 1
    fi
    echo -e "${GREEN}✓ Found device: $DEVICE_ID${NC}"
fi

echo ""
echo "╔════════════════════════════════════════════╗"
echo "║   KDE Connect Plugin Testing Suite        ║"
echo "║   Device: ${DEVICE_ID:0:16}...  ║"
echo "╚════════════════════════════════════════════╝"
echo ""

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_TOTAL=0

# Function to test a plugin
test_plugin() {
    local plugin_name="$1"
    local test_command="$2"
    local expected_result="$3"

    TESTS_TOTAL=$((TESTS_TOTAL + 1))
    echo -n "Testing ${plugin_name}... "

    if eval "$test_command" >/dev/null 2>&1; then
        echo -e "${GREEN}✓ PASS${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        echo -e "${RED}✗ FAIL${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
}

# Individual test functions for interactive mode
test_share_text() {
    local text="${1:-Test message from COSMIC Connect at $(date +%H:%M:%S)}"
    echo -e "${CYAN}📤 Testing Share Text Plugin${NC}"
    echo -e "   Sending: \"${text}\""

    if busctl --user call "$DBUS_SERVICE" "$DBUS_PATH" "$DBUS_INTERFACE" \
        ShareText ss "$DEVICE_ID" "$text" 2>&1 | grep -q "s \"\""; then
        echo -e "   ${GREEN}✓ Text shared successfully${NC}"
        echo -e "   ${YELLOW}→ Check device for shared text${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        echo -e "   ${RED}✗ Failed to share text${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
}

test_share_url() {
    local url="${1:-https://system76.com}"
    echo -e "${CYAN}🔗 Testing Share URL Plugin${NC}"
    echo -e "   Sending: ${url}"

    if busctl --user call "$DBUS_SERVICE" "$DBUS_PATH" "$DBUS_INTERFACE" \
        ShareUrl ss "$DEVICE_ID" "$url" 2>&1 | grep -q "s \"\""; then
        echo -e "   ${GREEN}✓ URL shared successfully${NC}"
        echo -e "   ${YELLOW}→ Check device for URL notification${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        echo -e "   ${RED}✗ Failed to share URL${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
}

test_battery_update() {
    echo -e "${CYAN}🔋 Testing Battery Update Request${NC}"

    if busctl --user call "$DBUS_SERVICE" "$DBUS_PATH" "$DBUS_INTERFACE" \
        RequestBatteryUpdate s "$DEVICE_ID" 2>&1 | grep -q "s \"\""; then
        echo -e "   ${GREEN}✓ Battery update requested${NC}"
        echo -e "   ${YELLOW}→ Check applet UI for battery status${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        echo -e "   ${RED}✗ Failed to request battery update${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
}

test_clipboard_sync() {
    echo -e "${CYAN}📋 Testing Clipboard Sync${NC}"
    local test_text="COSMIC Connect Clipboard Test $(date +%s)"

    if command -v xclip &> /dev/null; then
        echo -n "$test_text" | xclip -selection clipboard
        echo -e "   ${GREEN}✓ Copied to clipboard: \"$test_text\"${NC}"
        echo -e "   ${YELLOW}→ Paste on device to verify sync${NC}"
        echo -e "   ${YELLOW}→ Copy text on device and paste here to test reverse${NC}"
        return 0
    else
        echo -e "   ${YELLOW}⚠ xclip not installed, skipping clipboard test${NC}"
        return 1
    fi
}

# Interactive menu
show_interactive_menu() {
    while true; do
        clear
        echo "╔════════════════════════════════════════════╗"
        echo "║   COSMIC Connect Plugin Testing Menu      ║"
        echo "╠════════════════════════════════════════════╣"
        echo -e "║ Device: ${DEVICE_ID:0:24}...    ║"
        echo "╚════════════════════════════════════════════╝"
        echo ""
        echo "Select a test to run:"
        echo ""
        echo "  1) 🏓  Test Ping"
        echo "  2) 🔋  Test Battery Update"
        echo "  3) 📤  Test Share Text"
        echo "  4) 🔗  Test Share URL"
        echo "  5) 📋  Test Clipboard Sync"
        echo "  6) 🔍  Test Find My Phone"
        echo "  7) 🧪  Run All Tests"
        echo "  8) 📊  View Test Results"
        echo "  0) 🚪  Exit"
        echo ""
        echo -n "Choice: "
        read -r choice

        echo ""
        case $choice in
            1)
                test_plugin "Ping" \
                    "busctl --user call $DBUS_SERVICE $DBUS_PATH $DBUS_INTERFACE SendPing s '$DEVICE_ID'" \
                    "success"
                ;;
            2)
                test_battery_update
                ;;
            3)
                echo -n "Enter text to share (or press Enter for default): "
                read -r text
                test_share_text "$text"
                ;;
            4)
                echo -n "Enter URL to share (or press Enter for default): "
                read -r url
                test_share_url "$url"
                ;;
            5)
                test_clipboard_sync
                ;;
            6)
                echo -e "${YELLOW}⚠ This will make your phone ring loudly!${NC}"
                echo -n "Continue? (y/N): "
                read -r confirm
                if [[ "$confirm" =~ ^[Yy]$ ]]; then
                    test_plugin "Find My Phone" \
                        "busctl --user call $DBUS_SERVICE $DBUS_PATH $DBUS_INTERFACE FindPhone s '$DEVICE_ID'" \
                        "success"
                fi
                ;;
            7)
                RUN_ALL_TESTS=true
                break
                ;;
            8)
                echo "Test Results:"
                echo "  Passed: $TESTS_PASSED"
                echo "  Failed: $TESTS_FAILED"
                echo "  Total:  $TESTS_TOTAL"
                ;;
            0)
                echo "Exiting..."
                exit 0
                ;;
            *)
                echo -e "${RED}Invalid choice${NC}"
                ;;
        esac

        echo ""
        echo -n "Press Enter to continue..."
        read -r
    done
}

# Check if interactive mode
if [ "$INTERACTIVE_MODE" = true ]; then
    show_interactive_menu
fi

# Test suite
echo -e "${BLUE}Running Plugin Tests...${NC}\n"

# Test 1: Ping Plugin
echo -e "${BLUE}1. 🏓 Ping Plugin${NC}"
test_plugin "Ping" \
    "busctl --user call $DBUS_SERVICE $DBUS_PATH $DBUS_INTERFACE SendPing s '$DEVICE_ID'" \
    "success"
sleep 1

# Test 2: Battery Plugin
echo -e "${BLUE}2. 🔋 Battery Plugin${NC}"
test_battery_update
sleep 1

# Test 3: Share Text
if [ "$RUN_ALL_TESTS" = true ]; then
    echo -e "${BLUE}3. 📤 Share Text Plugin${NC}"
    test_share_text "Automated test from COSMIC Connect"
    sleep 1
fi

# Test 4: Share URL
if [ "$RUN_ALL_TESTS" = true ]; then
    echo -e "${BLUE}4. 🔗 Share URL Plugin${NC}"
    test_share_url "https://system76.com"
    sleep 1
fi

# Test 5: Find My Phone Plugin
if [ "$RUN_ALL_TESTS" = true ]; then
    echo -e "${BLUE}5. 🔍 Find My Phone Plugin${NC}"
    echo -e "   ${YELLOW}⚠ This will make your phone ring loudly!${NC}"
    read -t 5 -p "   Press Enter to test or wait 5 seconds to skip..." || echo "   Skipped"
    if [ $? -eq 0 ]; then
        test_plugin "Find My Phone" \
            "busctl --user call $DBUS_SERVICE $DBUS_PATH $DBUS_INTERFACE FindPhone s '$DEVICE_ID'" \
            "success"
        echo -e "   ${YELLOW}ℹ Check if phone is ringing${NC}"
        sleep 3
    fi
fi

# Test 6: Connection Status
echo -e "${BLUE}6. 🌐 Connection Status${NC}"
test_plugin "Device Connected" \
    "busctl --user call $DBUS_SERVICE $DBUS_PATH $DBUS_INTERFACE GetDevices | grep -q '$DEVICE_ID'" \
    "success"

# Summary
echo ""
echo "╔════════════════════════════════════════════╗"
echo "║            Test Results Summary            ║"
echo "╠════════════════════════════════════════════╣"
printf "║  Total Tests:  %-28s║\n" "$TESTS_TOTAL"
printf "║  ${GREEN}Passed:       %-28s${NC}║\n" "$TESTS_PASSED"
printf "║  ${RED}Failed:       %-28s${NC}║\n" "$TESTS_FAILED"
echo "╚════════════════════════════════════════════╝"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All automated tests passed!${NC}"
    echo ""

    if [ "$RUN_ALL_TESTS" = true ]; then
        echo -e "${CYAN}Comprehensive Test Complete${NC}"
        echo "All plugin integrations are working correctly!"
    else
        echo -e "${YELLOW}Basic Tests Complete${NC}"
        echo "Run with --all flag for comprehensive testing"
    fi

    echo ""
    echo "Additional manual tests recommended:"
    echo "  1. Run interactive mode: $0 --interactive"
    echo "  2. Test file transfer: Share file from phone"
    echo "  3. Test clipboard: Copy text on both devices"
    echo "  4. Test MPRIS: Control media playback"
    echo "  5. See docs/PLUGIN_TESTING_GUIDE.md for details"
    echo ""
    echo "Integration test suite: cargo test --test plugin_integration_tests"
    exit 0
else
    echo -e "${RED}✗ $TESTS_FAILED test(s) failed${NC}"
    echo ""
    echo "Troubleshooting steps:"
    echo "  1. Verify device is paired and connected"
    echo "     → Check applet UI for device status"
    echo ""
    echo "  2. Check daemon logs for errors"
    echo "     → tail -f /tmp/daemon-debug-verbose.log"
    echo ""
    echo "  3. Verify DBus service is running"
    echo "     → busctl --user status $DBUS_SERVICE"
    echo ""
    echo "  4. Try reconnecting the device"
    echo "     → Disconnect and re-pair if necessary"
    echo ""
    echo "  5. Check plugin configuration"
    echo "     → ~/.config/cosmic-ext-connect/config.json"
    echo ""
    echo "For more help, see:"
    echo "  → docs/DEBUGGING.md"
    echo "  → docs/TROUBLESHOOTING.md"
    echo ""
    exit 1
fi
