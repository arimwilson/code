package com.ariwilson.pong;

import java.util.Vector;

import android.content.Context;
import android.view.MotionEvent;
import android.view.SurfaceHolder;
import android.view.SurfaceView;

public class PongView extends SurfaceView implements SurfaceHolder.Callback {
  public PongView(Context ctx) {
    super(ctx);
    getHolder().addCallback(this);
    setKeepScreenOn(true);

    objects_ = new Vector<GameObject>(3);
    objects_.add(new AIPaddle(0, 0));  // TODO(ariw): Fix coords.
    objects_.add(new AIPaddle(1, 1));  // TODO(ariw): Fix coords.
    objects_.add(new Ball(2, 2));  // TODO(ariw): Fix coords.
  } 

  public void surfaceChanged(SurfaceHolder holder, int format, int width,
      int height) {
  }

  public void surfaceCreated(SurfaceHolder holder) {
    update_thread_ = new UpdateThread(objects_);
    update_thread_.start();
  }

  public void surfaceDestroyed(SurfaceHolder holder) {
    update_thread_.interrupt();
    try {
      update_thread_.join();
    } catch (InterruptedException e) {
      // Do nothing.
    }
  }

  @Override
  public boolean onTouchEvent(MotionEvent motion) {
    // TODO(ariw): Pass touch events down to appropriate game objects.
    return true;
  }

  private UpdateThread update_thread_;

  // Game logic-related.
  private Vector<GameObject> objects_;
}
