// Global object since I suck at programming.
var MEEPESH = {
}

MEEPESH.update = function() {
  // Render the scene.
  requestAnimationFrame(MEEPESH.update);
  MEEPESH.renderer.render(MEEPESH.scene, MEEPESH.camera);

  // Update controls.
  MEEPESH.controls.update(Date.now() - MEEPESH.time);
  MEEPESH.time = Date.now();
}

MEEPESH.onWindowResize = function() {
  MEEPESH.camera.aspect = window.innerWidth / window.innerHeight;
  MEEPESH.camera.updateProjectionMatrix();
  MEEPESH.renderer.setSize(window.innerWidth, window.innerHeight);
}

MEEPESH.pointerLockChange = function(event) {
  if (document.pointerLockElement === MEEPESH.element ||
      document.webkitPointerLockElement === MEEPESH.element ||
      document.mozPointerLockElement === MEEPESH.element) {
    MEEPESH.controls.enabled = true;
  } else {
    MEEPESH.controls.enabled = false;
  }
}

MEEPESH.start = function() {
  MEEPESH.renderer = new THREE.WebGLRenderer();
  MEEPESH.width = document.body.clientWidth;
  MEEPESH.height = document.body.clientHeight;
  MEEPESH.renderer.setSize(MEEPESH.width, MEEPESH.height);

  // Set up the initial scene.
  MEEPESH.scene = new THREE.Scene();

  MEEPESH.camera = new THREE.PerspectiveCamera(
      35,  // Field of view
      MEEPESH.width / MEEPESH.height,  // Aspect ratio
      0.1,  // Near plane
      10000  // Far plane
  );
  MEEPESH.camera.position.set(20, 20, 0);
  MEEPESH.camera.up.x = 0;
  MEEPESH.camera.up.y = 0;
  MEEPESH.camera.up.z = 1;
  MEEPESH.camera.lookAt(MEEPESH.scene.position);
  MEEPESH.scene.add(MEEPESH.camera);

  var cube = new THREE.Mesh(
      new THREE.CubeGeometry(5, 5, 5),
      new THREE.MeshLambertMaterial({ color: 0xFF0000 })
  );
  MEEPESH.scene.add(cube);
  var cube2 = new THREE.Mesh(
      new THREE.CubeGeometry(500, 500, 5),
      new THREE.MeshLambertMaterial({ color: 0x00FF00 })
  );
  cube2.translateX(250);
  cube2.translateY(250);
  cube2.translateZ(-5);
  MEEPESH.scene.add(cube2);

  var light = new THREE.PointLight(0xFFFF00);
  light.position.set(10, 0, 10);
  MEEPESH.scene.add(light);

  // Set up controls.
  MEEPESH.controls = new THREE.PointerLockControls(MEEPESH.camera);
  MEEPESH.scene.add(MEEPESH.controls.getObject());
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
  MEEPESH.element.addEventListener('click', function(event) {
    MEEPESH.element.requestPointerLock =
        MEEPESH.element.requestPointerLock ||
        MEEPESH.element.webkitRequestPointerLock ||
        MEEPESH.element.mozRequestPointerLock;
    MEEPESH.element.requestPointerLock();
  }, false);


  // Get the window ready.
  document.body.appendChild(MEEPESH.renderer.domElement);
  window.addEventListener('resize', MEEPESH.onWindowResize, false);

  // Begin updating.
  MEEPESH.time = Date.now();
  MEEPESH.update();
}

