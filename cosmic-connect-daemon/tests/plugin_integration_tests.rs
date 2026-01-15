//! Integration tests for plugin functionality via DBus
//!
//! Tests the integration between plugins and the DBus interface,
//! ensuring plugin actions work correctly end-to-end.

use anyhow::Result;
use cosmic_connect_core::plugins::{battery, notification, ping, Plugin, PluginFactory};
use cosmic_connect_core::{Device, DeviceInfo, DeviceType};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

/// Mock device for testing
fn create_mock_device() -> Device {
    Device {
        info: DeviceInfo::new("Test Device", DeviceType::Phone, 1716),
        connection_state: cosmic_connect_core::ConnectionState::Connected,
        pairing_status: cosmic_connect_core::PairingStatus::Paired,
        is_trusted: true,
        last_seen: 0,
        last_connected: Some(0),
        host: Some("127.0.0.1".to_string()),
        port: Some(1716),
        certificate_fingerprint: None,
        certificate_data: None,
    }
}

#[tokio::test]
async fn test_battery_plugin_initialization() -> Result<()> {
    // Test that battery plugin can be created and initialized
    let mut plugin = battery::BatteryPlugin::new();
    let device = create_mock_device();

    plugin.init(&device).await?;
    assert_eq!(plugin.name(), "battery");

    Ok(())
}

#[tokio::test]
async fn test_battery_plugin_capabilities() {
    let plugin = battery::BatteryPlugin::new();

    let incoming = plugin.incoming_capabilities();
    assert!(incoming.contains(&"kdeconnect.battery".to_string()));
    assert!(incoming.contains(&"kdeconnect.battery.request".to_string()));

    let outgoing = plugin.outgoing_capabilities();
    assert!(outgoing.contains(&"kdeconnect.battery".to_string()));
    assert!(outgoing.contains(&"kdeconnect.battery.request".to_string()));
}

#[tokio::test]
async fn test_battery_status_query() {
    let mut plugin = battery::BatteryPlugin::new();
    let device = create_mock_device();

    plugin.init(&device).await.unwrap();

    // Initially no battery status
    assert!(plugin.get_battery_status().is_none());

    // Create a battery status for the packet
    let battery_status = battery::BatteryStatus {
        current_charge: 85,
        is_charging: true,
        threshold_event: 0,
    };

    // Simulate receiving battery packet
    let battery_packet = plugin.create_battery_packet(&battery_status);
    let mut device_mut = device;
    plugin
        .handle_packet(&battery_packet, &mut device_mut)
        .await
        .unwrap();

    // Now should have battery status
    let status = plugin.get_battery_status();
    assert!(status.is_some());
    let status = status.unwrap();
    assert_eq!(status.current_charge, 85);
    assert_eq!(status.is_charging, true);
}

#[tokio::test]
async fn test_notification_plugin_initialization() -> Result<()> {
    let mut plugin = notification::NotificationPlugin::new();
    let device = create_mock_device();

    plugin.init(&device).await?;
    assert_eq!(plugin.name(), "notification");
    assert_eq!(plugin.notification_count(), 0);

    Ok(())
}

#[tokio::test]
async fn test_notification_packet_creation() {
    let plugin = notification::NotificationPlugin::new();

    let notification =
        notification::Notification::new("test-123", "Test App", "Test Title", "Test Body", true);

    let packet = plugin.create_notification_packet(&notification);
    assert_eq!(packet.packet_type, "kdeconnect.notification");

    // Check packet body contains notification data
    let body = packet.body;
    assert_eq!(body["id"], "test-123");
    assert_eq!(body["appName"], "Test App");
    assert_eq!(body["title"], "Test Title");
    assert_eq!(body["text"], "Test Body");
}

#[tokio::test]
async fn test_ping_plugin_initialization() -> Result<()> {
    let mut plugin = ping::PingPlugin::new();
    let device = create_mock_device();

    plugin.init(&device).await?;
    assert_eq!(plugin.name(), "ping");

    Ok(())
}

#[tokio::test]
async fn test_ping_packet_creation() {
    let plugin = ping::PingPlugin::new();

    let packet = plugin.create_ping(Some("Hello!".to_string()));
    assert_eq!(packet.packet_type, "kdeconnect.ping");
    assert_eq!(packet.body["message"], "Hello!");

    let packet_no_message = plugin.create_ping(None);
    assert_eq!(packet_no_message.packet_type, "kdeconnect.ping");
}

#[tokio::test]
async fn test_plugin_trait_downcast() {
    use cosmic_connect_core::plugins::Plugin;
    use std::any::Any;

    // Test that we can downcast from trait object
    let plugin: Box<dyn Plugin> = Box::new(battery::BatteryPlugin::new());

    // Downcast to concrete type
    let battery_plugin = plugin.as_any().downcast_ref::<battery::BatteryPlugin>();
    assert!(battery_plugin.is_some());

    let battery_plugin = battery_plugin.unwrap();
    assert_eq!(battery_plugin.name(), "battery");
}

#[tokio::test]
async fn test_plugin_manager_battery_query() {
    use cosmic_connect_core::plugins::PluginManager;

    let mut manager = PluginManager::new();

    // Register battery plugin factory
    manager.register_factory(Arc::new(battery::BatteryPluginFactory));

    let device = create_mock_device();
    let device_id = device.info.device_id.clone();

    // Initialize plugins for device (they auto-start after init)
    manager
        .init_device_plugins(&device_id, &device)
        .await
        .unwrap();

    // Initially no battery status
    let status = manager.get_device_battery_status(&device_id);
    assert!(status.is_none());

    // TODO: Test receiving battery packet and querying status
    // This requires access to device packet handling which is not
    // exposed in the current plugin API
}

/// Test that plugins can be created via factories
#[tokio::test]
async fn test_plugin_factories() {
    let battery_factory = battery::BatteryPluginFactory;
    assert_eq!(battery_factory.name(), "battery");
    let plugin = battery_factory.create();
    assert_eq!(plugin.name(), "battery");

    let ping_factory = ping::PingPluginFactory;
    assert_eq!(ping_factory.name(), "ping");
    let plugin = ping_factory.create();
    assert_eq!(plugin.name(), "ping");

    let notification_factory = notification::NotificationPluginFactory;
    assert_eq!(notification_factory.name(), "notification");
    let plugin = notification_factory.create();
    assert_eq!(plugin.name(), "notification");
}

/// Test multiple plugins can coexist
#[tokio::test]
async fn test_multiple_plugins() -> Result<()> {
    use cosmic_connect_core::plugins::PluginManager;

    let mut manager = PluginManager::new();

    // Register multiple plugin factories
    manager.register_factory(Arc::new(battery::BatteryPluginFactory));
    manager.register_factory(Arc::new(ping::PingPluginFactory));
    manager.register_factory(Arc::new(notification::NotificationPluginFactory));

    let device = create_mock_device();
    let device_id = device.info.device_id.clone();

    // Initialize all plugins for device (they auto-start after init)
    manager.init_device_plugins(&device_id, &device).await?;

    // All should be working
    Ok(())
}

#[tokio::test]
async fn test_plugin_lifecycle() -> Result<()> {
    let mut plugin = battery::BatteryPlugin::new();
    let device = create_mock_device();

    // Test lifecycle: init -> start -> stop
    plugin.init(&device).await?;
    plugin.start().await?;
    plugin.stop().await?;

    Ok(())
}
