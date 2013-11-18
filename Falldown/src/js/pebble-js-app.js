Pebble.addEventListener("appmessage",
    function(e) {
      var req = new XMLHttpRequest();
      req.open('POST', e.payload.url, true);
      req.onload = function(e) {
      }
      req.send(null);
    }
);
