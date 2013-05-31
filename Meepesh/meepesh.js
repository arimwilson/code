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
var MEEPESH = {
}

MEEPESH.update = function() {
  // Render the scene.
  requestAnimationFrame(MEEPESH.update);
  renderer.render(scene, camera);

  // Update controls.
  controls.update(Date.now() - time);
  time = Date.now();
}

MEEPESH.pointerLockChange = function(event) {
  if (document.pointerLockElement === MEEPESH.element ||
      document.webkitPointerLockElement === MEEPESH.element ||
      document.mozPointerLockElement === MEEPESH.element) {
    controls.enabled = true;
    document.addEventListener('click', MEEPESH.buildClick, false);
    document.addEventListener('keypress', MEEPESH.save, false);
    document.addEventListener('keypress', MEEPESH.load, false);

    MEEPESH.blocker.hide();
  } else {
    controls.enabled = false;
    document.removeEventListener('click', MEEPESH.buildClick, false);
    document.removeEventListener('keypress', MEEPESH.save, false);
    document.removeEventListener('keypress', MEEPESH.load, false);

    MEEPESH.blocker.show();
  }
}

MEEPESH.pointerLockClick = function(event) {
  MEEPESH.element.requestPointerLock =
      MEEPESH.element.requestPointerLock ||
      MEEPESH.element.webkitRequestPointerLock ||
      MEEPESH.element.mozRequestPointerLock;
  MEEPESH.element.requestPointerLock();
}

// Given world coordinates, return grid coordinates.
MEEPESH.gridCoordinates = function(v) {
  var u = new t.Vector3();
  u.x = Math.floor(v.x / MEEPESH.unitSize);
  u.y = Math.floor(v.y / MEEPESH.unitSize);
  u.z = Math.floor(v.z / MEEPESH.unitSize);
  return u;
}

MEEPESH.createFloor = function() {
  var geometry = new t.PlaneGeometry(
      MEEPESH.unitSize * MEEPESH.units, MEEPESH.unitSize * MEEPESH.units,
      MEEPESH.unitSize, MEEPESH.unitSize);
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
MEEPESH.createCube = function(v) {
  var cubeColor = 0xD4AF37;
  var cube = new t.Mesh(
      new t.CubeGeometry(MEEPESH.unitSize, MEEPESH.unitSize, MEEPESH.unitSize,
                         MEEPESH.unitSize, MEEPESH.unitSize, MEEPESH.unitSize),
      new t.MeshLambertMaterial(
          { color: cubeColor, ambient: cubeColor })
  );
  cube.position.set(v.x * MEEPESH.unitSize, (v.y + 0.5) * MEEPESH.unitSize,
                    v.z * MEEPESH.unitSize);
  return cube;
}

// Build blocks / destroy blocks controls.
MEEPESH.buildClick = function(event) {
  var direction = cameraDirection();
  var ray = new t.Raycaster(controls.getObject().position, direction);
  var intersects = ray.intersectObjects(MEEPESH.objects);

  if (intersects.length > 0) {
    if (event.which === 1) { // left click
      var cube = MEEPESH.createCube(MEEPESH.gridCoordinates(
          intersects[0].point.sub(direction)));
      scene.add(cube);
      MEEPESH.objects.push(cube);
    } else if (event.which === 3) { // right click
      var i = 0;
      for (; i < MEEPESH.objects.length; ++i) {
        if (MEEPESH.objects[i].id === intersects[0].object.id) {
          if (i != 0) MEEPESH.objects.remove(i);
          break;
        }
      }
      if (i != 0) scene.remove(intersects[0].object);
    }
  }
}

MEEPESH.save = function(event) {
  // z
  if (event.keyCode !== 122) return;
  MEEPESH.name = prompt("World name to save?", MEEPESH.name);
  var data = new Array();
  // Don't include floor in serialized objects.
  for (i = 1; i < MEEPESH.objects.length; ++i) {
    data.push(MEEPESH.gridCoordinates(MEEPESH.objects[i].position));
  }
  $.post("backend/save", { name: MEEPESH.name, data: JSON.stringify(data) });
}

MEEPESH.load = function(event) {
  // x
  if (event.keyCode != 120) return;
  MEEPESH.name = prompt("World name to load?", MEEPESH.name);
  $.ajax({
      url: "backend/load", type: 'POST', async: false,
      data: { name: MEEPESH.name },
      success: function(data) {
        // TODO(ariw): This algorithm is slow as balls.
        data = eval(data)
        if (data.length > 0) {
          // Remove existing objects from scene except floor.
          for (i = MEEPESH.objects.length - 1; i >= 1; --i) {
            scene.remove(MEEPESH.objects[i])
            MEEPESH.objects.remove(i);
          }
          // Add new objects to scene.
          objects = eval(data);
          for (i = 0; i < objects.length; ++i) {
            MEEPESH.objects.push(MEEPESH.createCube(objects[i]));
            scene.add(MEEPESH.objects[i + 1]);
          }
        }
      },
  });
}

MEEPESH.start = function() {
  t = THREE;
  renderer = new t.WebGLRenderer();
  width = document.body.clientWidth;
  height = document.body.clientHeight;
  renderer.setSize(width, height);
  scene = new t.Scene();
  time = Date.now();

  MEEPESH.blocker = $("#blocker");
  MEEPESH.menu = $("#menu");
  MEEPESH.unitSize = 20;
  MEEPESH.units = 1000;
  MEEPESH.name = "Default";
  MEEPESH.objects = new Array();

  // Floor.
  var floor = MEEPESH.createFloor();
  scene.add(floor);
  MEEPESH.objects.push(floor);

  // White ambient light.
  var light = new t.AmbientLight(0xFFFFFF);
  scene.add(light);

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
    MEEPESH.menu.html("No pointer lock functionality detected!");
    return;
  }
  MEEPESH.element = document.body;
  // TODO(ariw): This breaks on Firefox since we don't requestFullscreen()
  // first.
  document.addEventListener(
      'pointerlockchange', MEEPESH.pointerLockChange, false);
  document.addEventListener(
      'webkitpointerlockchange', MEEPESH.pointerLockChange, false);
  document.addEventListener(
      'mozpointerlockchange', MEEPESH.pointerLockChange, false);
  document.addEventListener(
      'pointerlockerror', function(event) {}, false);
  document.addEventListener(
      'webkitpointerlockerror', function(event) {}, false);
  document.addEventListener(
      'mozpointerlockerror', function(event) {}, false);
  MEEPESH.blocker.click(MEEPESH.pointerLockClick);

  // Get the window ready.
  document.body.appendChild(renderer.domElement);
  window.addEventListener('resize', onWindowResize, false);

  // Begin updating.
  MEEPESH.update();
}

