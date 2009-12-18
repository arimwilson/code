package com.arilwilson.seismo;

import android.app.Activity;
import android.os.Bundle;
import android.widget.TextView;

public class Seismo extends Activity {
  /** Called when the activity is first created. */
  @Override
  public void onCreate(Bundle savedInstanceState) {
    super.onCreate(savedInstanceState);
    TextView tv = new TextView(this);
    AccelerometerReader reader = new AccelerometerReader(this);
    tv.setText("Test!");
    setContentView(tv);
    while (true) {
      tv.setText(String.valueOf(reader.direction));
	  try {
        Thread.sleep(1000, 0);
	  } catch (Exception e) {
        // Ignore.
      }
    }
  }
}