var HOME = {
}

HOME.animate = function() {
  // HOME.controls.update();

  // Render the scene.
  HOME.renderer.render(HOME.scene, HOME.camera);
  window.webkitRequestAnimationFrame(HOME.animate);
}

HOME.start = function() {
  HOME.renderer = new THREE.WebGLRenderer();
  HOME.width = document.body.clientWidth;
  HOME.height = document.body.clientHeight;
  HOME.renderer.setSize(HOME.width, HOME.height);
  $(document.body).append(HOME.renderer.domElement);

  // Set up the initial scene.
  HOME.scene = new THREE.Scene();

  HOME.camera = new THREE.PerspectiveCamera(
    35,  // Field of view
    HOME.width / HOME.height,  // Aspect ratio
    0.1,  // Near plane
    10000  // Far plane
  );
  HOME.camera.position.set(20, 20, 0);
  HOME.camera.up.x = 0;
  HOME.camera.up.y = 0;
  HOME.camera.up.z = 1;
  HOME.camera.lookAt(HOME.scene.position);

  HOME.scene.add(HOME.camera);

  var cube = new THREE.Mesh(
    new THREE.CubeGeometry(5, 5, 5),
    new THREE.MeshLambertMaterial({ color: 0xFF0000 })
  );
  HOME.scene.add(cube);
  var cube2 = new THREE.Mesh(
    new THREE.CubeGeometry(500, 500, 5),
    new THREE.MeshLambertMaterial({ color: 0x00FF00 })
  );
  cube2.translateX(250);
  cube2.translateY(250);
  cube2.translateZ(-5);
  HOME.scene.add(cube2);

  var light = new THREE.PointLight(0xFFFF00);
  light.position.set(10, 0, 10);
  HOME.scene.add(light);

  // Set up controls.
  HOME.controls = new THREE.FirstPersonControls(HOME.camera);
  HOME.controls.noFly = true;

  HOME.animate();
}

