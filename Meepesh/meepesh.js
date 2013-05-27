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

MEEPESH.load = function() {
  $.ajax({
      url: "backend/load", type: 'POST', async: false,
      success: function(objects) {
        objects = eval(objects)
        if (objects.length > 0) {
          objects = eval(objects);
          for (i = 0; i < objects.length; ++i) {
            objects[i] = MEEPESH.createCube(objects[i]);
            scene.add(objects[i]);
          }
          MEEPESH.objects = MEEPESH.objects.concat(objects);
        }
      },
  });
}

MEEPESH.pointerLockChange = function(event) {
  if (document.pointerLockElement === MEEPESH.element ||
      document.webkitPointerLockElement === MEEPESH.element ||
      document.mozPointerLockElement === MEEPESH.element) {
    controls.enabled = true;
    document.removeEventListener('click', MEEPESH.pointerLockClick, false);
    document.addEventListener('click', MEEPESH.buildClick, false);
    document.addEventListener('keypress', MEEPESH.save, false);
  } else {
    controls.enabled = false;
    document.removeEventListener('click', MEEPESH.buildClick, false);
    document.removeEventListener('keypress', MEEPESH.save, false);
    document.addEventListener('click', MEEPESH.pointerLockClick, false);
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

// v should be in grid coordinates.
MEEPESH.createCube = function(v) {
  var cube = new t.Mesh(
      new t.CubeGeometry(MEEPESH.unitSize, MEEPESH.unitSize, MEEPESH.unitSize,
                         1, 1, 1),
      new t.MeshLambertMaterial(
          { color: MEEPESH.cubeColor, ambient: MEEPESH.cubeColor })
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
  if (event.keyCode !== 122) return;
  var objects = new Array();
  // Don't include floor in serialized objects.
  for (i = 1; i < MEEPESH.objects.length; ++i) {
    objects.push(MEEPESH.gridCoordinates(MEEPESH.objects[i].position));
  }
  $.post("backend/save", { objects: JSON.stringify(objects) });
}

MEEPESH.start = function() {
  t = THREE;
  renderer = new t.WebGLRenderer();
  width = document.body.clientWidth;
  height = document.body.clientHeight;
  renderer.setSize(width, height);
  scene = new t.Scene();
  time = Date.now();

  MEEPESH.unitSize = 20;
  MEEPESH.units = 1000;

  MEEPESH.cubeColor = 0xD4AF37;

  // Green grass floor
  MEEPESH.objects = new Array();
  var geometry = new t.PlaneGeometry(
      MEEPESH.unitSize * MEEPESH.units, MEEPESH.unitSize * MEEPESH.units,
      MEEPESH.units, MEEPESH.units);
  // Floors generally are on the xz plane rather than the yz plane. Rotate it
  // there :).
  geometry.applyMatrix(new t.Matrix4().makeRotationX(-Math.PI / 2));
  floorColor = 0x395D33;
  var floor = new t.Mesh(
      geometry, new t.MeshLambertMaterial(
          { color: floorColor, ambient: floorColor })
  );
  scene.add(floor);
  MEEPESH.objects.push(floor);

  // Existing cubes.
  MEEPESH.load();

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
    alert("No pointer lock functionality detected!");
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
  document.addEventListener('click', MEEPESH.pointerLockClick, false);

  // Get the window ready.
  document.body.appendChild(renderer.domElement);
  window.addEventListener('resize', onWindowResize, false);

  // Begin updating.
  MEEPESH.update();
}

