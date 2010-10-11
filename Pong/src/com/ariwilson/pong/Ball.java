package com.ariwilson.pong;

import java.util.LinkedList;

public class Ball extends GameObject {
  public Ball(LinkedList<GameComponent> components, int x, int y) {
    super(components);
    x_ = x;
    y_ = y;
  }

  @Override
  public void update(long millis) {
  }

  protected int x_;
  protected int y_;
}
