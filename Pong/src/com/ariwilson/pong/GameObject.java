package com.ariwilson.pong;

import android.graphics.Canvas;

public interface GameObject {
  public void update(long millis);
  public void draw(Canvas canvas); 
}
