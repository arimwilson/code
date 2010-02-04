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
    draw_thread_ = new DrawThread();
    update_thread_ = new UpdateThread();
    draw_thread_.start();
    update_thread_.start();
  }

  public void surfaceDestroyed(SurfaceHolder holder) {
    running_ = false;
    try {
      draw_thread_.join();
    } catch (InterruptedException e) {
      // Do nothing.
    }
  }

  private class DrawThread extends Thread {
    @Override
    public void run() {
      while (running_) {
        try {
          ctx_.wait(UPDATE_MS * 2);
        } catch (InterruptedException e) {
          // Do nothing.
        }
        synchronized (holder_) {
          Canvas canvas = holder_.lockCanvas();
          canvas.drawARGB(255, 255, 255, 255);
          // TODO(ariw): Draw all objects.
          holder_.unlockCanvasAndPost(canvas);
        }
      }
    }
  }

  private class UpdateThread extends Thread {
    @Override
    public void run() {
      while (running_) {
        long start_time = System.currentTimeMillis();
        // TODO(ariw): Update all objects.
        ctx_.notify();
        try {
          sleep(UPDATE_MS - System.currentTimeMillis() + start_time);
        } catch (InterruptedException e) {
          // Do nothing.
        }
      }
    }
  }

  public static final int UPDATE_MS = 33;  // ~30 updates/s  

  private boolean running_ = false;
  private int width_;
  private int height_;
  private DrawThread draw_thread_;
  private UpdateThread update_thread_;
  private SurfaceHolder holder_;
  private Context ctx_;
}
