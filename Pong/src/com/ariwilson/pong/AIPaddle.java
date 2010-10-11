package com.ariwilson.pong;

import java.util.LinkedList;

public class AIPaddle extends Paddle {
  public AIPaddle(LinkedList<GameComponent> components, int x, int y) {
    super(components, x, y);
  }

  @Override
  public void update(long millis) {
    // TODO(ariw): AI logic!
  }
}
