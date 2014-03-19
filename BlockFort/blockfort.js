// Generic three.js objects are in the global namespace.
var t, renderer, scene, sceneOrtho, width, height, camera, cameraOrtho,
    controls, time;

onWindowResize = function() {
  width = window.innerWidth;
  height = window.innerHeight;

  camera.aspect = width / height;
  camera.updateProjectionMatrix();

  cameraOrtho.left = -width / 2;
  cameraOrtho.right = width / 2;
  cameraOrtho.top = height / 2;
  cameraOrtho.bottom = -height / 2;
  cameraOrtho.updateProjectionMatrix();

  blockfort.crosshair.position.set(0, 0, 1);

  renderer.setSize(width, height);
}

cameraDirection = function() {
  // Transform point in front of camera in camera space into global space to
  // find the direction of the camera.
  return new t.Vector3(0, 0, -1).applyMatrix4(camera.matrixWorld).sub(
      controls.getObject().position).normalize();
}

// Global object for scene-specific stuff.
var blockfort = {
}

blockfort.update = function() {
  requestAnimationFrame(blockfort.update);

  // Render the scene.
  renderer.clear();
  renderer.render(scene, camera);
  renderer.clearDepth();
  renderer.render(sceneOrtho, cameraOrtho);

  // Update controls.
  controls.update(Date.now() - time);
  time = Date.now();
}

blockfort.pointerLockChange = function(event) {
  if (document.pointerLockElement === blockfort.element ||
      document.webkitPointerLockElement === blockfort.element ||
      document.mozPointerLockElement === blockfort.element) {
    controls.enabled = true;
    $(document).click(blockfort.buildClick);

    blockfort.blocker.hide();
  } else {
    controls.enabled = false;
    $(document).off("click");
    $(document).off("keypress");

    blockfort.blocker.show();
  }
}

blockfort.pointerLockClick = function(event) {
  blockfort.element.requestPointerLock =
      blockfort.element.requestPointerLock ||
      blockfort.element.webkitRequestPointerLock ||
      blockfort.element.mozRequestPointerLock;
  blockfort.element.requestPointerLock();
}

// Given world coordinates, return grid coordinates.
blockfort.gridCoordinates = function(v) {
  var u = new t.Vector3();
  u.x = Math.floor(v.x / blockfort.unitSize);
  u.y = Math.floor(v.y / blockfort.unitSize);
  u.z = Math.floor(v.z / blockfort.unitSize);
  return u;
}

blockfort.createFloor = function() {
  var geometry = new t.PlaneGeometry(
      blockfort.unitSize * blockfort.units, blockfort.unitSize * blockfort.units,
      blockfort.unitSize, blockfort.unitSize);
  // Floors generally are on the xz plane rather than the yz plane. Rotate it
  // there :).
  geometry.applyMatrix(new t.Matrix4().makeRotationX(-Math.PI / 2));
  var floorColor = 0x395D33;
  return new t.Mesh(
      geometry, new t.MeshLambertMaterial(
          { color: floorColor, ambient: floorColor })
  );
}

// v should be in grid coordinates.
blockfort.createCube = function(v, color) {
  var cube = new t.Mesh(
      new t.CubeGeometry(blockfort.unitSize, blockfort.unitSize,
                         blockfort.unitSize, blockfort.unitSize,
                         blockfort.unitSize, blockfort.unitSize),
      new t.MeshLambertMaterial({ color: color, ambient: color })
  );
  cube.position.set(v.x * blockfort.unitSize, (v.y + 0.5) * blockfort.unitSize,
                    v.z * blockfort.unitSize);
  return cube;
}

// Build block / destroy block controls.
blockfort.buildClick = function(event) {
  var direction = cameraDirection();
  var ray = new t.Raycaster(controls.getObject().position, direction);
  var intersects = ray.intersectObjects(blockfort.objects);

  if (intersects.length > 0) {
    if (event.which === 1) { // left click
      var cube = blockfort.createCube(
          blockfort.gridCoordinates(intersects[0].point.sub(
              direction.multiplyScalar(blockfort.unitSize))),
          "#" + blockfort.block_color.val());
      scene.add(cube);
      blockfort.objects.push(cube);
    } else if (event.which === 3) { // right click
      var i = 0;
      for (; i < blockfort.objects.length; ++i) {
        if (blockfort.objects[i].id === intersects[0].object.id) {
          if (i != 0) blockfort.objects.remove(i);
          break;
        }
      }
      if (i != 0) scene.remove(intersects[0].object);
    }
  }
}

// Convert rendered world into a simplified format suitable for later
// retrieval.
blockfort.serialize = function() {
  var data = {};
  data.position = controls.getObject().position;
  data.yaw = controls.getObject().rotation.y;
  data.pitch = controls.getObject().children[0].rotation.x;
  data.objects = new Array();
  // Don't include floor in serialized objects.
  for (i = 1; i < blockfort.objects.length; ++i) {
    var object = {};
    object.position = blockfort.gridCoordinates(blockfort.objects[i].position);
    object.color = blockfort.objects[i].material.color.getHex();
    data.objects.push(object);
  }
  return JSON.stringify(data);
}

