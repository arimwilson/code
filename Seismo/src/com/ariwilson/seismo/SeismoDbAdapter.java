package com.ariwilson.seismo;

import android.content.ContentValues;
import android.content.Context;
import android.database.Cursor;
import android.database.SQLException;
import android.database.sqlite.SQLiteDatabase;
import android.database.sqlite.SQLiteOpenHelper;
import android.util.Log;

public class SeismoDbAdapter {

  public static final String KEY_TITLE = "title";
  public static final String KEY_BODY = "body";
  public static final String KEY_ROWID = "_id";

  public SeismoDbAdapter(Context ctx) {
      ctx_ = ctx;
  }

  public SeismoDbAdapter open() throws SQLException {
      db_helper_ = new DatabaseHelper(ctx_);
      db_ = db_helper_.getWritableDatabase();
      return this;
  }
  
  public void close() {
      db_helper_.close();
  }

  public long createGraph(String title, byte[] body) {
      ContentValues initial_values = new ContentValues();
      initial_values.put(KEY_TITLE, title);
      initial_values.put(KEY_BODY, body);

      return db_.insert(DATABASE_TABLE, null, initial_values);
  }

  public boolean deleteGraph(long row_id) {
      return db_.delete(DATABASE_TABLE, KEY_ROWID + "=" + row_id, null) > 0;
  }

  public Cursor fetchAllGraphs() {
      return db_.query(DATABASE_TABLE, new String[] {KEY_ROWID, KEY_TITLE,
              KEY_BODY}, null, null, null, null, null);
  }

  public Cursor fetchGraph(long row_id) throws SQLException {
      Cursor cursor =
          db_.query(true, DATABASE_TABLE, new String[] {KEY_ROWID, KEY_TITLE,
                    KEY_BODY}, KEY_ROWID + "=" + row_id, null, null, null,
                    null, null);
      if (cursor != null) {
          cursor.moveToFirst();
      }
      return cursor;

  }

  public boolean updateGraph(long row_id, String title, byte[] body) {
      ContentValues args = new ContentValues();
      args.put(KEY_TITLE, title);
      args.put(KEY_BODY, body);

      return db_.update(DATABASE_TABLE, args, KEY_ROWID + "=" + row_id, null) >
             0;
  }

  private static class DatabaseHelper extends SQLiteOpenHelper {

    DatabaseHelper(Context context) {
        super(context, DATABASE_NAME, null, DATABASE_VERSION);
    }

    @Override
    public void onCreate(SQLiteDatabase db) {
        db.execSQL(DATABASE_CREATE);
    }

    @Override
    public void onUpgrade(SQLiteDatabase db, int old_version, int new_version) {
        Log.w(TAG, "Upgrading database from version " + old_version + " to " +
                   new_version + ", which will destroy all old data");
        db.execSQL("DROP TABLE IF EXISTS seismo");
        onCreate(db);
    }
  }

  private static final String TAG = "SeismoDbAdapter";
  private DatabaseHelper db_helper_;
  private SQLiteDatabase db_;
  
  private static final String DATABASE_CREATE =
      "create table seismo (_id integer primary key autoincrement, " +
          "title text not null, body blob not null);";

  private static final String DATABASE_NAME = "data";
  private static final String DATABASE_TABLE = "seismo";
  private static final int DATABASE_VERSION = 2;

  private final Context ctx_;
}
