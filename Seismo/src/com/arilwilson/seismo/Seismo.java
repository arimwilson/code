package com.arilwilson.seismo;

import android.app.Activity;
import android.os.Bundle;
import android.util.Log;
import android.view.Menu;
import android.view.MenuInflater;
import android.view.MenuItem;
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

  @Override
  public boolean onCreateOptionsMenu(Menu menu) {
    super.onCreateOptionsMenu(menu);
 
    MenuInflater inflater = getMenuInflater();
    inflater.inflate(R.menu.options, menu);
    
    return true;
  }

  @Override
  public boolean onOptionsItemSelected(MenuItem item) {
    if (item.getTitle() == "Filter") {
      if (item.isChecked()) {
        view_.filter();
      } else {
        view_.unfilter();
      }
      return true;
    } else if (item.getTitle() == "Pause") {
      if (item.isChecked()) {
        view_.resume();
        item.setChecked(false);
      } else {
        view_.pause();
        item.setChecked(true);
      }
      return true;
    }

    return false;
  }

  private SeismoView view_;
}
