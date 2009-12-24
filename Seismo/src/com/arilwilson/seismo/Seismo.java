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
  }
  
  @Override
  public void onPause() {
    super.onPause();
    view_.pause();
  }

  @Override
  public void onResume() {
    super.onPause();
    view_.resume();
  }

  private SeismoView view_;
}