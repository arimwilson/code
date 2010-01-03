package com.ariwilson.seismo;

import android.app.Activity;
import android.content.pm.ActivityInfo;
import android.os.Bundle;
import android.view.Menu;
import android.view.MenuInflater;
import android.view.MenuItem;
import android.view.WindowManager;
import android.widget.LinearLayout;

public class Seismo extends Activity {
  @Override
  public void onCreate(Bundle savedInstanceState) {
    super.onCreate(savedInstanceState);
    setRequestedOrientation(ActivityInfo.SCREEN_ORIENTATION_PORTRAIT);
    getWindow().setFlags(WindowManager.LayoutParams.FLAG_FULLSCREEN,
                         WindowManager.LayoutParams.FLAG_FULLSCREEN);
    LinearLayout layout = new LinearLayout(this);
    view_ = new SeismoView(this, 25);
    layout.addView(view_);

    setContentView(layout);
  }

  @Override
  public boolean onCreateOptionsMenu(Menu menu) {
    super.onCreateOptionsMenu(menu);
 
    MenuInflater inflater = getMenuInflater();
    inflater.inflate(R.menu.options, menu);

    return true;
  }

  @Override
  public boolean onOptionsItemSelected(MenuItem item) {
    super.onOptionsItemSelected(item);

    switch (item.getItemId()) {
    case R.id.Filter:
      if (item.getTitleCondensed().equals("Filter")) {
        view_.filter();
        item.setTitle("Unfilter noise");
        item.setTitleCondensed("Unfilter");
      } else {
        view_.unfilter();
        item.setTitle("Filter noise");
        item.setTitleCondensed("Filter");
      }
      return true;
    case R.id.Pause:
      if (item.getTitleCondensed().equals("Pause")) {
        view_.pause();
        item.setTitle("Resume measurement");
        item.setTitleCondensed("Resume");
      } else {
        view_.resume();
        item.setTitle("Pause measurement");
        item.setTitleCondensed("Pause");
      }
      return true;
    case R.id.x:
      view_.x();
      return true;
    case R.id.y:
      view_.y();
      return true;
    case R.id.z:
      view_.z();
      return true;
    case R.id.Save:
      view_.save();
      return true;
    case R.id.View:
      // TODO(ariw): Display view of all graphs.
      return true;
    }

    return false;
  }

  private SeismoView view_;
}
