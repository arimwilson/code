package com.ariwilson.pong;

import android.graphics.Canvas;

public interface GameComponent {
  public void update(GameObject object, long millis);
  public void draw(GameObject object, Canvas canvas);
}
