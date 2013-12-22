#include <pebble.h>

#include "settings.h"

Window* menu_window;
SimpleMenuLayer* menu_layer;
SimpleMenuSection menu_sections[1];
SimpleMenuItem menu_items[1];

FalldownSettings falldown_settings;
bool in_menu = false;

void empty_accel(AccelData* data, uint32_t num_samples) {
}

void accelerometer_control_callback(int index, void* context) {
  if (index != 0) return;
  falldown_settings.accelerometer_control =
      !falldown_settings.accelerometer_control;
  persist_write_bool(0, falldown_settings.accelerometer_control);
  if (falldown_settings.accelerometer_control) {
    menu_items[0].subtitle = "Yes";
    accel_data_service_subscribe(0, (AccelDataHandler)empty_accel);
  } else {
    menu_items[0].subtitle = "No";
    accel_data_service_unsubscribe();
  }
  menu_layer_reload_data((MenuLayer*)menu_layer);
}

void handle_appear(Window* window) {
  in_menu = true;
}

void handle_unload(Window* window) {
  in_menu = false;
}

void init_settings() {
  if (persist_read_data(0, &falldown_settings, sizeof(FalldownSettings)) ==
      E_DOES_NOT_EXIST) {
    falldown_settings.accelerometer_control = false;
  }

  menu_window = window_create();
  window_set_window_handlers(menu_window, (WindowHandlers) {
    .appear = (WindowHandler)handle_appear,
    .unload = (WindowHandler)handle_unload,
  });
  menu_items[0] = (SimpleMenuItem) {
    .title = "Motion control?",
    .callback = &accelerometer_control_callback
  };
  menu_sections[0] = (SimpleMenuSection) {
    .title = NULL,
    .items = menu_items,
    .num_items = ARRAY_LENGTH(menu_items)
  };
  menu_layer = simple_menu_layer_create(
      layer_get_frame(window_get_root_layer(menu_window)), menu_window,
      menu_sections, ARRAY_LENGTH(menu_sections), NULL);
  layer_add_child(window_get_root_layer(menu_window),
                  simple_menu_layer_get_layer(menu_layer));
}

void display_settings() {
  menu_items[0].subtitle =
      (falldown_settings.accelerometer_control? "Yes" : "No");
  window_stack_push(menu_window, true /* Animated */);
}

void deinit_settings() {
  persist_write_data(0, &falldown_settings, sizeof(FalldownSettings));

  window_destroy(menu_window);
  simple_menu_layer_destroy(menu_layer);
}
