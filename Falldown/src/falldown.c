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

  config[BUTTON_ID_SELECT]->click.handler = (ClickHandler)select_single_click_handler;

  config[BUTTON_ID_SELECT]->long_click.handler = (ClickHandler)select_long_click_handler;

  config[BUTTON_ID_UP]->click.handler = (ClickHandler)up_single_click_handler;
  config[BUTTON_ID_UP]->click.repeat_interval_ms = 100;

  config[BUTTON_ID_DOWN]->click.handler = (ClickHandler)down_single_click_handler;
  config[BUTTON_ID_DOWN]->click.repeat_interval_ms = 100;
}


const int16_t kCircleRadius = 10;
struct Circle {
  int16_t x;
  int16_t y;
  Layer layer;
}

void init_circle(Layer* parent_layer, int16_t x, int16_t y, Circle* circle) {
  circle->x = x;
  circle->y = y;
  layer_init(&circle->layer, GRect(
        x, y, kCircleRadius, kCircleRadius));
  layer_set_update_proc(&circle->layer, &circle_update_proc);
  layer_add_child(&parent_layer, &circle->layer);
}

void circle_update_proc(Layer* layer, GContext* gtx) {
}

void delete_circle(Circle* circle) {
  layer_remove_child_layers(&circle->layer);
  layer_remove_from_parent(&circle->layer);
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
  int16_t* holes;  // which segments have holes..
  int16_t holes_size;
  Layer layer;
}

void init_line(Layer* parent_layer, int16_t y, Line* line) {
  line->y = y;
  line->holes_size = rand() % kMaxHoles + 1;
  line->holes = malloc(line->holes_size * sizeof(line->holes));
  for (int16_t i = 0; i < holes_size) {
    line->holes[i] = rand() % kLineSegments;
  }
  layer_init(&line->layer, GRect(0, y, kWidth, kLineThickness));
  layer_set_update_proc(&line->layer, &line_update_proc);
  layer_add_child(&parent_layer, &line->layer);
}

void init_lines(Layer* parent_layer, Line *lines[], int16_t lines_size) {
  for (int16_t i = 1; i <= lines_size; ++i) {
    init_line(parent_layer, kDistanceBetweenLines * i, &((*lines)[i - 1]));
  }
}

void line_update_proc(Layer* layer, GContext* ctx) {
}

void delete_line(Line* line) {
  free(line->holes);
  line->holes = NULL;
  layer_remove_child_layers(&line->layer);
  layer_remove_from_parent(&line->layer);
}

void delete_lines(Line* lines[], int16_t lines_size) {
  for (int16_t i = 0; i < lines_size; ++i) {
    delete_line(&(*lines)[i]);
  }
}

// Standard app initialisation
void handle_init(AppContextRef ctx) {
  (void)ctx;

  window_init(&window, "Falldown");
  window_set_background_color(&window, GColorBlack);
  window_stack_push(&window, true /* Animated */);

  srand(time(NULL));
  // Initialize the player circle.
  Circle circle;
  init_circle(&window.layer, (kWidth - kCircleRadius) / 2, 0. &circle);

  // Initialize the lines to fall down.
  int16_t lines_size = kHeight / (kLineThickness + kDistanceBetweenLines);
  Line lines[lines_size];
  init_lines(&window.layer, &lines, lines_size);

  // Attach our desired button functionality
  window_set_click_config_provider(&window, (ClickConfigProvider)click_config_provider);
}

void pbl_main(void *params) {
  PebbleAppHandlers handlers = {
    .init_handler = &handle_init,
  };
  app_event_loop(params, &handlers);
}
