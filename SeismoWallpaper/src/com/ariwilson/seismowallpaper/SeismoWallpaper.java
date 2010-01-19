package com.ariwilson.seismowallpaper;

import android.content.Context;
import android.service.wallpaper.WallpaperService;
import android.view.SurfaceHolder;

public class SeismoWallpaper extends WallpaperService {

  @Override
  public Engine onCreateEngine() {
    return new SeismoEngine(33);
  }

  private class SeismoEngine extends Engine {
    SeismoEngine(int period) {
      ctx_ = getApplicationContext();
      period_ = period;
    }

    @Override
    public void onVisibilityChanged(boolean visible) {
      super.onVisibilityChanged(visible);
      if (visible) {
        view_thread_.setPaused(false);
        reader_thread_.setPaused(false);        
      } else {
        view_thread_.setPaused(true);
        reader_thread_.setPaused(true);
      }
    }

    @Override
    public void onSurfaceChanged(SurfaceHolder holder, int format, int width,
                                 int height) {
      view_thread_.setSurfaceSize(width, height);
    }

    @Override
    public void onSurfaceCreated(SurfaceHolder holder) {
      AccelerometerReader reader = new AccelerometerReader(ctx_);
      view_thread_ = new SeismoViewThread(ctx_, holder, filter_, axis_, period_);
      reader_thread_ = new AccelerometerReaderThread(reader, view_thread_,
                                                     paused_, period_);
      view_thread_.start();
      reader_thread_.start();
    }

    @Override
    public void onSurfaceDestroyed(SurfaceHolder holder) {
      view_thread_.setRunning(false);
      reader_thread_.setRunning(false);
      try {
        view_thread_.join();
        reader_thread_.join();
      } catch (InterruptedException e) {
        // Do nothing.
      }
    }

    private AccelerometerReaderThread reader_thread_;
    private SeismoViewThread view_thread_;
    private boolean paused_ = false;
    private boolean filter_ = true;
    private int axis_ = 2;
    private Context ctx_;
    private int period_; 
  }
}
