package com.ariwilson.mytracksforpebble;

import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;

public class MyTracksReceiver extends BroadcastReceiver {

  // start my tracks pebble service
  @Override
  public void onReceive(Context context, Intent intent) {
    Intent service = new Intent(context,PebbleSportsService.class);
    context.startService(service);        
  }
}

