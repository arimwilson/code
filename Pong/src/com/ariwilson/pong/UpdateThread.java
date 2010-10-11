package com.ariwilson.pong;

import java.util.Vector;

public class UpdateThread extends InterruptibleThread {
  public UpdateThread(
      Vector<GameObject> objects) {
    last_update_ = System.currentTimeMillis();
    objects_ = objects;
  }

  @Override
  public void work() {
    long time = System.currentTimeMillis();
    for (GameObject object : objects_) {
      object.update(time - last_update_);
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
  private Vector<GameObject> objects_;
}
