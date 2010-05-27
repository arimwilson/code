package com.ariwilson.pong;

import android.graphics.Canvas;
import android.graphics.Paint;

public abstract class Paddle implements GameObject {
  public Paddle(int x, int y) {
    // TODO(ariw): Adjust width/height based on screen size.
    height_ = 20;
    width_ = 5;
    paint_ = new Paint();

    x_ = x;
    y_ = y;
  }

  @Override
  public void draw(Canvas canvas) {
    canvas.drawRect(x_, y_, x_ + width_, y_ + height_, paint_);
  }

  protected int height_;
  protected int width_;
  protected Paint paint_;

  protected int x_;
  protected int y_;
}
