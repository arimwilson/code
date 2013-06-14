package com.ariwilson.mytracksforpebble;

import android.app.Service;
import android.content.ComponentName;
import android.content.Context;
import android.content.Intent;
import android.content.ServiceConnection;
import android.graphics.Bitmap;
import android.graphics.BitmapFactory;
import android.location.Location;
import android.os.Handler;
import android.os.IBinder;
import android.os.RemoteException;
import android.util.Log;
import android.view.View;
import com.getpebble.android.kit.Constants;
import com.getpebble.android.kit.PebbleKit;
import com.getpebble.android.kit.util.PebbleDictionary;
import com.google.android.apps.mytracks.content.MyTracksProviderUtils;
import com.google.android.apps.mytracks.services.ITrackRecordingService;
import com.google.android.apps.mytracks.stats.TripStatistics;

import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import java.util.concurrent.TimeUnit;

public class PebbleSportsService extends Service {
  private static final String TAG = "PebbleSportsService";

  public static final long NOTIFY_INTERVAL = 2; // 2 seconds

  // Pebble
  private PebbleKit.PebbleDataReceiver sportsDataHandler = null;
  private int sportsState = Constants.SPORTS_STATE_INIT;

  // MyTracks
  private MyTracksProviderUtils myTracksProviderUtils;
  private ITrackRecordingService myTracksService;
  private Intent intent;

  // Timer for Pebble updates
  private ScheduledExecutorService scheduleTaskExecutor;

  // Connection to the MyTracks service
  private ServiceConnection serviceConnection = new ServiceConnection() {
    @Override
    public void onServiceConnected(ComponentName className, IBinder service) {
      myTracksService = ITrackRecordingService.Stub.asInterface(service);
    }

    @Override
    public void onServiceDisconnected(ComponentName className) {
      myTracksService = null;

      stopWatchApp();

      if ( scheduleTaskExecutor != null ) {
        scheduleTaskExecutor.shutdown();
      }

      // Always deregister any Activity-scoped BroadcastReceivers when the Activity is paused
      if (sportsDataHandler != null) {
        unregisterReceiver(sportsDataHandler);
        sportsDataHandler = null;
      }

      // unbind and stop the MyTracks service
      if (myTracksService != null) {
        unbindService(serviceConnection);
      }
    }
  };

  @Override
  public void onCreate() {
    super.onCreate();

    final Handler handler = new Handler();

    intent = new Intent();
    ComponentName componentName = new ComponentName(getString(R.string.mytracks_service_package), getString(R.string.mytracks_service_class));
    intent.setComponent(componentName);

    startService(intent);
    bindService(intent, serviceConnection, 0);

    myTracksProviderUtils = MyTracksProviderUtils.Factory.get(getApplicationContext());

    // To receive data back from the sports watch-app, Android
    // applications must register a "DataReceiver" to operate on the
    // dictionaries received from the watch.
    //
    // In this example, we're registering a receiver to listen for
    // changes in the activity state sent from the watch, allowing
    // us the pause/resume the activity when the user presses a
    // button in the watch-app.
    sportsDataHandler = new PebbleKit.PebbleDataReceiver(Constants.SPORTS_UUID) {
      @Override
      public void receiveData(final Context context, final int transactionId, final PebbleDictionary data) {
        int newState = data.getUnsignedInteger(Constants.SPORTS_STATE_KEY).intValue();
        sportsState = newState;

        PebbleKit.sendAckToPebble(context, transactionId);

        handler.post(new Runnable() {
          @Override
          public void run() {
            if (sportsState == Constants.SPORTS_STATE_RUNNING) {
              Log.i(TAG,"Running");
            } else {
              Log.i(TAG,"Paused");
            }
          }
        });
      }
    };
    PebbleKit.registerReceivedDataHandler(this, sportsDataHandler);

    startWatchApp();

    scheduleTaskExecutor = Executors.newScheduledThreadPool(5);
    // This schedule a task to run every 10 minutes:
    scheduleTaskExecutor.scheduleAtFixedRate(new Runnable() {
      public void run() {
        if (myTracksService != null) {
          try {
            if (myTracksService.isRecording() && !myTracksService.isPaused()) {
              if (!myTracksService.isPaused()) {
                Location loc = myTracksProviderUtils.getLastValidTrackPoint();
                TripStatistics statistics = myTracksProviderUtils.getTrack(myTracksService.getRecordingTrackId()).getTripStatistics();
                float speed = 0;
                if (loc != null) {
                  speed = loc.getSpeed();
                }
                updateWatchApp(statistics.getTotalDistance(),statistics.getTotalTime(),speed);
              }
            }
          } catch (RemoteException e) {
          }
        }
      }
    }, 0, 2, TimeUnit.SECONDS);
  }

  @Override
  public IBinder onBind(android.content.Intent intent) {
    return null;
  }

  // Send a broadcast to launch the specified application on the connected Pebble
  public void startWatchApp() {
    PebbleKit.startAppOnPebble(getApplicationContext(), Constants.SPORTS_UUID);
  }

  // Send a broadcast to close the specified application on the connected Pebble
  public void stopWatchApp() {
    PebbleKit.closeAppOnPebble(getApplicationContext(), Constants.SPORTS_UUID);
  }

  // A custom icon and name can be applied to the sports-app to
  // provide some support for "branding" your Pebble-enabled sports
  // application on the watch.
  //
  // It is recommended that applications customize the sports
  // application before launching it. Only one application may
  // customize the sports application at a time on a first-come,
  // first-serve basis.
  public void customizeWatchApp(View view) {
    final String customAppName = "My Tracks";
    final Bitmap customIcon = BitmapFactory.decodeResource(getResources(), R.drawable.watch);

    PebbleKit.customizeWatchApp(
        getApplicationContext(), Constants.PebbleAppType.SPORTS, customAppName, customIcon);
  }

  // Push (distance, time, pace) data to be displayed on Pebble's Sports app.
  //
  // To simplify formatting, values are transmitted to the watch as strings.
  public void updateWatchApp(double distance, long time, float speed) {
    PebbleDictionary data = new PebbleDictionary();
    data.addString(Constants.SPORTS_TIME_KEY, String.format("%04d", time));
    data.addString(Constants.SPORTS_DISTANCE_KEY, String.format("%02.1f", distance));
    data.addString(Constants.SPORTS_DATA_KEY, String.format("%.1f",speed));
    data.addUint8(Constants.SPORTS_UNITS_KEY, (byte)Constants.SPORTS_UNITS_METRIC);

    PebbleKit.sendDataToPebble(getApplicationContext(), Constants.SPORTS_UUID, data);
  }
}
