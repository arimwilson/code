// Example log line:
// console.log("log");

Pebble.addEventListener("appmessage",
  function(e) {
    var req = new XMLHttpRequest();
    req.open('POST', e.payload.url, true);
    delete e.payload.url;
    var body = JSON.stringify(e.payload);
    req.onreadystatechange = function() {
      if (req.readyState == 4 && req.status == 200) {
        Pebble.sendAppMessage(JSON.parse(req.responseText));
      }
    }
    req.send(body);
  }
);

Pebble.addEventListener("showConfiguration",
  function(e) {
    Pebble.openURL("http://pebblescores.appspot.com/configuration.html");
  }
);

Pebble.addEventListener("webviewclosed",
  function(e) {
    var configuration = JSON.parse(decodeURIComponent(e.response));
  }
);
