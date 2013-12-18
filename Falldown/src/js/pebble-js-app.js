// Example log line:
// console.log("log");

Pebble.addEventListener("appmessage",
    function(e) {
      var body = JSON.stringify(e.payload);
      var req = new XMLHttpRequest();
      req.open('POST', e.payload.url, true);
      req.onreadystatechange = function() {
        if (req.readyState == 4 && req.status == 200) {
          Pebble.sendAppMessage(JSON.parse(req.responseText));
        }
      }
      req.send(body);
    }
);
