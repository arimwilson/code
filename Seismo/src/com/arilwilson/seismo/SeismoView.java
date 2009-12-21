package com.arilwilson.seismo;

import android.content.Context;
import android.graphics.Canvas;
import android.graphics.Path;
import android.graphics.drawable.BitmapDrawable;
import android.graphics.drawable.shapes.PathShape;
import android.view.View;

public class SeismoView extends View {
    public SeismoView(Context ctx) {
        super(ctx);

        drawable_ = new BitmapDrawable();
        drawable_.getPaint().setColor(0xFFFFFF00);
        drawable_.setBounds(x, y, x + width, y + height);
    }

    protected void onDraw(Canvas canvas) {
        drawable_.draw(canvas);
    }

    private BitmapDrawable drawable_;
}
