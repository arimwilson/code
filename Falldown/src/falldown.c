#include "pebble_os.h"
#include "pebble_app.h"
#include "pebble_fonts.h"

#include "common.h"
#include "mini-printf.h"

#define MY_UUID { 0x51, 0x74, 0xB3, 0x1A, 0x71, 0xB4, 0x4F, 0x92, 0xA1, 0xF5, 0x0E, 0xCC, 0x5A, 0xB5, 0x1B, 0x52 }
PBL_APP_INFO(
    MY_UUID, "Falldown", "Ari Wilson", 1, 0 /* App version */,
    RESOURCE_ID_IMAGE_ICON, APP_INFO_STANDARD_APP);

const bool kDebug = false;
const int kTextSize = 14;

const int kWidth = 144;
const int kHeight = 168;
const int kStatusBarHeight = 16;

// How often to update game state.
const int kUpdateMs = 33;

// Player circle constants.
const int kCircleRadius = 8;
// Should be able to get across the screen in kAcrossScreenMs:
const int kAcrossScreenMs = 1000;
// kWidth / (kAcrossScreenMs / kUpdateMs)
const float kCircleXVelocity = 4.752;
// Falling speed of circle.
const float kCircleYVelocity = 1;

// Line constants.
const int kDistanceBetweenLines = 30;
const int kLineThickness = 3;
const int kMaxHoles = 2;
// TODO(ariw): Different size holes?
const int kLineSegments = 6;
const int kLineSegmentWidth = 24;  // kWidth / kLineSegments
// ceil((kHeight - kStatusBarHeight) / (kLineThickness + kDistanceBetweenLines))
const int kLineCount = 5;
// Lines move up one full screen size once every kDownScreenMs:
const int kDownScreenMs = 8000;
// -(kHeight - kStatusBarHeight) / (kDownScreenMs / kUpdateMs)
const float kInitialLineVelocity = -0.627;
// Every kVelocityIncreaseMs, multiply line velocity by kVelocityIncrease:
const int kVelocityIncreaseMs = 15000;
const float kVelocityIncrease = 1.05;

Window window;

TextLayer text_layer;
// TODO(ariw): Persistent high scores via httpebble?
int score = 0;

// Player circle data and functions.
typedef struct {
  Layer layer;
  float x;
  float y;
} Circle;
Circle circle;
float circle_x_velocity = 0;

void circle_update_proc(Circle* circle, GContext* ctx) {
  graphics_context_set_fill_color(ctx, GColorWhite);
  // TODO(ariw): Use an animated circle here instead of this function.
  graphics_fill_circle(
      ctx, GPoint(kCircleRadius / 2, kCircleRadius / 2), kCircleRadius);
}

void circle_init(Layer* parent_layer, int x, int y, Circle* circle) {
  layer_init(&circle->layer, GRect(
        circle->x, circle->y, kCircleRadius, kCircleRadius));
  layer_set_update_proc(&circle->layer, (LayerUpdateProc)circle_update_proc);
  layer_add_child(parent_layer, &circle->layer);
  circle->x = x;
  circle->y = y;
}

// Lines data and functions.
typedef struct {
  Layer layer;
  float y;  // location of this line on the screen
  int holes[2 /* kMaxHoles */];  // which segments have holes
  int holes_size;
} Line;
Line lines[5 /* kLineCount */];
int elapsed_time_ms = 0;
float lines_velocity = -0.627;  // kInitialLineVelocity

void line_update_proc(Line* line, GContext* ctx) {
  graphics_context_set_fill_color(ctx, GColorWhite);
  graphics_fill_rect(ctx, GRect(0, 0, kWidth, kLineThickness), 0, GCornerNone);
  graphics_context_set_fill_color(ctx, GColorBlack);
  for (int i = 0; i < line->holes_size; ++i) {
    graphics_fill_rect(
        ctx,
        GRect(line->holes[i] * kLineSegmentWidth, 0, kLineSegmentWidth,
              kLineThickness),
        0,
        GCornerNone);
  }
}

