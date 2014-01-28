// Generic three.js objects are in the global namespace.
var t, renderer, scene, width, height, camera, controls, time;

onWindowResize = function() {
  width = window.innerWidth;
  height = window.innerHeight;
  camera.aspect = width / height;
  camera.updateProjectionMatrix();
  renderer.setSize(width, height);
}

cameraDirection = function() {
  // Transform point in front of camera in camera space into global space to
  // find the direction of the camera.
  return new t.Vector3(0, 0, -1).applyMatrix4(camera.matrixWorld).sub(
      controls.getObject().position).normalize();
}

// Global object for scene-specific stuff.
var BLOCKFORT = {
}

BLOCKFORT.update = function() {
  // Render the scene.
  requestAnimationFrame(BLOCKFORT.update);
  renderer.render(scene, camera);

  // Update controls.
  controls.update(Date.now() - time);
  time = Date.now();
}

BLOCKFORT.pointerLockChange = function(event) {
  if (document.pointerLockElement === BLOCKFORT.element ||
      document.webkitPointerLockElement === BLOCKFORT.element ||
      document.mozPointerLockElement === BLOCKFORT.element) {
    controls.enabled = true;
    $(document).click(BLOCKFORT.buildClick);
    $(document).keypress(BLOCKFORT.save);
    $(document).keypress(BLOCKFORT.load);

    BLOCKFORT.blocker.hide();
  } else {
    controls.enabled = false;
    $(document).off('click');
    $(document).off('keypress');

    BLOCKFORT.blocker.show();
  }
}

BLOCKFORT.pointerLockClick = function(event) {
  BLOCKFORT.element.requestPointerLock =
      BLOCKFORT.element.requestPointerLock ||
      BLOCKFORT.element.webkitRequestPointerLock ||
      BLOCKFORT.element.mozRequestPointerLock;
  BLOCKFORT.element.requestPointerLock();
}

// Given world coordinates, return grid coordinates.
BLOCKFORT.gridCoordinates = function(v) {
  var u = new t.Vector3();
  u.x = Math.floor(v.x / BLOCKFORT.unitSize);
  u.y = Math.floor(v.y / BLOCKFORT.unitSize);
  u.z = Math.floor(v.z / BLOCKFORT.unitSize);
  return u;
}

