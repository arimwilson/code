#include <stdlib.h>
#include <time.h>

#include "pebble_os.h"
#include "pebble_app.h"
#include "pebble_fonts.h"


#define MY_UUID { 0x51, 0x74, 0xB3, 0x1A, 0x71, 0xB4, 0x4F, 0x92, 0xA1, 0xF5, 0x0E, 0xCC, 0x5A, 0xB5, 0x1B, 0x52 }
PBL_APP_INFO_SIMPLE(MY_UUID, "Falldown", "Ari Wilson", 1 /* App version */);

Window window;

void up_single_click_handler(ClickRecognizerRef recognizer, Window *window) {
  (void)recognizer;
  (void)window;
}

void down_single_click_handler(ClickRecognizerRef recognizer, Window *window) {
  (void)recognizer;
  (void)window;
}

void select_single_click_handler(ClickRecognizerRef recognizer, Window *window) {
  (void)recognizer;
  (void)window;

  text_layer_set_text(&textLayer, "Select!");
}

void select_long_click_handler(ClickRecognizerRef recognizer, Window *window) {
  (void)recognizer;
  (void)window;
}

void click_config_provider(ClickConfig **config, Window *window) {
  (void)window;

  config[BUTTON_ID_SELECT]->click.handler = (ClickHandler) select_single_click_handler;

  config[BUTTON_ID_SELECT]->long_click.handler = (ClickHandler) select_long_click_handler;

  config[BUTTON_ID_UP]->click.handler = (ClickHandler) up_single_click_handler;
  config[BUTTON_ID_UP]->click.repeat_interval_ms = 100;

  config[BUTTON_ID_DOWN]->click.handler = (ClickHandler) down_single_click_handler;
  config[BUTTON_ID_DOWN]->click.repeat_interval_ms = 100;
}

const int16_t kHeight = 168;
const int16_t kWidth = 144;
const int16_t kDistanceBetweenLines = 20;
const int16_t kLineThickness = 3;
const int kLineSegments = 6;
const int kMaxHoles = 2;
const int16_t kLineSegmentWidth = kWidth / kLineSegments;
struct Line {
  int16_t y;  // location of this line on the screen.
  int16_t* holes;  // which segments have holes.
  int16_t holes_size;
  Layer layer;
}

void init_line(int16_t y, Line* line) {
  line->y = y;
  line->holes_size = rand() % kMaxHoles + 1;
  line->holes = malloc(line->holes_size * sizeof(line->holes));
}

void init_lines(Line lines[], int16_t lines_size) {
  for (int16_t i = 1; i <= lines_size; ++i) {
    init_line(kDistanceBetweenLines * i, &lines[i - 1]);
  }
}}

void delete_line(Line* line) {
  free(line->holes);
  line->holes = NULL;
}

void delete_lines(Line lines[], int16_t lines_size) {
  for (int16_t i = 0; i < lines_size; ++i) {
    delete_line(&lines[i]);
  }
}

// Standard app initialisation
void handle_init(AppContextRef ctx) {
  (void)ctx;

  window_init(&window, "Falldown");
  window_set_background_color(&window, GColorBlack);
  window_stack_push(&window, true /* Animated */);

  srand(time(NULL));
  Layer circle;
  int16_t lines_size = kHeight / (kLineThickness + kDistanceBetweenLines);
  Line lines[lines_size];
  text_layer_init(&textLayer, window.layer.frame);
  layer_add_child(&window.layer, &textLayer.layer);

  // Attach our desired button functionality
  window_set_click_config_provider(&window, (ClickConfigProvider) click_config_provider);
}

void pbl_main(void *params) {
  PebbleAppHandlers handlers = {
    .init_handler = &handle_init,
  };
  app_event_loop(params, &handlers);
}