void line_generate(int y, Line* line) {
  line->y = y;
  line->holes_size = common_rand() % kMaxHoles + 1;
  line->holes_size = 2;
  common_shuffle_integers(line->holes_size, (int*)line->holes);
  common_insertion_sort((int*)line->holes, line->holes_size);
}

void line_init(Layer* parent_layer, int y, Line* line) {
  line_generate(y, line);
  layer_init(&line->layer, GRect(0, line->y, kWidth, kLineThickness));
  layer_set_update_proc(&line->layer, (LayerUpdateProc)line_update_proc);
  layer_add_child(parent_layer, &line->layer);
}

void lines_init(Layer* parent_layer, Line (*lines)[5 /* kLineCount */]) {
  for (int i = 1; i <= kLineCount; ++i) {
    line_init(
        parent_layer, (kDistanceBetweenLines + kLineThickness) * i,
        &((*lines)[i - 1]));
  }
}

// Whether a circle intersects any line at or before the next move.
// relative_velocity represents the per update pixel velocity between the lines
// and the circle. If an intersecting line exists, return its y coordinate.
// Return the enclosing x coordinates for the circle.
bool lines_circle_intersect(
    float relative_velocity, Line (*lines)[5 /* kLineCount */], Circle* circle,
    int* line_x, int* circle_min_x, int* circle_max_x) {
  *circle_min_x = 0;
  *circle_max_x = kWidth;
  for (int i = 0; i < kLineCount; ++i) {
    int y = (*lines)[i].y;
    // Determine whether the circle is passing through a line. If either the top
    // or bottom of the circle is inside the line, the circle is intersecting
    // the line.
    // TODO(ariw): This logic allows you to get caught if you move into a line
    // while passing through it.
    if ((circle->y + kCircleRadius + relative_velocity >= y &&
         circle->y + kCircleRadius + relative_velocity < y + kLineThickness) ||
        (circle->y >= y && circle->y < y + kLineThickness)) {
      // The circle is passing through a line. We need to check if our circle
      // fits through any holes in that line.  Since kCircleRadius <
      // kLineSegmentWidth, if the left side of our circle fits through a hole
      // and the right side of our circle fits through a hole, the entire circle
      // fits through a hole.
      bool hole_left = false;
      bool hole_right = false;
      for (int j = 0; j < (*lines)[i].holes_size; ++j) {
        int hole = (*lines)[i].holes[j];
        int hole_start_x = hole * kLineSegmentWidth;
        int hole_end_x = (hole + 1) * kLineSegmentWidth;
        if (circle->x >= hole_start_x && circle->x < hole_end_x) {
          hole_left = true;
        }
        float circle_end_x = circle->x + kCircleRadius;
        if (circle_end_x >= hole_start_x && circle_end_x < hole_end_x) {
          hole_right = true;
        }
      }
      if (!hole_left || !hole_right) *line_x = y;
      return !hole_left || !hole_right;
    }
  }
  return false;
}

// Input handlers.
void up_single_click_handler(ClickRecognizerRef recognizer, Window *window) {
  (void)recognizer;
  (void)window;
  circle_x_velocity = kCircleXVelocity;
}

void down_single_click_handler(ClickRecognizerRef recognizer, Window *window) {
  (void)recognizer;
  (void)window;
  circle_x_velocity = -kCircleXVelocity;
}

void click_config_provider(ClickConfig **config, Window *window) {
  (void)window;

  config[BUTTON_ID_UP]->click.handler = (ClickHandler)up_single_click_handler;
  config[BUTTON_ID_UP]->click.repeat_interval_ms = kUpdateMs;

  config[BUTTON_ID_DOWN]->click.handler = (ClickHandler)down_single_click_handler;
  config[BUTTON_ID_DOWN]->click.repeat_interval_ms = kUpdateMs;
}

