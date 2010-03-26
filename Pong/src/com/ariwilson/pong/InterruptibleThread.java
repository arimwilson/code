package com.ariwilson.pong;

public abstract class InterruptibleThread extends Thread {
  @Override
  public void run() {
    while (!isInterrupted()) {
      work();
    }
  }

  public abstract void work();
}
