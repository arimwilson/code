package com.arilwilson.seismo;

import android.content.Context;
import android.graphics.Canvas;
import android.view.SurfaceHolder;
import android.view.SurfaceView;

public class SeismoView extends SurfaceView implements SurfaceHolder.Callback {
  public SeismoView(Context ctx, int period) {
    super(ctx);

    // register our interest in hearing about changes to our surface
    SurfaceHolder holder = getHolder();
    holder.addCallback(this);
    thread_ = new SeismoViewThread(holder, ctx, period);
  }
  
  public void surfaceChanged(SurfaceHolder holder, int format, int width,
                             int height) {
    thread_.setSurfaceSize(width, height);
  }

  public void surfaceCreated(SurfaceHolder holder) {
    thread_.setRunning(true);
    thread_.start();
  }

  public void surfaceDestroyed(SurfaceHolder holder) {
    boolean retry = true;
    thread_.setRunning(false);
    while (retry) {
      try {
        thread_.join();
        retry = false;
      } catch (InterruptedException e) {
      }
    }
  }

  public void update(float x, float y, float z) {
    thread_.update(x, y, z);
  }

  private class SeismoViewThread extends Thread {
    public SeismoViewThread(SurfaceHolder holder, Context ctx, int period) {
      holder_ = holder;
      ctx_ = ctx;
      period_ = period;
    }

    @Override
    public void run() {
      while (running_) {
        synchronized (holder_) {
          Canvas canvas = holder_.lockCanvas();
          canvas.drawARGB(255, 255, 255, 255);
          canvas.clipRect(0, 0, 50, 50);
          canvas.drawARGB(255, 0, 0, 0);
          holder_.unlockCanvasAndPost(canvas);
        }
        try {
          Thread.sleep(period_);
        } catch (Exception e) {
          // Do nothing.
        }
      }
    }

    public void setRunning(boolean running) {
      running_ = running;
    }
    
    public void update(float x, float y, float z) {
      synchronized (holder_) {
        x_[0] = x;
        y_[0] = y;
        z_[0] = z;
      }
    }

    public void setSurfaceSize(int canvas_height, int canvas_width) {
      synchronized (holder_) {
        canvas_height_ = canvas_height;
        canvas_width_ = canvas_width;
      }
    }

    private float[] x_ = new float[100];
    private float[] y_ = new float[100];
    private float[] z_ = new float[100];

    private int canvas_height_ = 1;
    private int canvas_width_ = 1;
    private boolean running_ = false;
    private SurfaceHolder holder_;
    private Context ctx_;
    private int period_;
  }
  
  private SeismoViewThread thread_;
}
