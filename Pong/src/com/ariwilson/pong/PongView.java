package com.ariwilson.pong;

import java.util.Vector;
import java.util.concurrent.ArrayBlockingQueue;

import android.content.Context;
import android.view.SurfaceHolder;
import android.view.SurfaceView;

public class PongView extends SurfaceView implements SurfaceHolder.Callback {
  public PongView(Context ctx) {
    super(ctx);
    getHolder().addCallback(this);
    setKeepScreenOn(true);

    components_ = new Vector<GameComponent>(1);
    objects_ = new Vector<GameObject>(3);
    updated_objects_ = new ArrayBlockingQueue<GameObject>(3);
  } 

  public void surfaceChanged(SurfaceHolder holder, int format, int width,
      int height) {
    draw_thread_.setHolder(holder);
  }

  public void surfaceCreated(SurfaceHolder holder) {
    draw_thread_ = new DrawThread(getHolder(), components_, updated_objects_);
    update_thread_ = new UpdateThread(components_, objects_, updated_objects_);
    draw_thread_.start();
    update_thread_.start();
  }

  public void surfaceDestroyed(SurfaceHolder holder) {
    draw_thread_.interrupt();
    update_thread_.interrupt();
    try {
      draw_thread_.join();
      update_thread_.join();
    } catch (InterruptedException e) {
      // Do nothing.
    }
  }

  private DrawThread draw_thread_;
  private UpdateThread update_thread_;

  // Game logic-related.
  private Vector<GameComponent> components_;
  private Vector<GameObject> objects_;
  private ArrayBlockingQueue<GameObject> updated_objects_;
}
