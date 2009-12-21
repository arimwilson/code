package com.arilwilson.seismo;

import android.content.Context;
import android.graphics.Canvas;
import android.graphics.Path;
import android.graphics.drawable.ShapeDrawable;
import android.graphics.drawable.shapes.PathShape;
import android.view.View;

public class SeismoView extends View {
    private ShapeDrawable mDrawable;

    public SeismoView(Context ctx) {
        super(ctx);

        int x = 10;
        int y = 10;
        int width = 300;
        int height = 50;

        Path mPath = new Path();
        mDrawable = new ShapeDrawable(new PathShape(mPath));
        mDrawable.getPaint().setColor(0xff74AC23);
        mDrawable.setBounds(x, y, x + width, y + height);
    }

    protected void onDraw(Canvas canvas) {
        mDrawable.draw(canvas);
    }
}
