package com.ariwilson.pong;

public abstract class BaseObject {
  static public ObjectRegistry registry = new ObjectRegistry();

  public abstract void update(long millis, BaseObject parent);
}
