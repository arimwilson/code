package com.ariwilson.pong;

import java.util.LinkedList;

public class GameObject {
  public GameObject(LinkedList<GameComponent> components) {
    components_ = components;
  }

  public void update(long millis) {
    for (GameComponent component : components_) {
      component.update(millis, this);
    }
  }

  private LinkedList<GameComponent> components_;
}
