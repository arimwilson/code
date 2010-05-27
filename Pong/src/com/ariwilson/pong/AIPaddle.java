package com.ariwilson.pong;

import android.graphics.Color;

public class AIPaddle extends Paddle {
  public AIPaddle(int x, int y) {
    super(x, y);
    paint_.setColor(Color.RED);
  }

  @Override
  public void update(long millis) {
    // TODO(ariw): AI logic!
  }
}
