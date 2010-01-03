package com.ariwilson.seismo;

import android.app.Activity;
import android.content.pm.ActivityInfo;
import android.os.Bundle;
import android.view.Menu;
import android.view.MenuInflater;
import android.view.MenuItem;
import android.view.WindowManager;
import android.widget.LinearLayout;
import android.widget.ListView;

public class Seismo extends Activity {
  @Override
  public void onCreate(Bundle savedInstanceState) {
    super.onCreate(savedInstanceState);
    setRequestedOrientation(ActivityInfo.SCREEN_ORIENTATION_PORTRAIT);
    getWindow().setFlags(WindowManager.LayoutParams.FLAG_FULLSCREEN,
                         WindowManager.LayoutParams.FLAG_FULLSCREEN);
    LinearLayout layout = new LinearLayout(this);
    seismo_view_ = new SeismoView(this, 25);
    layout.addView(seismo_view_);

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
        seismo_view_.filter();
        item.setTitle("Unfilter noise");
        item.setTitleCondensed("Unfilter");
      } else {
        seismo_view_.unfilter();
        item.setTitle("Filter noise");
        item.setTitleCondensed("Filter");
      }
      return true;
    case R.id.Pause:
      if (item.getTitleCondensed().equals("Pause")) {
        seismo_view_.pause();
        item.setTitle("Resume measurement");
        item.setTitleCondensed("Resume");
      } else {
        seismo_view_.resume();
        item.setTitle("Pause measurement");
        item.setTitleCondensed("Pause");
      }
      return true;
    case R.id.x:
      seismo_view_.x();
      return true;
    case R.id.y:
      seismo_view_.y();
      return true;
    case R.id.z:
      seismo_view_.z();
      return true;
    case R.id.Save:
      seismo_view_.save();
      return true;
    case R.id.Load:
      // TODO(ariw): Display view of all graphs.
      return true;
    }

    return false;
  }

  private SeismoView seismo_view_;
  private ListView graph_view_;
}