blockfort.save = function(event) {
  blockfort.name = prompt("World name to save?", blockfort.name);
  if (blockfort.name === null) return;
  $.post("save", { name: blockfort.name, data: blockfort.serialize() },
         function(data) { blockfort.id = data; }
  );
}

// Convert simplified format into rendered world.
blockfort.deserialize = function(world) {
  // TODO(ariw): This algorithm is slow as balls.
  if (world.Data.length === 0) return;
  data = JSON.parse(window.atob(world.Data));
  blockfort.id = world.Id

  // Remove existing objects from scene except floor.
  for (i = blockfort.objects.length - 1; i >= 1; --i) {
    scene.remove(blockfort.objects[i])
    blockfort.objects.remove(i);
  }
  // Load scene.
  var objects;
  // TODO(ariw): Remove this legacy mode.
  if (data instanceof Array) {
    objects = data;
  } else {
    objects = data.objects;
    controls.getObject().position.copy(data.position);
    controls.getObject().rotation.set(0, data.yaw, 0);
    controls.getObject().children[0].rotation.set(data.pitch, 0, 0);
  }
  for (i = 0; i < objects.length; ++i) {
    blockfort.objects.push(blockfort.createCube(
        objects[i].position, objects[i].color));
    scene.add(blockfort.objects[i + 1]);
  }
}

blockfort.load = function(event) {
  blockfort.name = prompt("World name to load?", blockfort.name);
  if (blockfort.name === null) return;
  $.ajax({
      url: "load", type: "POST", async: false,
      data: { name: blockfort.name }, success: blockfort.deserialize,
      dataType: "json"
  });
}

blockfort.share = function(event) {
  if (!("id" in blockfort)) return;
  alert(window.location.origin + "?id=" + blockfort.id);
}

blockfort.start = function() {
  t = THREE;
  renderer = new t.WebGLRenderer();
  width = document.body.clientWidth;
  height = document.body.clientHeight;
  renderer.setSize(width, height);
  renderer.autoClear = false;
  scene = new t.Scene();
  sceneOrtho = new t.Scene();
  time = Date.now();

  blockfort.blocker = $("#blocker");
  blockfort.menu = $("#menu");
  $("#save").click(blockfort.save);
  $("#load").click(blockfort.load);
  $("#share").click(blockfort.share);
  blockfort.block_color = $("#block_color");
  blockfort.unitSize = 20;
  blockfort.units = 1000;
  blockfort.name = "Default";
  blockfort.crosshair = new t.Sprite(new t.SpriteMaterial(
      {map: t.ImageUtils.loadTexture("crosshair.png")}));
  blockfort.crosshair.position.set(0, 0, 1);
  // TODO(ariw): Replace these constants with material.map.image.width,
  // material.map.image.height on texture load finish.
  blockfort.crosshair.scale.set(32, 32, 1.0);
  sceneOrtho.add(blockfort.crosshair);
  blockfort.objects = new Array();

  // Floor.
  var floor = blockfort.createFloor();
  scene.add(floor);
  blockfort.objects.push(floor);

  // White ambient light.
  var light = new t.AmbientLight(0xFFFFFF);
  scene.add(light);

  // Blue background color.
  renderer.setClearColor(0x00BFFF);

  // Set up cameras and controls.
  camera = new t.PerspectiveCamera(
      60,  // Field of view
      width / height,  // Aspect ratio
      1,  // Near plane
      10000  // Far plane
  );
  controls = new t.PointerLockControls(camera);
  cameraOrtho = new THREE.OrthographicCamera(
      -width / 2, width / 2, height / 2, -height / 2, 1, 10);
  cameraOrtho.position.z = 10;
  scene.add(controls.getObject());

  var havePointerLock = "pointerLockElement" in document ||
                        "mozPointerLockElement" in document ||
                        "webkitPointerLockElement" in document;
  if (!havePointerLock) {
    blockfort.menu.html("No pointer lock functionality detected!");
    return;
  }
  blockfort.element = document.body;
  // TODO(ariw): This breaks on Firefox since we don't requestFullscreen()
  // first.
  $(document).on("pointerlockchange", blockfort.pointerLockChange);
  $(document).on("webkitpointerlockchange", blockfort.pointerLockChange);
  $(document).on("mozpointerlockchange", blockfort.pointerLockChange);
  $(document).on("pointerlockerror", function(event) {});
  $(document).on("webkitpointerlockerror", function(event) {});
  $(document).on("mozpointerlockerror", function(event) {});
  blockfort.blocker.click(blockfort.pointerLockClick);
  blockfort.menu.click(function(event) { event.stopPropagation(); });

  blockfort.block_color.get(0).color.fromString("D4AF37");

  // Get the window ready.
  $(document.body).append(renderer.domElement);
  $(window).on("resize", onWindowResize);

  // Load world if previously specified.
  if ("id" in common.URL_PARAMETERS) {
    $.ajax({
        url: "load", type: "POST", async: false,
        data: { id: common.URL_PARAMETERS.id }, success: blockfort.deserialize,
        dataType: "json"
    });
  }

  // Begin updating.
  blockfort.update();
}

