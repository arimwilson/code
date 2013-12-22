// Example log line:
// app_log(APP_LOG_LEVEL_INFO, "falldown.c", 1, "log");

#include <pebble.h>

#include "common.h"
#include "hmac_sha2.h"
#include "mac_key.h"  // for kMacKey / kMacKeyLength
#include "settings.h"

extern const char* kMacKey;
extern const int kMacKeyLength;

// Size of temporary buffers.
const int kBufferSize = 256;

const int kTextSize = 14;
const int kTextLength = 12;

const int kWidth = 144;
const int kHeight = 168;
const int kStatusBarHeight = 16;

// How often to update game state.
const int kUpdateMs = 33;

// Player circle constants.
const int kCircleRadius = 4;
// Should be able to get across the screen in kAcrossScreenMs:
const int kAcrossScreenMs = 1000;
// Derive max acceleration from calculating constant acceleration required to
// make it across the screen in kAcrossScreenMs:
//
// distance(t) = integral(integral(acceleration(t)))
// d(t) = a*t^2/2
// kWidth = a*(kAcrossScreenMs / kUpdateMs)^2/2
// a = kWidth * 2 / (kAcrossScreenMs / kUpdateMs)^2
const float kCircleXMaxAccel = 0.32;
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

Window* game_window;

TextLayer* text_layer;
char text[12 /* kTextLength */];
int score = 0;
int sent_score;

// Player circle data and functions.
typedef Layer CircleLayer;
CircleLayer* circle_layer;
typedef struct {
  float x;
  float y;
} Circle;
float circle_x_velocity = 0;

AccelData filter = {
  .x = 0,
  .y = 0,
  .z = 0
};

void circle_update_proc(CircleLayer* circle_layer, GContext* ctx) {
  // TODO(ariw): Use an animated circle here instead of this function.
  graphics_context_set_fill_color(ctx, GColorWhite);
  graphics_fill_circle(
      ctx, GPoint(kCircleRadius, kCircleRadius), kCircleRadius - 1);
}

void circle_init(Layer* parent_layer, int x, int y, CircleLayer** circle_layer) {
  *circle_layer = layer_create_with_data(
      GRect(x, y, kCircleRadius * 2, kCircleRadius * 2), sizeof(Circle));
  Circle* circle = layer_get_data(*circle_layer);
  circle->x = x;
  circle->y = y;
  layer_set_update_proc(*circle_layer, (LayerUpdateProc)circle_update_proc);
  layer_add_child(parent_layer, *circle_layer);
}

// Lines data and functions.
typedef Layer LineLayer;
typedef LineLayer *(LineLayers[5 /* kLineCount */]);
LineLayers line_layers;
typedef struct {
  float y;  // location of this line on the screen
  int holes[2 /* kMaxHoles */];  // which segments have holes
  int holes_size;
} Line;
int elapsed_time_ms = 0;
float lines_velocity = -0.627;  // kInitialLineVelocity

void line_update_proc(LineLayer* line_layer, GContext* ctx) {
  Line* line = layer_get_data(line_layer);
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
  line->holes_size = rand() % kMaxHoles + 1;
  common_shuffle_integers(line->holes_size, (int*)line->holes);
  common_insertion_sort((int*)line->holes, line->holes_size);
}

void line_init(Layer* parent_layer, int y, LineLayer** line_layer) {
  *line_layer = layer_create_with_data(
      GRect(0, y, kWidth, kLineThickness), sizeof(Line));
  Line* line = layer_get_data(*line_layer);
  line_generate(y, line);
  layer_set_update_proc(*line_layer, (LayerUpdateProc)line_update_proc);
  layer_add_child(parent_layer, *line_layer);
}

void lines_init(Layer* parent_layer, LineLayers* line_layers) {
  for (int i = 0; i < kLineCount; ++i) {
    line_init(
        parent_layer, (kDistanceBetweenLines + kLineThickness) * (i + 2),
        &((*line_layers)[i]));
  }
}

// Whether a circle intersects any line during the next move and whether this is
// due to its x velocity or y velocity (considered independently).
// relative_{x,y}_velocity represents the per update pixel {x,y} velocity
// between the lines and the circle.
void lines_circle_intersect(
    float relative_x_velocity, float relative_y_velocity,
    LineLayers* line_layers, CircleLayer* circle_layer, bool* intersects_x,
    bool* intersects_y) {
  *intersects_x = false;
  *intersects_y = false;
  Circle* circle = layer_get_data(circle_layer);
  for (int i = 0; i < kLineCount; ++i) {
    Line* line = layer_get_data((*line_layers)[i]);
    int y = line->y;
    // Determine whether the circle is passing through a line. This happens only
    // if before the move, the top of the circle is either in or above the line
    // and, after the move, the bottom of the circle is either in or below the
    // line.
    if (circle->y < y + kLineThickness &&
        circle->y + kCircleRadius * 2 + relative_y_velocity >= y) {
      *intersects_y = true;
      // The circle is passing through a line. We need to check if our circle
      // fits through any holes in that line. Since holes are stored in
      // ascending order, we can simultaneously establish the boundaries of
      // larger holes and see if the circle fits through any of them.
      for (int j = 0; j < line->holes_size; ++j) {
        int hole_start_x = line->holes[j] * kLineSegmentWidth;
        while (j < line->holes_size - 1 &&
               line->holes[j] + 1 == line->holes[j + 1]) {
          ++j;
        }
        int hole_end_x = (line->holes[j] + 1) * kLineSegmentWidth;
        if (circle->x >= hole_start_x &&
            circle->x + kCircleRadius * 2 < hole_end_x) {
          if (circle->x + relative_x_velocity < hole_start_x ||
              circle->x + kCircleRadius * 2 + relative_x_velocity >=
                  hole_end_x) {
            *intersects_x = true;
          }
          *intersects_y = false;
        }
      }
      return;  // Circle can't be in more than one line since lines don't touch.
    }
  }
}

