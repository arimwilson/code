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
    ctx_ = ctx;
    period_ = period;
    resume();
  }
  
  public void pause() {
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

  public void resume() {
    AccelerometerReader reader = new AccelerometerReader(ctx_);
    view_thread_ = new SeismoViewThread(getHolder(), ctx_, period_);
    reader_thread_ = new AccelerometerReaderThread(reader, view_thread_,
                                                   period_);
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

  private AccelerometerReaderThread reader_thread_;
  private SeismoViewThread view_thread_;
  private Context ctx_;
  private int period_;
}
