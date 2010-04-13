package com.ariwilson.pong;

import android.graphics.Canvas;
import android.view.SurfaceHolder;

import java.util.LinkedList;
import java.util.List;
import java.util.Vector;
import java.util.concurrent.ArrayBlockingQueue;

public class DrawThread extends InterruptibleThread {
  public DrawThread(
      SurfaceHolder holder,
      Vector<GameComponent> components,
      ArrayBlockingQueue<GameObject> updated_objects) {
    holder_ = holder;
    components_ = components;
    updated_objects_ = updated_objects;
  }

  @Override
  public void work() {
    synchronized (holder_) {
      Canvas canvas = holder_.lockCanvas();
      // Draw background.
      canvas.drawARGB(255, 255, 255, 255);
      // TODO(ariw): Draw dotted black line across middle of screen.

      // Retrieve updated objects from queue, blocking until at least one is
      // available.
      List<GameObject> objects = new LinkedList<GameObject>();
      objects.add(updated_objects_.poll());
      updated_objects_.drainTo(objects);

      // Draw all updated objects.
      for (GameObject object : objects) {
        for (GameComponent component : components_) {
          component.draw(object, canvas);
        }
        object.draw(canvas);
      }
      holder_.unlockCanvasAndPost(canvas);
    }
  }

  // Used to change holder if surface changes.
  public void setHolder(SurfaceHolder holder) {
    synchronized (holder_) {
      holder_ = holder;
    }
  }

  private SurfaceHolder holder_;
  private Vector<GameComponent> components_;
  private ArrayBlockingQueue<GameObject> updated_objects_;
}
