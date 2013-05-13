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
        if (objects.length > 0) {
          MEEPESH.objects = eval(objects);
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

MEEPESH.createCube = function(v) {
  // Return in grid coordinates where this cube should be located.
  v = (function(v) {
    v.x = Math.floor(v.x / MEEPESH.unitSize);
    v.y = Math.floor(v.y / MEEPESH.unitSize);
    v.z = Math.floor(v.z / MEEPESH.unitSize);
    return v;
  })(v);
  var cube = new t.Mesh(
      new t.CubeGeometry(MEEPESH.unitSize, MEEPESH.unitSize, MEEPESH.unitSize,
                         MEEPESH.unitSize, MEEPESH.unitSize, MEEPESH.unitSize),
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
      var cube = MEEPESH.createCube(intersects[0].point.sub(direction));
      scene.add(cube);
      MEEPESH.objects.push(cube);
    } else if (event.which === 3 && // right click
               intersects[0].object.id !== 0) {
      for (i = 0; i < MEEPESH.objects.length; ++i) {
        if (MEEPESH.objects[i].id === intersects[0].object.id) {
          MEEPESH.objects.remove(i);
          break;
        }
      }
      scene.remove(intersects[0].object);
    }
  }
}

MEEPESH.save = function(event) {
  console.log(event.keyCode);
  if (event.keyCode !== 122) return;
  // TODO(ariw): Fix this so it doesn't crash and die.
  // $.post("backend/save", MEEPESH.objects);
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

  MEEPESH.load();
  if (typeof MEEPESH.objects !== "undefined") {
    for (i = 0; i < MEEPESH.objects.length; ++i) {
      scene.add(MEEPESH.objects[i]);
    }
  } else {
    // Green grass floor
    MEEPESH.objects = new Array();
    var geometry = new t.PlaneGeometry(
        MEEPESH.unitSize * MEEPESH.units, MEEPESH.unitSize * MEEPESH.units,
        MEEPESH.unitSize, MEEPESH.unitSize);
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
  }

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

