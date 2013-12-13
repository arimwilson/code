#ifndef SETTINGS_H
#define SETTINGS_H

typedef struct {
  bool accelerometer_control;
} FalldownSettings;
FalldownSettings falldown_settings;
bool in_menu = false;

void accelerometer_control_callback(int index, void* context);
void handle_appear(Window* window);
void handle_unload(Window* window);
void init_settings();
void display_settings();
void deinit_settings();

#endif // SETTINGS_H
