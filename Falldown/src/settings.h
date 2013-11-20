#ifndef SETTINGS_H
#define SETTINGS_H

#include "pebble_os.h"

Window* menu_window;
SimpleMenuLayer* menu_layer;
SimpleMenuSection menu_sections[1];
SimpleMenuItem menu_items[1];

typedef struct {
  bool accelerometer_control;
} FalldownSettings;
FalldownSettings falldown_settings;
bool in_menu = false;

void accelerometer_control_callback(int index, void* context) {
  if (index != 0) return;
  falldown_settings.accelerometer_control =
      !falldown_settings.accelerometer_control;
  menu_items[0].subtitle =
      (falldown_settings.accelerometer_control? "Yes" : "No");
  menu_layer_reload_data(&menu_layer.menu);
}

void handle_appear(Window *window) {
  scroll_layer_set_frame(menu_layer.menu.scroll_layer, window->layer.bounds);
  in_menu = true;
}

void handle_unload(Window* window) {
  in_menu = false;
}

void init_settings() {
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
      menu_window->layer.frame, &menu_window, menu_sections,
      ARRAY_LENGTH(menu_sections), NULL);
  layer_add_child(&menu_window->layer, &menu_layer.menu.scroll_layer.layer);
}

void display_settings() {
  menu_items[0].subtitle =
      (falldown_settings.accelerometer_control? "Yes" : "No");
  window_stack_push(menu_window, true /* Animated */);
}

#endif