// TODO(ariw): Merge this with the circle/line init functions.
void reset() {
  // Reset the score.
  score = 0;

  // Reset player circle.
  circle.x = (kWidth - kCircleRadius) / 2;
  circle.y = 0;
  circle_x_velocity = 0;

  // Reset the lines.
  for (int i = 1; i <= kLineCount; ++i) {
    line_generate((kDistanceBetweenLines + kLineThickness) * i, &lines[i - 1]);
  }

  // Reset our speed.
  elapsed_time_ms = 0;
  lines_velocity = kInitialLineVelocity;
}


void handle_init(AppContextRef ctx) {
  (void)ctx;
  common_srand(common_time());

  window_init(&window, "Falldown");
  window_set_background_color(&window, GColorBlack);
  window_stack_push(&window, true /* Animated */);

  Layer* root_layer = window_get_root_layer(&window);

  // Initialize the lines to fall down.
  lines_init(root_layer, &lines);

  // Initialize the score.
  text_layer_init(&text_layer, GRect(0, 0, kWidth, kTextSize));
  text_layer_set_text_alignment(&text_layer, GTextAlignmentRight);
  text_layer_set_background_color(&text_layer, GColorClear);
  text_layer_set_text_color(&text_layer, GColorWhite);
  layer_add_child(root_layer, (Layer*)&text_layer);

  // Initialize the player circle.
  circle_init(root_layer, (kWidth - kCircleRadius) / 2,  0, &circle);

  // Attach our desired button functionality
  window_set_click_config_provider(
      &window, (ClickConfigProvider)click_config_provider);


  // Start updating the game.
  app_timer_send_event(ctx, kUpdateMs, 0);
}

void handle_timer(AppContextRef ctx, AppTimerHandle handle, uint32_t cookie) {
  (void)ctx;

  app_timer_send_event(ctx, kUpdateMs, 0);

  // Update the score.
  if (!kDebug) {
    static char score_string[10];
    snprintf(score_string, 10, "%d", score);
    text_layer_set_text(&text_layer, score_string);
  }

  // Update the player circle.
  int line_y, circle_min_x, circle_max_x;
  if (circle.y < 0) {
    // Game over!
    reset();
  } else if (lines_circle_intersect(
      kCircleYVelocity - lines_velocity, &lines, &circle, &line_y,
      &circle_min_x, &circle_max_x)) {
    // Can't fall down yet, move up with the line.
    circle.y = line_y - kCircleRadius + lines_velocity;
  } else if (circle.y + kCircleRadius + lines_velocity <=
                 kHeight - kStatusBarHeight) {
    // Fall down!
    circle.y += kCircleYVelocity;
  }
  if (circle.x + circle_x_velocity >= circle_min_x &&
      circle.x + kCircleRadius + circle_x_velocity < circle_max_x) {
    circle.x += circle_x_velocity;
  }
  circle_x_velocity = 0;
  layer_set_frame(&circle.layer,
                  GRect((int)circle.x, (int)circle.y, kCircleRadius,
                        kCircleRadius));

  // Update the lines as they move upward.
  for (int i = 0; i < kLineCount; ++i) {
    lines[i].y += lines_velocity;
    if (lines[i].y < 0) {
      line_generate(
          lines[common_mod(i - 1, kLineCount)].y + kDistanceBetweenLines +
              kLineThickness,
          &lines[i]);
      score += 10;
    }
    layer_set_frame(&lines[i].layer,
                    GRect(0, (int)lines[i].y, kWidth, kLineThickness));
  }

  // Increase our speed sometimes.
  elapsed_time_ms += kUpdateMs;
  if (elapsed_time_ms % kVelocityIncreaseMs < kUpdateMs) {
    lines_velocity *= kVelocityIncrease;
  }
}

void pbl_main(void *params) {
  PebbleAppHandlers handlers = {
    .init_handler = &handle_init,
    .timer_handler = &handle_timer,
  };
  app_event_loop(params, &handlers);
}
