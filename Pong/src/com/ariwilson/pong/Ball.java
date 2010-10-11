package com.ariwilson.pong;

public class Ball extends GameObject {
  public Ball(int x, int y) {
    x_ = x;
    y_ = y;
  }

  @Override
  public void update(long millis) {
  }

  protected int x_;
  protected int y_;
}
