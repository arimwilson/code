Pebble.addEventListener("appmessage",
    function(e) {
      // Check to see if we're proxying a nonce or a score.
      var req = new XMLHttpRequest();
      req.open('POST', e.payload.url, true);
      req.onload = function(e) {
      }
      req.send(null);
    }
);
