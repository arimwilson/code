package com.ariwilson.pong;

import android.app.Activity;
import android.content.pm.ActivityInfo;
import android.os.Bundle;
import android.view.Window;
import android.view.WindowManager;
import android.widget.FrameLayout;

public class Pong extends Activity {
  @Override
  public void onCreate(Bundle savedInstanceState) {
    super.onCreate(savedInstanceState);
    setRequestedOrientation(ActivityInfo.SCREEN_ORIENTATION_LANDSCAPE);
    getWindow().setFlags(WindowManager.LayoutParams.FLAG_FULLSCREEN,
                         WindowManager.LayoutParams.FLAG_FULLSCREEN);
    requestWindowFeature(Window.FEATURE_NO_TITLE);
    FrameLayout layout = new FrameLayout(this);
    view_ = new PongView(this);
    layout.addView(view_);
    setContentView(layout);
  }

  PongView view_;
}
