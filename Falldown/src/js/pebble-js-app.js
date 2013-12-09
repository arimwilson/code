Pebble.addEventListener("appmessage",
    function(e) {
      // Check to see if we're proxying a nonce or a score.
      var method = e.payload.url.substr(e.payload.url.lastIndexOf("/") + 1);
      if (method == "nonce") {
      } else if (method == "submit") {
      }
      var req = new XMLHttpRequest();
      req.open('POST', e.payload.url, true);
      req.onload = function(e) {
      }
      req.send(null);
    }
);
