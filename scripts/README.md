# COSMIC Connect Testing Scripts

This directory contains scripts for testing COSMIC Connect functionality with real devices.

## Available Scripts

### test-plugins.sh

Comprehensive plugin testing script for validating all plugin integrations with real devices.

**Features:**
- Automated testing of all major plugins
- Interactive testing menu
- Individual plugin tests
- Comprehensive test reporting

**Usage:**

```bash
# Basic tests with auto-detected device
./scripts/test-plugins.sh

# Interactive menu for manual testing
./scripts/test-plugins.sh --interactive

# Comprehensive tests on specific device
./scripts/test-plugins.sh --all <device_id>

# Show help
./scripts/test-plugins.sh --help
```

**Tests Included:**
- 🏓 Ping - Connectivity test
- 🔋 Battery - Battery status updates
- 📤 Share Text - Text sharing
- 🔗 Share URL - URL sharing
- 🔍 Find Phone - Ring device
- 📋 Clipboard - Clipboard sync
- 🌐 Connection Status - Device connectivity

**Interactive Mode:**

Interactive mode provides a menu-driven interface for testing plugins one at a time:

```bash
./scripts/test-plugins.sh --interactive
```

This launches an interactive menu where you can:
- Test individual plugins
- Run all tests
- View test results
- Customize test parameters (text, URLs, etc.)

**Requirements:**
- COSMIC Connect daemon running
- Device paired and connected
- xclip (optional, for clipboard tests)

**Examples:**

```bash
# Quick test with first available device
./scripts/test-plugins.sh

# Interactive testing
./scripts/test-plugins.sh --interactive

# Full test suite on specific device
./scripts/test-plugins.sh --all 1b7bbb613c0c42bb9a0b80b24d28631d

# Get device ID first, then test
busctl --user call io.github.olafkfreund.CosmicExtConnect \
  /io/github/olafkfreund/CosmicExtConnect \
  io.github.olafkfreund.CosmicExtConnect \
  GetDevices
```

## Related Documentation

- **Manual Testing:** docs/PLUGIN_TESTING_GUIDE.md - Comprehensive manual testing procedures
- **Automated Tests:** docs/AUTOMATED_TESTING.md - Integration test suite documentation
- **Debugging:** docs/DEBUGGING.md - Debugging tools and techniques
- **Troubleshooting:** docs/TROUBLESHOOTING.md - Common issues and solutions

## Adding New Tests

To add a new plugin test to `test-plugins.sh`:

1. **Add test function:**
```bash
test_my_plugin() {
    echo -e "${CYAN}🔌 Testing My Plugin${NC}"
    if busctl --user call "$DBUS_SERVICE" "$DBUS_PATH" "$DBUS_INTERFACE" \
        MyPluginMethod s "$DEVICE_ID" 2>&1 | grep -q "success"; then
        echo -e "   ${GREEN}✓ Test passed${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        echo -e "   ${RED}✗ Test failed${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
}
```

2. **Add to test sequence:**
```bash
echo -e "${BLUE}X. 🔌 My Plugin${NC}"
test_my_plugin
sleep 1
```

3. **Add to interactive menu:**
```bash
echo "  X) 🔌  Test My Plugin"
# ...
case $choice in
    # ...
    X)
        test_my_plugin
        ;;
esac
```

## Testing Workflow

1. **Start daemon:**
   ```bash
   cosmic-ext-connect-daemon
   ```

2. **Pair device:**
   - Open applet UI
   - Pair with device

3. **Run automated tests:**
   ```bash
   ./scripts/test-plugins.sh --all <device_id>
   ```

4. **Run integration tests:**
   ```bash
   cargo test --test plugin_integration_tests
   ```

5. **Manual testing:**
   - Follow docs/PLUGIN_TESTING_GUIDE.md
   - Test edge cases
   - Test with multiple devices

## Test Results Interpretation

**Exit Codes:**
- `0` - All tests passed
- `1` - One or more tests failed

**Output Indicators:**
- ✓ (Green) - Test passed
- ✗ (Red) - Test failed
- ⚠ (Yellow) - Warning or manual verification needed
- → (Yellow) - Action required on device

## Troubleshooting

If tests fail:

1. Check daemon is running:
   ```bash
   busctl --user status io.github.olafkfreund.CosmicExtConnect
   ```

2. Check device is connected:
   ```bash
   busctl --user call io.github.olafkfreund.CosmicExtConnect \
     /io/github/olafkfreund/CosmicExtConnect \
     io.github.olafkfreund.CosmicExtConnect \
     GetDevices
   ```

3. Monitor logs:
   ```bash
   tail -f /tmp/daemon-debug-verbose.log
   ```

4. Try reconnecting device

See docs/TROUBLESHOOTING.md for more help.

## Contributing

When adding new scripts:
- Follow bash best practices
- Include usage documentation
- Make scripts executable: `chmod +x script.sh`
- Update this README
- Add examples

---

**Last Updated:** 2026-01-16
**Test Coverage:** 6+ plugins with automated tests
