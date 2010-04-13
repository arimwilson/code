package com.ariwilson.pong;

import java.util.Vector;
import java.util.concurrent.ArrayBlockingQueue;

public class UpdateThread extends InterruptibleThread {
  public UpdateThread(
      Vector<GameComponent> components,
      Vector<GameObject> objects,
      ArrayBlockingQueue<GameObject> updated_objects) {
    last_update_ = System.currentTimeMillis();
    components_ = components;
    objects_ = objects;
    updated_objects_ = updated_objects;
  }

  @Override
  public void work() {
    long time = System.currentTimeMillis();
    for (GameObject object : objects_) {
      for (GameComponent component : components_) {
        component.update(object, time - last_update_);
      }
      object.update(time - last_update_);
      // TODO(ariw): Shouldn't need to always push objects onto the
      // to-be-updated list.
      updated_objects_.add(object);
    }
    last_update_ = time;
    try {
      sleep(UPDATE_MS - System.currentTimeMillis() + time);
    } catch (InterruptedException e) {
      // Do nothing.
    }
  }

  public static final int UPDATE_MS = 33;  // ~30 updates/s

  private long last_update_;
  private Vector<GameComponent> components_;
  private Vector<GameObject> objects_;
  private ArrayBlockingQueue<GameObject> updated_objects_;
}
