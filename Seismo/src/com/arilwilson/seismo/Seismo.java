package com.arilwilson.seismo;

import android.app.Activity;
import android.os.Bundle;
import android.os.Handler;
import android.os.Message;
import android.widget.TextView;

public class Seismo extends Activity {
  @Override
  public void onCreate(Bundle savedInstanceState) {
	super.onCreate(savedInstanceState);
    AccelerometerReader reader = new AccelerometerReader(this);
    view_ = new TextView(this);
    updater_ = new AccelerometerUpdater(reader, ui_updater_, 1000);
    setContentView(view_);
    updater_thread_ = new Thread(updater_);
    updater_thread_.start();
  }

  @Override
  public void onPause() {
    super.onPause();
  }

  @Override
  public void onResume() {
    super.onResume();
  }

  @Override
  public void onStop() {
    super.onStop();
  }

  @Override
  public void onDestroy() {
    super.onDestroy();
	updater_.destroy();
    try {
      updater_thread_.join();
    } catch (InterruptedException e) {
      // Ignore.
    }
  }

  private class AccelerometerUpdater implements Runnable {
    public AccelerometerUpdater(AccelerometerReader reader, Handler ui_updater,
    		                    int updater_period) {
      stop_ = false;
      reader_ = reader;
      ui_updater_ = ui_updater;
      updater_period_ = updater_period;
    }

    public void run() {
      while (!stop_) {
        Bundle b = new Bundle();
        Message m = new Message();
        m.setData(b);
        b.putString("action", "update");
        b.putDouble("x", reader_.x);
        b.putDouble("y", reader_.y);
        b.putDouble("z", reader_.z);
        ui_updater_.sendMessage(m);
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
    private Handler ui_updater_;
    private int updater_period_;
    private volatile boolean stop_;
  }

  private Handler ui_updater_ = new Handler() {
    @Override
    public void handleMessage(Message m) {
      Bundle b = m.getData();
      if (b != null) {
        if (b.getString("action") == "update") {
          view_.setText(String.valueOf(b.getDouble("x")) + " " +
        		        String.valueOf(b.getDouble("y")) + " " +
        		        String.valueOf(b.getDouble("z")));
        }
      }
    }
  };

  private TextView view_;
  private AccelerometerUpdater updater_;
  private Thread updater_thread_;
}