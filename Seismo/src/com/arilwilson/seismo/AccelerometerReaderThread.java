package com.arilwilson.seismo;

public class AccelerometerReaderThread extends Thread {
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
