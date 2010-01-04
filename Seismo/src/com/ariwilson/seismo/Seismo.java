package com.ariwilson.seismo;

import android.app.Activity;
import android.content.Intent;
import android.content.pm.ActivityInfo;
import android.os.Bundle;
import android.view.Menu;
import android.view.MenuInflater;
import android.view.MenuItem;
import android.view.WindowManager;
import android.widget.FrameLayout;
import android.widget.ListView;
import android.widget.TextView;
import android.widget.Toast;

public class Seismo extends Activity {
  @Override
  public void onCreate(Bundle savedInstanceState) {
    super.onCreate(savedInstanceState);
    setRequestedOrientation(ActivityInfo.SCREEN_ORIENTATION_PORTRAIT);
    getWindow().setFlags(WindowManager.LayoutParams.FLAG_FULLSCREEN,
                         WindowManager.LayoutParams.FLAG_FULLSCREEN);
    db_ = new SeismoDbAdapter(this);
    db_.open();
    setSeismoView();
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
      String name = seismo_view_.save();
      Toast.makeText(this, "Saved graph as " + name + ".", Toast.LENGTH_LONG)
          .show();
      return true;
    case R.id.Export:
      // TODO(ariw): Use list view to display all saved files and intents to
      // send via e-mail.
      return true;
    case R.id.Help:
      startActivity(new Intent(this, Help.class));
      return true;
    }

    return false;
  }

  private void setSeismoView() {
    FrameLayout layout = new FrameLayout(this);
    seismo_view_ = new SeismoView(this, db_, 25);
    layout.addView(seismo_view_);
    setContentView(layout);
  }

  /*private void setExportView() {
    FrameLayout layout = new FrameLayout(this);
    export_view_ = new ListView(this);
    Cursor c = db_.fetchAllGraphs();
    startManagingCursor(c);
    String[] from = new String[] { SeismoDbAdapter.KEY_TITLE };
    int[] to = new int[] { R.id.text1 };

    // Now create an array adapter and set it to display using our row
    SimpleCursorAdapter notes =
        new SimpleCursorAdapter(this, R.layout.notes_row, c, from, to);
    setListAdapter(notes);
    layout.addView(export_view_);
    setContentView(layout);
  }*/

  private SeismoDbAdapter db_;
  private SeismoView seismo_view_;
}
