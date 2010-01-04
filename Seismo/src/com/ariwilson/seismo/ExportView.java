package com.ariwilson.seismo;

import android.content.Context;
import android.widget.ListView;

public class ExportView extends ListView {
  public ExportView(Context ctx, SeismoDbAdapter db) {
    super(ctx);
    db_ = db;
  }

  private SeismoDbAdapter db_;
}
