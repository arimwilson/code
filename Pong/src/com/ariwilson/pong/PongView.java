package com.ariwilson.pong;

import android.content.Context;
import android.view.View;

public class PongView extends View {
  public PongView(Context ctx) {
    super(ctx);
    ctx_ = ctx;
  }

  private Context ctx_;
}
