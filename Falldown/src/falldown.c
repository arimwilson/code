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

const int16_t kCircleRadius = 8;
typedef struct {
  Layer layer;
  int16_t x;
  int16_t y;
} Circle;
Circle circle;

void circle_update_proc(Circle* circle, GContext* ctx) {
  graphics_context_set_fill_color(ctx, GColorWhite);
  graphics_fill_circle(
      ctx, GPoint(kCircleRadius / 2, kCircleRadius / 2), kCircleRadius);
}

void circle_init(Layer* parent_layer, int16_t x, int16_t y, Circle* circle) {
  circle->x = x;
  circle->y = y;
  layer_init(&circle->layer, GRect(
        circle->x, circle->y, kCircleRadius, kCircleRadius));
  layer_set_update_proc(&circle->layer, (LayerUpdateProc)circle_update_proc);
  layer_add_child(parent_layer, &circle->layer);
}

const int16_t kDistanceBetweenLines = 30;
const int16_t kLineThickness = 3;
const int16_t kLineSegments = 6;
const int16_t kMaxHoles = 2;
const int16_t kLineSegmentWidth = 24;  // kWidth / kLineSegments
// (kHeight - kStatusBarHeight) / (kLineThickness + kDistanceBetweenLines)
const int16_t kLineCount = 4;
typedef struct {
  Layer layer;
  int16_t y;  // location of this line on the screen.
  int16_t holes[2 /* kMaxHoles */];  // which segments have holes
  int16_t holes_size;
} Line;
Line lines[4 /* kLineCount */];

void line_update_proc(Line* line, GContext* ctx) {
  graphics_context_set_fill_color(ctx, GColorWhite);
  graphics_fill_rect(ctx, GRect(0, 0, kWidth, kLineThickness), 0, GCornerNone);
  graphics_context_set_fill_color(ctx, GColorBlack);
  for (int16_t i = 0; i < line->holes_size; ++i) {
    graphics_fill_rect(
        ctx,
        GRect(line->holes[i] * kLineSegmentWidth, 0, kLineSegmentWidth,
              kLineThickness),
        0,
        GCornerNone);
  }
}

void line_generate(int16_t y, Line* line) {
  line->y = y;
  line->holes_size = common_rand() % kMaxHoles + 1;
  for (int16_t i = 0; i < line->holes_size; ++i) {
    line->holes[i] = common_rand() % kLineSegments;
  }
}

void line_init(Layer* circle_layer, int16_t y, Line* line) {
  line_generate(y, line);
  layer_init(&line->layer, GRect(0, line->y, kWidth, kLineThickness));
  layer_set_update_proc(&line->layer, (LayerUpdateProc)line_update_proc);
  layer_insert_below_sibling(&line->layer, circle_layer);
}

void lines_init(Layer* circle_layer, Line (*lines)[4 /* kLineCount */]) {
  for (int16_t i = 1; i <= kLineCount; ++i) {
    line_init(
        circle_layer,
        kStatusBarHeight + (kDistanceBetweenLines + kLineThickness) * i,
        &((*lines)[i - 1]));
  }
}
Window window;
const int16_t kUpdateMs = 50;
// Should be able to get across the screen in about 1s:
// kWidth / (1000 / kUpdateMs)
const int16_t kCircleVelocity = 7;
void up_single_click_handler(ClickRecognizerRef recognizer, Window *window) {
  (void)recognizer;
  (void)window;
  if (circle.x + kCircleRadius + kCircleVelocity < kWidth) {
    circle.x += kCircleVelocity;
    layer_set_frame(&circle.layer,
                    GRect(circle.x, circle.y, kCircleRadius, kCircleRadius));
  }
}

void down_single_click_handler(ClickRecognizerRef recognizer, Window *window) {
  (void)recognizer;
  (void)window;
  if (circle.x - kCircleVelocity > 0) {
    circle.x -= kCircleVelocity;
    layer_set_frame(&circle.layer,
                    GRect(circle.x, circle.y, kCircleRadius, kCircleRadius));
  }
}

void click_config_provider(ClickConfig **config, Window *window) {
  (void)window;

  config[BUTTON_ID_UP]->click.handler = (ClickHandler)up_single_click_handler;
  config[BUTTON_ID_UP]->click.repeat_interval_ms = kUpdateMs;

  config[BUTTON_ID_DOWN]->click.handler = (ClickHandler)down_single_click_handler;
  config[BUTTON_ID_DOWN]->click.repeat_interval_ms = kUpdateMs;
}

void handle_init(AppContextRef ctx) {
  (void)ctx;
  common_srand(common_time());

  window_init(&window, "Falldown");
  window_set_background_color(&window, GColorBlack);
  window_stack_push(&window, true /* Animated */);

  Layer* root_layer = window_get_root_layer(&window);
  // Initialize the player circle.
  circle_init(root_layer, (kWidth - kCircleRadius) / 2, kStatusBarHeight, &circle);

  // Initialize the lines to fall down.
  lines_init(&circle.layer, &lines);

  // Attach our desired button functionality
  window_set_click_config_provider(&window, (ClickConfigProvider)click_config_provider);

  // Start updating the game..
  app_timer_send_event(ctx, kUpdateMs, 0);
}

void handle_timer(AppContextRef ctx, AppTimerHandle handle, uint32_t cookie) {
  (void)ctx;

  // Update the player circle.
  // TODO(ariw): Intersection testing.

  // Update the lines to fall down.
  for (int16_t i = 0; i < kLineCount; ++i) {
    lines[i].y--;
    if (lines[i].y < kStatusBarHeight) {
      line_generate(
          lines[common_mod(i - 1, kLineCount)].y + kDistanceBetweenLines +
              kLineThickness,
          &lines[i]);
    }
    layer_set_frame(&lines[i].layer,
                    GRect(0, lines[i].y, kWidth, kLineThickness));
  }

  app_timer_send_event(ctx, kUpdateMs, 0);
}

void pbl_main(void *params) {
  PebbleAppHandlers handlers = {
    .init_handler = &handle_init,
    .timer_handler = &handle_timer,
  };
  app_event_loop(params, &handlers);
}
