package com.arilwilson.seismo;

import android.app.Activity;
import android.os.Bundle;
import android.widget.TextView;

public class Seismo extends Activity {
  private class AccelerometerThread implements Runnable {
    private volatile AccelerometerReader reader_;
    private volatile TextView tv_;

    public AccelerometerThread(AccelerometerReader reader, TextView tv) {
      reader_ = reader;
      tv_ = tv;
    }

    public void run() {
      while (true) {
        tv_.setText(String.valueOf(reader_.direction));
        try {
            Thread.sleep(1000, 0);
        } catch (Exception e) {
            // Ignore.
        }
      }
    }
  }

  @Override
  public void onCreate(Bundle savedInstanceState) {
    super.onCreate(savedInstanceState);
    AccelerometerReader reader = new AccelerometerReader(this);
    TextView tv = new TextView(this);
    AccelerometerThread thread = new AccelerometerThread(reader, tv);
    setContentView(tv);
    new Thread(thread).start();
  }
}