BLOCKFORT.createFloor = function() {
  var geometry = new t.PlaneGeometry(
      BLOCKFORT.unitSize * BLOCKFORT.units, BLOCKFORT.unitSize * BLOCKFORT.units,
      BLOCKFORT.unitSize, BLOCKFORT.unitSize);
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
BLOCKFORT.createCube = function(v, color) {
  var cube = new t.Mesh(
      new t.CubeGeometry(BLOCKFORT.unitSize, BLOCKFORT.unitSize,
                         BLOCKFORT.unitSize, BLOCKFORT.unitSize,
                         BLOCKFORT.unitSize, BLOCKFORT.unitSize),
      new t.MeshLambertMaterial({ color: color, ambient: color })
  );
  cube.position.set(v.x * BLOCKFORT.unitSize, (v.y + 0.5) * BLOCKFORT.unitSize,
                    v.z * BLOCKFORT.unitSize);
  return cube;
}

// Build blocks / destroy blocks controls.
BLOCKFORT.buildClick = function(event) {
  var direction = cameraDirection();
  var ray = new t.Raycaster(controls.getObject().position, direction);
  var intersects = ray.intersectObjects(BLOCKFORT.objects);

  if (intersects.length > 0) {
    if (event.which === 1) { // left click
      var cube = BLOCKFORT.createCube(
          BLOCKFORT.gridCoordinates(intersects[0].point.sub(direction)),
          "#" + BLOCKFORT.block_color.val());
      scene.add(cube);
      BLOCKFORT.objects.push(cube);
    } else if (event.which === 3) { // right click
      var i = 0;
      for (; i < BLOCKFORT.objects.length; ++i) {
        if (BLOCKFORT.objects[i].id === intersects[0].object.id) {
          if (i != 0) BLOCKFORT.objects.remove(i);
          break;
        }
      }
      if (i != 0) scene.remove(intersects[0].object);
    }
  }
}

// Convert rendered world into a simplified format suitable for later
// retrieval.
BLOCKFORT.serialize = function() {
  var data = {};
  data.position = controls.getObject().position;
  data.rotation = {};
  data.rotation.x = controls.getObject().rotation._x;
  data.rotation.y = controls.getObject().rotation._y;
  data.rotation.z = controls.getObject().rotation._z;
  data.objects = new Array();
  // Don't include floor in serialized objects.
  for (i = 1; i < BLOCKFORT.objects.length; ++i) {
    var object = {};
    object.position = BLOCKFORT.gridCoordinates(BLOCKFORT.objects[i].position);
    object.color = BLOCKFORT.objects[i].material.color.getHex();
    data.objects.push(object);
  }
  console.log(data);
  return JSON.stringify(data);
}

BLOCKFORT.save = function(event) {
  // z
  if (event.keyCode !== 122) return;
  BLOCKFORT.name = prompt("World name to save?", BLOCKFORT.name);
  $.post("backend/save", {
      name: BLOCKFORT.name, data: BLOCKFORT.serialize()
  });
}

// Convert simplified format into rendered world.
BLOCKFORT.deserialize = function(data) {
  // TODO(ariw): This algorithm is slow as balls.
  if (data.length > 0) {
    data = JSON.parse(JSON.parse(data));

    // Remove existing objects from scene except floor.
    for (i = BLOCKFORT.objects.length - 1; i >= 1; --i) {
      scene.remove(BLOCKFORT.objects[i])
      BLOCKFORT.objects.remove(i);
    }
    // Load scene.
    var objects;
    // TODO(ariw): Remove this legacy mode.
    if (data instanceof Array) {
      objects = data;
    } else {
      objects = data.objects;
      controls.getObject().position.copy(data.position);
      controls.getObject().rotation.set(
          data.rotation.x, data.rotation.y, data.rotation.z);
    }
    for (i = 0; i < objects.length; ++i) {
      BLOCKFORT.objects.push(BLOCKFORT.createCube(
          objects[i].position, objects[i].color));
      scene.add(BLOCKFORT.objects[i + 1]);
    }
  }
}

BLOCKFORT.load = function(event) {
  // x
  if (event.keyCode != 120) return;
  BLOCKFORT.name = prompt("World name to load?", BLOCKFORT.name);
  $.ajax({
      url: "backend/load", type: 'POST', async: false,
      data: { name: BLOCKFORT.name }, success: BLOCKFORT.deserialize,
  });
}

BLOCKFORT.start = function() {
  t = THREE;
  renderer = new t.WebGLRenderer();
  width = document.body.clientWidth;
  height = document.body.clientHeight;
  renderer.setSize(width, height);
  scene = new t.Scene();
  time = Date.now();

  BLOCKFORT.blocker = $("#blocker");
  BLOCKFORT.menu = $("#menu");
  BLOCKFORT.block_color = $("#block_color");
  BLOCKFORT.unitSize = 20;
  BLOCKFORT.units = 1000;
  BLOCKFORT.name = "Default";
  BLOCKFORT.objects = new Array();

  // Floor.
  var floor = BLOCKFORT.createFloor();
  scene.add(floor);
  BLOCKFORT.objects.push(floor);

  // White ambient light.
  var light = new t.AmbientLight(0xFFFFFF);
  scene.add(light);

  // Blue background color.
  renderer.setClearColor(0x00BFFF);

  // Set up controls.
  camera = new t.PerspectiveCamera(
      60,  // Field of view
      width / height,  // Aspect ratio
      1,  // Near plane
      10000  // Far plane
  );
  controls = new t.PointerLockControls(camera);
  scene.add(controls.getObject());
  var havePointerLock = 'pointerLockElement' in document ||
                        'mozPointerLockElement' in document ||
                        'webkitPointerLockElement' in document;
  if (!havePointerLock) {
    BLOCKFORT.menu.html("No pointer lock functionality detected!");
    return;
  }
  BLOCKFORT.element = document.body;
  // TODO(ariw): This breaks on Firefox since we don't requestFullscreen()
  // first.
  $(document).on('pointerlockchange', BLOCKFORT.pointerLockChange);
  $(document).on('webkitpointerlockchange', BLOCKFORT.pointerLockChange);
  $(document).on('mozpointerlockchange', BLOCKFORT.pointerLockChange);
  $(document).on('pointerlockerror', function(event) {});
  $(document).on('webkitpointerlockerror', function(event) {});
  $(document).on('mozpointerlockerror', function(event) {});
  BLOCKFORT.blocker.click(BLOCKFORT.pointerLockClick);
  BLOCKFORT.menu.click(function(event) { event.stopPropagation(); });

  BLOCKFORT.block_color.get(0).color.fromString("D4AF37");

  // Get the window ready.
  $(document.body).append(renderer.domElement);
  $(window).on('resize', onWindowResize);

  // Begin updating.
  BLOCKFORT.update();
}

