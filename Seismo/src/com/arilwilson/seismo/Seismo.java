package com.arilwilson.seismo;

import android.app.Activity;
import android.os.Bundle;
import android.widget.LinearLayout;

public class Seismo extends Activity {
  @Override
  public void onCreate(Bundle savedInstanceState) {
	super.onCreate(savedInstanceState);
	LinearLayout layout = new LinearLayout(this);
	view_ = new SeismoView(this, 25);
	layout.addView(view_);
	
	setContentView(layout);
    createUpdater();
  }

  @Override
  public void onDestroy() {
    super.onDestroy();
	destroyUpdater();
  }

  private void destroyUpdater() {
    updater_.destroy();
	try {
	  updater_thread_.join();
	} catch (InterruptedException e) {
	  // Ignore.
    }
  }

  private void createUpdater() {
	AccelerometerReader reader = new AccelerometerReader(this);
	updater_ = new AccelerometerUpdater(reader, view_, 25);
    updater_thread_ = new Thread(updater_);
    updater_thread_.start();
  }

  private class AccelerometerUpdater implements Runnable {
    public AccelerometerUpdater(AccelerometerReader reader, SeismoView view,
    		                    int updater_period) {
      stop_ = false;
      reader_ = reader;
      view_ = view;
      updater_period_ = updater_period;
    }

    public void run() {
      while (!stop_) {
        view_.update(reader_.x, reader_.y, reader_.z);
        try {
          Thread.sleep(updater_period_, 0);
        } catch (Exception e) {
          // Ignore.
        }
      }
    }
    
    public void destroy() {
      stop_ = true;
    }

    private volatile AccelerometerReader reader_;
    private int updater_period_;
    private volatile boolean stop_;
  }

  private AccelerometerUpdater updater_;
  private SeismoView view_;
  private Thread updater_thread_;
}