// Input handlers.
void up_single_click_handler(ClickRecognizerRef recognizer, Window *window) {
  (void)recognizer;
  (void)window;
  circle_x_velocity += kCircleXMaxAccel;
}

void down_single_click_handler(ClickRecognizerRef recognizer, Window *window) {
  (void)recognizer;
  (void)window;
  circle_x_velocity -= kCircleXMaxAccel;
}

void select_single_click_handler(ClickRecognizerRef recognizer, Window *window) {
  (void)recognizer;
  (void)window;
  display_settings();
}

void click_config_provider(Window *window) {
  (void)window;

  window_single_repeating_click_subscribe(
      BUTTON_ID_UP, kUpdateMs, (ClickHandler)up_single_click_handler);
  window_single_repeating_click_subscribe(
      BUTTON_ID_DOWN, kUpdateMs, (ClickHandler)down_single_click_handler);
  // We want to not do anything upon button holds so configure really long
  // repeat interval.
  window_single_repeating_click_subscribe(
      BUTTON_ID_SELECT, 65535, (ClickHandler)select_single_click_handler);
}

AccelData filter_accel(const AccelData* accel, AccelData* filter) {
  AccelData filtered_accel;
  const float kFilteringFactor = 0.1;
  filter->x = accel->x * kFilteringFactor + filter->x * (1 - kFilteringFactor);
  filtered_accel.x = accel->x - filter->x;
  filter->y = accel->y * kFilteringFactor + filter->y * (1 - kFilteringFactor);
  filtered_accel.y = accel->y - filter->y;
  filter->z = accel->z * kFilteringFactor + filter->z * (1 - kFilteringFactor);
  filtered_accel.z = accel->z - filter->z;
  return filtered_accel;
}

void handle_accel() {
  if (!falldown_settings.accelerometer_control) return;

  // Conversion from sensor data to g.
  const float kAccelToG = 1.0 / 1000;
  // Get raw accelerometer data, try to filter out constant acceleration (e.g.
  // gravity), and apply to circle velocity.
  AccelData accel;
  accel_service_peek(&accel);
  accel = filter_accel(&accel, &filter);
  float accel_g = accel.z * kAccelToG;
  // TODO(ariw): Good multiplier here? kCircleXMaxAccel?
  circle_x_velocity -= accel_g;
}

void get_mac(const char* game, int score, const char* nonce, char* mac) {
  char message[kBufferSize];
  int message_length;
  message_length = snprintf(
      message, kBufferSize, "%s%d%s", game, score, nonce);
  char binary_mac[SHA256_DIGEST_SIZE];
  hmac_sha256(
      (unsigned char*)kMacKey, kMacKeyLength, (unsigned char*)message,
      message_length, (unsigned char*)binary_mac, SHA256_DIGEST_SIZE);
  // Convert binary MAC to hexdigest.
  for (int i = 0; i < SHA256_DIGEST_SIZE; ++i) {
    snprintf(mac + i * 2, 3, "%02x", binary_mac[i]);
  }
}

void app_message_inbox_received(DictionaryIterator* iterator, void* context) {
  // Are we in a nonce callback or a score callback?
  Tuple* tuple = dict_find(iterator, 4);
  if (!tuple) return;
  char* nonce = tuple->value->cstring;
  static const char* kGameName = "Falldown2";
  char mac[SHA256_DIGEST_SIZE * 2 + 1];  // sha256 in hex and terminating \0.
  get_mac(kGameName, sent_score, nonce, (char*)mac);
  DictionaryIterator* body;
  app_message_outbox_begin(&body);
  dict_write_cstring(body, 0, "http://pebblescores.appspot.com/submit");
  dict_write_cstring(body, 1, kGameName);
  dict_write_int32(body, 2, (int32_t)sent_score);
  dict_write_cstring(body, 3, mac);
  dict_write_cstring(body, 4, nonce);
  app_message_outbox_send();
}

void send_score(int score) {
  DictionaryIterator* body;
  app_message_outbox_begin(&body);
  sent_score = score;
  dict_write_cstring(body, 0, "http://pebblescores.appspot.com/nonce");
  app_message_outbox_send();
}

