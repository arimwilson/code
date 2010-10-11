package com.ariwilson.pong;

import java.util.LinkedList;

public class GameObject {
  public void addComponent(GameComponent component) {
    components_.add(component);
  }

  public void update(long millis) {
    for (GameComponent component : components_) {
      component.update(millis, this);
    }
  }

  private LinkedList<GameComponent> components_;
}
