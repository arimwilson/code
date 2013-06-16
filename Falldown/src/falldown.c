#include "pebble_os.h"
#include "pebble_app.h"
#include "pebble_fonts.h"

#include "common.h"

#define MY_UUID { 0x51, 0x74, 0xB3, 0x1A, 0x71, 0xB4, 0x4F, 0x92, 0xA1, 0xF5, 0x0E, 0xCC, 0x5A, 0xB5, 0x1B, 0x52 }
PBL_APP_INFO(
    MY_UUID, "Falldown", "Ari Wilson", 1, 0 /* App version */,
    RESOURCE_ID_IMAGE_ICON, APP_INFO_STANDARD_APP);

const int16_t kWidth = 144;
const int16_t kHeight = 168;
const int16_t kStatusBarHeight = 16;
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
typedef struct {
  int16_t x;
  int16_t y;
  Layer layer;
} Circle;
Circle circle;

void circle_update_proc(Layer* layer, GContext* ctx) {
  graphics_context_set_fill_color(ctx, GColorWhite);
  graphics_fill_circle(
      ctx, GPoint(kCircleRadius / 2, kCircleRadius / 2), kCircleRadius);
}

void init_circle(Layer* parent_layer, int16_t x, int16_t y, Circle* circle) {
  circle->x = x;
  circle->y = y;
  layer_init(&circle->layer, GRect(
        x, y, kCircleRadius, kCircleRadius));
  layer_set_update_proc(&circle->layer, (LayerUpdateProc)circle_update_proc);
  layer_add_child(parent_layer, &circle->layer);
}

const int16_t kDistanceBetweenLines = 20;
const int16_t kLineThickness = 3;
const int16_t kLineSegments = 6;
const int16_t kMaxHoles = 2;
const int16_t kLineSegmentWidth = 24;  // kWidth / kLineSegments
// (kHeight - kStatusBarHeight) / (kLineThickness + kDistanceBetweenLines)
const int16_t kLineCount = 6;
typedef struct {
  int16_t y;  // location of this line on the screen.
  int16_t holes[2 /* kMaxHoles */];  // which segments have holes
  int16_t holes_size;
  Layer layer;
} Line;
Line lines[6 /* kLineCount */];

void line_update_proc(Layer* layer, GContext* ctx) {
  graphics_context_set_fill_color(ctx, GColorWhite);
  graphics_fill_rect(ctx, GRect(0, 0, kWidth, kLineThickness), 0, GCornerNone);
}

void init_line(Layer* parent_layer, int16_t y, Line* line) {
  line->y = y;
  line->holes_size = common_rand() % kMaxHoles + 1;
  for (int16_t i = 0; i < line->holes_size; ++i) {
    line->holes[i] = common_rand() % kLineSegments;
  }
  layer_init(&line->layer, GRect(0, y, kWidth, kLineThickness));
  layer_set_update_proc(&line->layer, (LayerUpdateProc)line_update_proc);
  layer_add_child(parent_layer, &line->layer);
}

void init_lines(Layer* parent_layer, Line (*lines)[6 /* kLineCount */]) {
  for (int16_t i = 1; i <= kLineCount; ++i) {
    init_line(
        parent_layer,
        kStatusBarHeight + (kDistanceBetweenLines + kLineThickness) * i,
        &((*lines)[i - 1]));
  }
}

void handle_init(AppContextRef ctx) {
  (void)ctx;
  common_srand(common_time());

  window_init(&window, "Falldown");
  window_set_background_color(&window, GColorBlack);
  window_stack_push(&window, true /* Animated */);

  Layer* root_layer = window_get_root_layer(&window);
  // Initialize the player circle.
  init_circle(root_layer, (kWidth - kCircleRadius) / 2, kStatusBarHeight, &circle);

  // Initialize the lines to fall down.
  init_lines(root_layer, &lines);

  // Attach our desired button functionality
  window_set_click_config_provider(&window, (ClickConfigProvider)click_config_provider);
}

void handle_deinit(AppContextRef ctx) {
  (void)ctx;
}

void pbl_main(void *params) {
  PebbleAppHandlers handlers = {
    .init_handler = &handle_init,
    .deinit_handler = &handle_deinit,
  };
  app_event_loop(params, &handlers);
}
