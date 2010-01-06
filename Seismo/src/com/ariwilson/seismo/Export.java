package com.ariwilson.seismo;

import java.util.ArrayList;

import android.app.Activity;
import android.content.Context;
import android.content.pm.ActivityInfo;
import android.graphics.drawable.ColorDrawable;
import android.os.Bundle;
import android.view.WindowManager;
import android.widget.ArrayAdapter;
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
    // TODO(ariw): Insert db from other Seismo activity context.
    export_view_ = new ExportView(this, SeismoDbAdapter.getAdapter());
    export_view_.setBackgroundColor(0xFFFFFFFF);
    export_view_.setDivider(new ColorDrawable(0xFF898989));
    export_view_.setDividerHeight(1);
    layout.addView(export_view_);
    setContentView(layout);
  }

  private class ExportView extends ListView {
    public ExportView(Context ctx, SeismoDbAdapter db) {
      super(ctx);
      db_ = db;
      db_.open(ctx);
      ArrayList<String> graph_names = db_.fetchGraphNames();
      setAdapter(new ArrayAdapter<String>(ctx, R.layout.export, graph_names)); 
      db_.close();
    }
    private SeismoDbAdapter db_;
  }

  private ExportView export_view_;
}
