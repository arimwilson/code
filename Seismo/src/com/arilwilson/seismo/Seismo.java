package com.arilwilson.seismo;

import android.app.Activity;
import android.os.Bundle;
import android.os.Handler;
import android.os.Message;
import android.util.Log;
import android.widget.TextView;

public class Seismo extends Activity {
  @Override
  public void onCreate(Bundle savedInstanceState) {
    super.onCreate(savedInstanceState);
    AccelerometerReader reader = new AccelerometerReader(this);
    direction_view_ = new TextView(this);
    updater_ = new AccelerometerUpdater(reader, ui_updater_);
    setContentView(direction_view_);
    updater_thread_ = new Thread(updater_);
    updater_thread_.start();
  }

  @Override
  public void onDestroy() {
    updater_.destroy();
    try {
      updater_thread_.join();
    } catch (InterruptedException e) {
      // Ignore.
    }
  }

  private class AccelerometerUpdater implements Runnable {
    public AccelerometerUpdater(AccelerometerReader reader, Handler ui_updater) {
      stop_ = false;
      reader_ = reader;
      ui_updater_ = ui_updater;
    }

    public void run() {
      while (!stop_) {
        Bundle b = new Bundle();
        Message m = new Message();
        m.setData(b);
        b.putString("action", "update");
        b.putDouble("direction", reader_.direction);
        ui_updater_.sendMessage(m);
        try {
            Thread.sleep(1000, 0);
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
    private volatile boolean stop_;
  }

  private Handler ui_updater_ = new Handler() {
    @Override
    public void handleMessage(Message m) {
      Bundle b = m.getData();
      if (b != null) {
        if (b.getString("action") == "update") {
          direction_view_.setText(String.valueOf(b.getDouble("direction")));
        }
      }
    }
  };

  private TextView direction_view_;
  private AccelerometerUpdater updater_;
  private Thread updater_thread_;
}