package com.ariwilson.seismo;

import android.app.Activity;
import android.content.Context;
import android.os.Bundle;
import android.view.WindowManager;
import android.widget.FrameLayout;

public class Graph extends Activity {
  @Override
  public void onCreate(Bundle savedInstanceState) {
    super.onCreate(savedInstanceState);
    getWindow().setFlags(WindowManager.LayoutParams.FLAG_FULLSCREEN,
                         WindowManager.LayoutParams.FLAG_FULLSCREEN);
    FrameLayout layout = new FrameLayout(this);
    graph_view_ = new GraphView(this);
    layout.addView(graph_view_);
    setContentView(layout);
  }

  private class GraphView extends GraphListView {
    GraphView(Context ctx) {
      super(ctx);
      ctx_ = ctx;
    }

    private Context ctx_;
  }

  private GraphView graph_view_;
}
