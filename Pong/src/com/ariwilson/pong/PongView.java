package com.ariwilson.pong;

import android.content.Context;
import android.graphics.Canvas;
import android.view.SurfaceHolder;
import android.view.SurfaceView;

public class PongView extends SurfaceView implements SurfaceHolder.Callback {
  public PongView(Context ctx) {
    super(ctx);
    holder_ = getHolder();
    holder_.addCallback(this);
    setKeepScreenOn(true);
    ctx_ = ctx;
  } 

  public void surfaceChanged(SurfaceHolder holder, int format, int width,
      int height) {
    width_ = width;
    height_ = height;
  }

  public void surfaceCreated(SurfaceHolder holder) {
    running_ = true;
    thread_ = new PongViewThread();
    thread_.start();
  }

  public void surfaceDestroyed(SurfaceHolder holder) {
    running_ = false;
    try {
      thread_.join();
    } catch (InterruptedException e) {
      // Do nothing.
    }
  }

  private class PongViewThread extends Thread {
    @Override
    public void run() {
      while (running_) {
        synchronized (holder_) {
          Canvas canvas = holder_.lockCanvas();
          canvas.drawARGB(255, 255, 255, 255);
          holder_.unlockCanvasAndPost(canvas);
        }
      }
    }
  }

  private boolean running_ = false;
  private int width_;
  private int height_;
  private PongViewThread thread_;
  private SurfaceHolder holder_;
  private Context ctx_;
}
