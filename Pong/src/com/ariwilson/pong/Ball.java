package com.ariwilson.pong;

import android.graphics.Canvas;
import android.graphics.Color;
import android.graphics.Paint;

public class Ball implements GameObject {
  public Ball(int x, int y) {
    // TODO(ariw): Adjust radius based on screen size.
    radius_ = 10;
    paint_ = new Paint();
    paint_.setColor(Color.BLUE);

    x_ = x;
    y_ = y;
  }

  @Override
  public void update(long millis) {
  }

  @Override
  public void draw(Canvas canvas) {
    canvas.drawCircle(x_, y_, radius_, paint_);
  }

  protected int radius_;
  protected Paint paint_;

  protected int x_;
  protected int y_;
}
