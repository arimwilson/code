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
  if (document.pointerLockElement == MEEPESH.element) {
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
  $(document.body).append(MEEPESH.renderer.domElement);

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
  document.addEventListener(
      'pointerlockchange', MEEPESH.pointerLockChange, false);
  document.addEventListener(
      'webkitpointerlockchange', MEEPESH.pointerLockChange, false);
  MEEPESH.element = document.body;
  MEEPESH.element.webkitRequestPointerLock();

  window.addEventListener('resize', MEEPESH.onWindowResize, false);

  MEEPESH.time = Date.now();
  MEEPESH.update();
}

