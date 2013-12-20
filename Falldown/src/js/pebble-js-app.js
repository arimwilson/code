// Example log line:
// console.log("log");

// This function will merge two objects, preferring object1's properties to
// object2's. It assumes there are no recursive objects present in either
// object.
function merge_objects(object1, object2) {
  if (object1 == null && object2 == null) return null;
  var object3 = {};
  for (var attrname in object2) { object3[attrname] = object2[attrname]; }
  for (var attrname in object1) { object3[attrname] = object1[attrname]; }
  return object3;
}

// Proxy HTTP requests/responses for the Pebble app. All app configuration
// options are sent with each HTTP request.
Pebble.addEventListener("appmessage",
  function(e) {
    var req = new XMLHttpRequest();
    req.open('POST', e.payload.url, true);
    delete e.payload.url;
    var body = JSON.stringify(merge_objects(
        e.payload, window.localStorage.getItem("configuration"));
    req.onreadystatechange = function() {
      if (req.readyState == 4 && req.status == 200) {
        Pebble.sendAppMessage(JSON.parse(req.responseText));
      }
    }
    req.send(body);
  }
);

// Open app configuration for PebbleScores.
Pebble.addEventListener("showConfiguration",
  function(e) {
    Pebble.openURL("http://pebblescores.appspot.com/configuration.html");
  }
);

// Store app configuration on phone.
Pebble.addEventListener("webviewclosed",
  function(e) {
    if (e.response != "") {
      var configuration = JSON.parse(decodeURIComponent(e.response));
      window.localStorage.setItem("configuration", JSON.stringify(configuration));
    }
  }
);
