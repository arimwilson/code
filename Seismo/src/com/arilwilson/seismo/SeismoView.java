package com.arilwilson.seismo;

import android.content.Context;
import android.graphics.Canvas;
import android.view.SurfaceHolder;
import android.view.SurfaceView;

public class SeismoView extends SurfaceView implements SurfaceHolder.Callback {
  public SeismoView(Context ctx, int period) {
    super(ctx);

    SurfaceHolder holder = getHolder();
    holder.addCallback(this);
    AccelerometerReader reader = new AccelerometerReader(ctx);
    view_thread_ = new SeismoViewThread(holder, ctx, period);
    reader_thread_ = new AccelerometerReaderThread(reader, view_thread_,
                                                   period);
  }
  
  public void surfaceChanged(SurfaceHolder holder, int format, int width,
                             int height) {
    view_thread_.setSurfaceSize(width, height);
  }

  public void surfaceCreated(SurfaceHolder holder) {
    view_thread_.setRunning(true);
    view_thread_.start();
    reader_thread_.setRunning(true);
    reader_thread_.start();
  }

  public void surfaceDestroyed(SurfaceHolder holder) {
    boolean retry = true;
    view_thread_.setRunning(false);
    reader_thread_.setRunning(false);
    while (retry) {
      try {
        view_thread_.join();
        reader_thread_.join();
        retry = false;
      } catch (InterruptedException e) {
      }
    }
  }

  private class AccelerometerReaderThread extends Thread {
    public AccelerometerReaderThread(AccelerometerReader reader,
                                     SeismoViewThread view,
                                     int updater_period) {
      reader_ = reader;
      view_ = view;
      updater_period_ = updater_period;
    }

    @Override
    public void run() {
      while (running_) {
        view_.update(reader_.x, reader_.y, reader_.z);
        try {
          Thread.sleep(updater_period_, 0);
        } catch (Exception e) {
          // Ignore.
        }
      }
    }
    
    public void setRunning(boolean running) {
      running_ = running;
    }

    private boolean running_ = false;
    private volatile AccelerometerReader reader_;
    private SeismoViewThread view_;
    private int updater_period_;
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
          canvas.clipRect(
              canvas_width_ / 2 - 1, 0, canvas_width_ / 2 + 1,
              canvas_height_ * (z_[cur_index_] / MAX_ACCELERATION));
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
        x_[next_index_] = x;
        y_[next_index_] = y;
        z_[next_index_] = z;
        cur_index_ = next_index_;
        next_index_ = (next_index_ + 1) % HISTORY_SIZE;
      }
    }

    public void setSurfaceSize(int canvas_width, int canvas_height) {
      synchronized (holder_) {
        canvas_width_ = canvas_width;
        canvas_height_ = canvas_height;
      }
    }

    private float[] x_ = new float[HISTORY_SIZE];
    private float[] y_ = new float[HISTORY_SIZE];
    private float[] z_ = new float[HISTORY_SIZE];
    private int cur_index_ = 0;
    private int next_index_ = 0;
    private int canvas_height_ = 1;
    private int canvas_width_ = 1;
    private boolean running_ = false;
    private SurfaceHolder holder_;
    private Context ctx_;
    private int period_;
  }

  private static final int HISTORY_SIZE = 100;
  private static final float MAX_ACCELERATION = 3.0f;

  private AccelerometerReaderThread reader_thread_;
  private SeismoViewThread view_thread_;
}
