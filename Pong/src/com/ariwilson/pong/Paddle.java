package com.ariwilson.pong;

import android.graphics.Canvas;

public abstract class Paddle implements GameObject {
  public Paddle(int x, int y) {
    x_ = x;
    y_ = y;
  }

  @Override
  public void draw(Canvas canvas) {
    // TODO(ariw): Draw!
  }

  // TODO(ariw): Adjust based on screen size.
  protected static final int HEIGHT = 20;
  protected static final int WIDTH = 5;

  protected int x_;
  protected int y_;
}
