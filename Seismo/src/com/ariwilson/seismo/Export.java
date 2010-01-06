package com.ariwilson.seismo;

import java.util.ArrayList;

import android.app.Activity;
import android.content.Context;
import android.content.Intent;
import android.content.pm.ActivityInfo;
import android.graphics.drawable.ColorDrawable;
import android.os.Bundle;
import android.util.Log;
import android.view.View;
import android.view.WindowManager;
import android.widget.AdapterView;
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
    db_ = SeismoDbAdapter.getAdapter();
    export_view_ = new ExportView(this);
    layout.addView(export_view_);
    setContentView(layout);
  }

  private String graphToCsv(ArrayList<ArrayList<Float>> graph) {
    StringBuffer csv = new StringBuffer("t,x,y,z\n");
    int t = 0;
    for (int i = 0; i < graph.size(); ++i) {
      assert(graph.get(i).size() == 3);
      csv.append(Integer.toString(t));
      csv.append(",");
      csv.append(graph.get(i).get(0).toString());
      csv.append(",");
      csv.append(graph.get(i).get(1).toString());
      csv.append(",");
      csv.append(graph.get(i).get(2).toString());
      csv.append("\n");
      t += 25;
    }
    return csv.toString();
  }

  private class ExportView extends ListView implements
      AdapterView.OnItemClickListener {
    public ExportView(Context ctx) {
      super(ctx);
      ctx_ = ctx;
      setBackgroundColor(0xFFFFFFFF);
      setDivider(new ColorDrawable(0xFF898989));
      setDividerHeight(1);
      setOnItemClickListener(this);
      db_.open(ctx_);
      graph_names_ = db_.fetchGraphNames();
      setAdapter(new ArrayAdapter<String>(ctx, R.layout.export, graph_names_)); 
      db_.close();
    }

    @Override
    public void onItemClick(AdapterView<?> parent_view, View child_view,
                            int position, long id) {
      Log.i("Seismo", "Is anyone out there?");
      db_.open(ctx_);
      ArrayList<ArrayList<Float>> graph = db_.fetchGraph(graph_names_.get(
          position));
      db_.close();
      Intent send_intent = new Intent(Intent.ACTION_SEND);
      send_intent.setType("plain/text");
      send_intent.putExtra(Intent.EXTRA_SUBJECT,
                           "Seismo data from " + graph_names_.get(position));
      send_intent.putExtra(Intent.EXTRA_TEXT, graphToCsv(graph)); 
      startActivity(Intent.createChooser(send_intent, "Email")); 
    }    

    private Context ctx_;
  }

  private SeismoDbAdapter db_;
  private ArrayList<String> graph_names_;
  private ExportView export_view_;
}
