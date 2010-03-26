package com.ariwilson.pong;

public interface GameComponent {
  public void update(GameObject object, long millis);
  public void draw(GameObject object);
}
