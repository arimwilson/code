package com.arilwilson.seismo;

import android.content.Context;
import android.graphics.Canvas;
import android.graphics.Paint;
import android.view.SurfaceHolder;

public class SeismoViewThread extends Thread {
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
        float[] pts = new float[canvas_height_ * 2];
        for (int i = 0; i < canvas_height_; ++i) {
          pts[i * 2] = canvas_width_ / 2 * (1 + z_[i] / MAX_ACCELERATION);
          pts[i * 2 + 1] = i;
        }
        Paint paint = new Paint();
        paint.setARGB(255, 0, 0, 0);
        paint.setStrokeWidth(2.0f);
        paint.setAntiAlias(false);
        canvas.drawLines(pts, paint);
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
      if (filter_) {
        acceleration[0] = x * FILTERING_FACTOR +
                          acceleration[0] * (1.0f - FILTERING_FACTOR);
        x_[next_index_] = x - acceleration[0];
        acceleration[1] = y * FILTERING_FACTOR +
                          acceleration[1] * (1.0f - FILTERING_FACTOR);
        y_[next_index_] = y - acceleration[1];
        acceleration[2] = z * FILTERING_FACTOR +
                          acceleration[2] * (1.0f - FILTERING_FACTOR);
        z_[next_index_] = z - acceleration[2];
      } else {
        x_[next_index_] = x;
        y_[next_index_] = y;
        z_[next_index_] = z;
      }
      next_index_ = (next_index_ + 1) % canvas_height_;
    }
  }

  public void setSurfaceSize(int canvas_width, int canvas_height) {
    synchronized (holder_) {
      canvas_width_ = canvas_width;
      canvas_height_ = canvas_height;
      x_ = new float[canvas_height];
      y_ = new float[canvas_height];
      z_ = new float[canvas_height];
      next_index_ = 0;
    }
  }
  
  public void setFilter(boolean filter) {
    filter_ = filter;
  }

  private static final float MAX_ACCELERATION = 2.0f * 9.806f;
  private static final float FILTERING_FACTOR = 0.1f;

  private float[] acceleration = new float[3];
  private float[] x_;
  private float[] y_;
  private float[] z_;
  private int next_index_;
  private int canvas_height_ = 0;
  private int canvas_width_ = 1;
  private boolean filter_ = false;
  private boolean running_ = false;
  private SurfaceHolder holder_;
  private Context ctx_;
  private int period_;
}
