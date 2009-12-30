package com.arilwilson.seismo;

import android.content.Context;
import android.graphics.Canvas;
import android.graphics.Paint;
import android.view.SurfaceHolder;

public class SeismoViewThread extends Thread {
  public SeismoViewThread(SurfaceHolder holder, Context ctx, boolean paused,
                          boolean filter, int axis, int period) {
    holder_ = holder;
    ctx_ = ctx;
    setPaused(paused);
    setFilter(filter);
    setAxis(axis);
    period_ = period;
  }

  @Override
  public void run() {
    while (running_) {
      synchronized (holder_) {
        Canvas canvas = holder_.lockCanvas();
        canvas.drawARGB(255, 255, 255, 255);

        Paint scale_paint = new Paint();
        scale_paint.setARGB(255, 137, 137, 137);
        scale_paint.setStrokeWidth(canvas_width_ / 300f);
        scale_paint.setAntiAlias(true);
        float text_size = canvas_width_ / 35f;
        scale_paint.setTextSize(text_size);
        
        // Draw g scale.
        scale_paint.setTextAlign(Paint.Align.CENTER);
        for (int i = -MAX_G + 1; i <= MAX_G - 1; ++i) {
          float x = canvas_width_ / 2 * (1 + (float)i / MAX_G);
          canvas.drawLine(x, 0, x, canvas_height_ / 20, scale_paint);
          canvas.drawText(Integer.toString(i) + "g", x,
                          canvas_height_ / 20 + 1.2f * text_size,
                          scale_paint);
        }

        // Draw time scale in seconds.
        scale_paint.setTextAlign(Paint.Align.LEFT);
        int num_seconds = canvas_height_ * period_ / 1000;
        for (int s = 1; s < num_seconds; ++s) {
          float y = canvas_height_ * (float)s / num_seconds;
          canvas.drawLine(0, y, canvas_width_ / 20, y, scale_paint);
          canvas.drawText(Integer.toString(s) + "s",
                          canvas_width_ / 20 + 0.2f * text_size,
                          y + 0.5f * text_size, scale_paint);
        }


        // Draw line.
        float[] pts = new float[next_index_ * 4];
        for (int i = 1; i < next_index_; ++i) {
          pts[i * 4] = canvas_width_ / 2 *
                           (1 + history_[i - 1][axis_] / MAX_ACCELERATION);
          pts[i * 4 + 1] = i - 1;
          pts[i * 4 + 2] = canvas_width_ / 2 *
                               (1 + history_[i][axis_] / MAX_ACCELERATION);
          pts[i * 4 + 3] = i;
        }
        Paint line_paint = new Paint();
        line_paint.setARGB(255, 0, 0, 0);
        line_paint.setStrokeWidth(canvas_width_ / 300f);
        line_paint.setAntiAlias(false);
        canvas.drawLines(pts, line_paint);
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
        history_[next_index_][0] = x - acceleration[0];
        acceleration[1] = y * FILTERING_FACTOR +
                          acceleration[1] * (1.0f - FILTERING_FACTOR);
        history_[next_index_][1] = y - acceleration[1];
        acceleration[2] = z * FILTERING_FACTOR +
                          acceleration[2] * (1.0f - FILTERING_FACTOR);
        history_[next_index_][2] = z - acceleration[2];
      } else {
        history_[next_index_][0] = x;
        history_[next_index_][1] = y;
        history_[next_index_][2] = z;
      }
      next_index_ = (next_index_ + 1) % canvas_height_;
    }
  }

  public void setSurfaceSize(int canvas_width, int canvas_height) {
    synchronized (holder_) {
      canvas_width_ = canvas_width;
      canvas_height_ = canvas_height;
      history_ = new float[canvas_height][3];
      next_index_ = 0;
    }
  }

  public void setPaused(boolean paused) {
    paused_ = paused;
  }

  public void setFilter(boolean filter) {
    filter_ = filter;
  }
  
  public void setAxis(int axis) {
    axis_ = axis;
  }

  private static final int MAX_G = 3;
  private static final float MAX_ACCELERATION = MAX_G * 9.807f;
  private static final float FILTERING_FACTOR = 0.1f;

  private float[] acceleration = new float[3];
  private float[][] history_ = new float[1][3];
  private int next_index_ = 0;
  private int canvas_height_ = 1;
  private int canvas_width_ = 1;
  private boolean running_;
  private boolean paused_;
  private boolean filter_;
  private int axis_ = 2;
  private SurfaceHolder holder_;
  private Context ctx_;
  private int period_;
}