// TODO(ariw): Merge this with the circle/line init functions.
void reset() {
  // Reset the score.
  score = 0;

  // Reset player circle.
  Circle* circle = layer_get_data(circle_layer);
  circle->x = kWidth / 2 - kCircleRadius;
  circle->y = 0;
  circle_x_velocity = 0;

  // Reset the lines.
  for (int i = 0; i < kLineCount; ++i) {
    Line* line = layer_get_data(line_layers[i]);
    line_generate(
        (kDistanceBetweenLines + kLineThickness) * (i + 2), line);
  }

  // Reset our speed.
  elapsed_time_ms = 0;
  lines_velocity = kInitialLineVelocity;
}

void handle_timer(void* data) {
  // Check to see if game is over yet.
  Circle* circle = layer_get_data(circle_layer);
  if (circle->y < 0) {
    send_score(score);
    reset();
    // Don't update the screen for a bit to let the user see their score after
    // a game over.
    app_timer_register(3000, (AppTimerCallback)handle_timer, NULL);
    return;
  }
  app_timer_register(kUpdateMs, (AppTimerCallback)handle_timer, NULL);

  if (in_menu) return;
  handle_accel();

  // Update the text.
  snprintf(text, kTextLength, "%d", score);
  text_layer_set_text(text_layer, text);

  // Update the player circle.
  bool intersects_x = false, intersects_y = false;
  lines_circle_intersect(
      circle_x_velocity, kCircleYVelocity - lines_velocity, &line_layers,
      circle_layer, &intersects_x, &intersects_y);
  if (intersects_x ||
      circle->x + circle_x_velocity < 0 ||
      circle->x + kCircleRadius * 2 + circle_x_velocity >= kWidth) {
    circle_x_velocity = 0;
  }
  circle->x += circle_x_velocity;
  if (!intersects_y &&
      circle->y + kCircleRadius * 2 + kCircleYVelocity <=
          kHeight - kStatusBarHeight) {
    // Fall down!
    circle->y += kCircleYVelocity;
  }
  if (intersects_y) {
    // Can't fall down yet, move up with the line.
    circle->y += lines_velocity;
  }
  layer_set_frame(circle_layer,
                  GRect((int)circle->x, (int)circle->y, kCircleRadius * 2,
                        kCircleRadius * 2));

  // Update the lines as they move upward.
  for (int i = 0; i < kLineCount; ++i) {
    Line* line = layer_get_data(line_layers[i]);
    line->y += lines_velocity;
    if (line->y < 0) {
      Line* base_line = layer_get_data(
          line_layers[common_mod(i - 1, kLineCount)]);
      line_generate(base_line->y + kDistanceBetweenLines + kLineThickness,
                    line);
      score += 10;
    }
    layer_set_frame(line_layers[i],
                    GRect(0, (int)line->y, kWidth, kLineThickness));
  }

  // Increase our speed sometimes.
  // TODO(ariw): Update this by actual elapsed number of ms so game time matches
  // real time.
  elapsed_time_ms += kUpdateMs;
  if (elapsed_time_ms % kVelocityIncreaseMs < kUpdateMs) {
    lines_velocity *= kVelocityIncrease;
  }
}

void handle_init() {
  srand(time(NULL));

  game_window = window_create();
  window_set_background_color(game_window, GColorBlack);
  window_stack_push(game_window, true);

  Layer* root_layer = window_get_root_layer(game_window);

  // Initialize AppMessage.
  app_message_register_inbox_received(
      (AppMessageInboxReceived)app_message_inbox_received);

  // Initialize the lines to fall down.
  lines_init(root_layer, &line_layers);

  // Initialize the score.
  text_layer = text_layer_create(GRect(0, 0, kWidth, kTextSize));
  text_layer_set_text_alignment(text_layer, GTextAlignmentRight);
  text_layer_set_background_color(text_layer, GColorClear);
  text_layer_set_text_color(text_layer, GColorWhite);
  layer_add_child(root_layer, (Layer*)text_layer);

  // Initialize the player circle.
  circle_init(root_layer, kWidth / 2 - kCircleRadius,  0, &circle_layer);

  // Attach our desired button functionality
  window_set_click_config_provider(
      game_window, (ClickConfigProvider)click_config_provider);

  app_message_open(kBufferSize, kBufferSize);

  init_settings();

  // Start updating the game.
  app_timer_register(kUpdateMs, (AppTimerCallback)handle_timer, NULL);
}

void handle_deinit() {
  // Unsubscribe from used services.
  accel_data_service_unsubscribe();

  // Pop all windows off.
  window_stack_pop_all(true);

  // Clear all memory.
  window_destroy(game_window);
  for (int i = 0; i < kLineCount; ++i) {
    layer_destroy(line_layers[i]);
  }
  text_layer_destroy(text_layer);
  layer_destroy(circle_layer);
  deinit_settings();
}

int main(void) {
  handle_init();
  app_event_loop();
  handle_deinit();
}

