package com.arilwilson.seismo;

import android.content.Context;
import android.hardware.Sensor; 
import android.hardware.SensorEvent;
import android.hardware.SensorEventListener;
import android.hardware.SensorManager;

public class AccelerometerReader { 
  public volatile float direction = (float)0;
  public volatile float inclination;
  public volatile float rollingZ = (float)0;

  public volatile float kFilteringFactor = (float)0.05;
  public float aboveOrBelow = (float)0;
	
  public AccelerometerReader(Context ctx) 
      throws UnsupportedOperationException { 
    SensorManager sensor_manager = (SensorManager) ctx.getSystemService(
        Context.SENSOR_SERVICE);
    sensor_manager.registerListener(
        listener_, sensor_manager.getDefaultSensor(Sensor.TYPE_ACCELEROMETER),
        SensorManager.SENSOR_DELAY_FASTEST); 
  }

  private SensorEventListener listener_ = new SensorEventListener(){
    public void onAccuracyChanged(Sensor arg0, int arg1) {}

    public void onSensorChanged(SensorEvent evt) {
      float vals[] = evt.values;
      
      if(evt.sensor.getType() == Sensor.TYPE_ORIENTATION) {
        float rawDirection = vals[0];

        direction =(float) ((rawDirection * kFilteringFactor) + 
            (direction * (1.0 - kFilteringFactor)));

        inclination = 
            (float) ((vals[2] * kFilteringFactor) + 
            (inclination * (1.0 - kFilteringFactor)));

                
        if(aboveOrBelow > 0) inclination = inclination * -1;
          
        if(evt.sensor.getType() == Sensor.TYPE_ACCELEROMETER) {
            aboveOrBelow =
                (float) ((vals[2] * kFilteringFactor) + 
                (aboveOrBelow * (1.0 - kFilteringFactor)));
        }
      }
    }
  };
}
