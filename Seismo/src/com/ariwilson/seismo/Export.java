package com.ariwilson.seismo;

import android.app.Activity;
import android.content.Context;
import android.content.pm.ActivityInfo;
import android.os.Bundle;
import android.view.WindowManager;
import android.widget.FrameLayout;
import android.widget.ListView;

public class Export extends Activity {
  @Override
  public void onCreate(Bundle savedInstanceState) {
    super.onCreate(savedInstanceState);
    setRequestedOrientation(ActivityInfo.SCREEN_ORIENTATION_PORTRAIT);
    getWindow().setFlags(WindowManager.LayoutParams.FLAG_FULLSCREEN,
                         WindowManager.LayoutParams.FLAG_FULLSCREEN);
    FrameLayout layout = new FrameLayout(this);
    // export_view_ = new ExportView(this, db);
    layout.addView(export_view_);
    setContentView(layout);
  }

  private class ExportView extends ListView {
    public ExportView(Context ctx, SeismoDbAdapter db) {
      super(ctx);
      db_ = db;
    }

    private SeismoDbAdapter db_;
  }

  private ExportView export_view_;
}
