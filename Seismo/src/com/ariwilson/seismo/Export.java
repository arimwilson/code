package com.ariwilson.seismo;

import java.io.File;
import java.io.FileOutputStream;
import java.util.ArrayList;

import android.app.Activity;
import android.content.Context;
import android.content.Intent;
import android.graphics.drawable.ColorDrawable;
import android.net.Uri;
import android.os.Bundle;
import android.util.Log;
import android.view.ContextMenu;
import android.view.MenuItem;
import android.view.View;
import android.view.WindowManager;
import android.view.View.OnCreateContextMenuListener;
import android.widget.AdapterView;
import android.widget.ArrayAdapter;
import android.widget.FrameLayout;
import android.widget.ListView;
import android.widget.Toast;
import android.widget.AdapterView.AdapterContextMenuInfo;

public class Export extends Activity {
  @Override
  public void onCreate(Bundle savedInstanceState) {
    super.onCreate(savedInstanceState);
    getWindow().setFlags(WindowManager.LayoutParams.FLAG_FULLSCREEN,
                         WindowManager.LayoutParams.FLAG_FULLSCREEN);
    FrameLayout layout = new FrameLayout(this);
    db_ = SeismoDbAdapter.getAdapter();
    export_view_ = new ExportView(this);
    layout.addView(export_view_);
    setContentView(layout);
  }

  private String graphToCsv(ArrayList<ArrayList<Float>> graph) {
    StringBuffer csv = new StringBuffer();
    csv.append("time (seconds),");
    csv.append("x acceleration (m/s^2),");
    csv.append("y acceleration (m/s^2),");
    csv.append("z acceleration (m/s^2)\n");
    int t = 0;
    for (int i = 0; i < graph.size(); ++i) {
      assert(graph.get(i).size() == 3);
      csv.append(Float.toString(graph.get(i).get(0) / 1000));
      csv.append(",");
      csv.append(graph.get(i).get(1).toString());
      csv.append(",");
      csv.append(graph.get(i).get(2).toString());
      csv.append(",");
      csv.append(graph.get(i).get(3).toString());
      csv.append("\n");
      t += 25;
    }
    return csv.toString();
  }

  @Override 
  public boolean onContextItemSelected(MenuItem item) { 
    AdapterContextMenuInfo menu_info =
        (AdapterContextMenuInfo) item.getMenuInfo(); 

    switch (item.getItemId()) { 
      case 0:
        db_.open(this);
        if (db_.deleteGraph(graph_names_.get(menu_info.position))) {
          graph_names_.remove(menu_info.position);
          adapter_.notifyDataSetChanged();
        } else {
          Toast.makeText(this, "Failed to delete graph in position " +
                               Long.toString(menu_info.position) + ".",
                         Toast.LENGTH_LONG)
              .show();
        }
        db_.close();
        return true;
    } 
    return false; 
  }  

  private class ExportView extends ListView implements
      AdapterView.OnItemClickListener, OnCreateContextMenuListener {
    public ExportView(Context ctx) {
      super(ctx);
      ctx_ = ctx;
      setCacheColorHint(0xFFFFFFFF);
      setBackgroundColor(0xFFFFFFFF);
      setDivider(new ColorDrawable(0xFF898989));
      setDividerHeight(1);
      setOnItemClickListener(this);
      setOnCreateContextMenuListener(this);
      db_.open(ctx_);
      graph_names_ = db_.fetchGraphNames();
      adapter_ = new ArrayAdapter<String>(ctx, R.layout.export, graph_names_);
      setAdapter(adapter_);
      db_.close();
    }

    @Override
    public void onItemClick(AdapterView<?> parent_view, View child_view,
                            int position, long id) {
      db_.open(ctx_);
      ArrayList<ArrayList<Float>> graph = db_.fetchGraph(graph_names_.get(
          position));
      db_.close();
      try {
        File temp_file = File.createTempFile("Seismo", ".csv");
        FileOutputStream out = new FileOutputStream(temp_file);
        out.write(graphToCsv(graph).getBytes());
        out.close();
        Intent send_intent = new Intent(Intent.ACTION_SEND);
        send_intent.setType("text/csv");
        send_intent.putExtra(Intent.EXTRA_SUBJECT,
                             "Seismo data from " + graph_names_.get(position));
        send_intent.putExtra(Intent.EXTRA_STREAM, Uri.fromFile(
            temp_file)); 
        startActivity(Intent.createChooser(send_intent, "E-mail"));
        temp_file.deleteOnExit();
      } catch (Exception e) {
        Log.e("Seismo", e.toString());
      }
    }

    @Override 
    public void onCreateContextMenu(ContextMenu menu, View view,
                                    ContextMenu.ContextMenuInfo menu_info) { 
       menu.setHeaderTitle("Options");
       menu.add(0, 0, 0, "Delete");
    }

    private Context ctx_;
  }

  private SeismoDbAdapter db_;
  private ArrayList<String> graph_names_;
  private ArrayAdapter<String> adapter_;
  private ExportView export_view_;
}
