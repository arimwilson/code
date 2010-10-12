package com.ariwilson.pong;

import java.util.LinkedList;

public class GameObject extends BaseObject {
  public void addComponent(BaseObject object) {
    objects_.add(object);
  }

  public void update(long millis, BaseObject parent) {
    for (BaseObject component : objects_) {
      component.update(millis, this);
    }
  }

  private LinkedList<BaseObject> objects_;
}
