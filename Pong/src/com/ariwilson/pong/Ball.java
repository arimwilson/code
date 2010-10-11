package com.ariwilson.pong;

public class Ball extends GameObject {
  public Ball(int x, int y) {
    // TODO(ariw): Adjustable based on screen size.
    radius_ = 10;

    x_ = x;
    y_ = y;
  }

  protected int radius_; 

  protected int x_;
  protected int y_;
}
