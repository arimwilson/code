var HOME = {
}

HOME.keydown(event) {
  if (event.keyCode == 37) {  // left
    HOME.horizontalSpeed -= 1;
    return false;
  } else if (event.keyCode == 38) {  // up
    HOME.forwardSpeed += 1;
    return false;
  } else if (event.keyCode == 39) {  // right
    HOME.horizontalSpeed += 1;
    return false;
  } else if (event.keyCode == 40) { // down
    HOME.forwardSpeed -= 1;
    return false;
  }
}

HOME.keyup(event) {
  if (event.keyCode == 37) {  // left
    HOME.horizontalSpeed += 1;
    return false;
  } else if (event.keyCode == 38) {  // up
    HOME.forwardSpeed -= 1;
    return false;
  } else if (event.keyCode == 39) {  // right
    HOME.horizontalSpeed -= 1;
    return false;
  } else if (event.keyCode == 40) { // down
    HOME.forwardSpeed += 1;
    return false;
  }
}

HOME.mousemove(event) {
}

HOME.animate = function() {
  // TODO(ariw): Update HOME.camera.position.
  HOME.renderer.render(HOME.scene, HOME.camera);
  window.webkitRequestAnimationFrame(HOME.animate);
}

HOME.start = function() {
  HOME.renderer = new THREE.WebGLRenderer();
  HOME.renderer.setSize(document.body.clientWidth, document.body.clientHeight);
  document.body.appendChild(HOME.renderer.domElement);

  // Set up the initial scene.
  HOME.scene = new THREE.Scene();

  HOME.camera = new THREE.PerspectiveCamera(
    35,  // Field of view
    document.body.clientWidth / document.body.clientHeight,  // Aspect ratio
    0.1,  // Near plane
    10000  // Far plane
  );
  HOME.camera.position.set(0, 0, 30);
  HOME.camera.lookAt(scene.position);

  HOME.scene.add(camera);

  var cube = new THREE.Mesh(
    new THREE.CubeGeometry(5, 5, 5),
    new THREE.MeshLambertMaterial({ color: 0xFF0000 })
  );
  HOME.scene.add(cube);

  var light = new THREE.PointLight(0xFFFF00);
  light.position.set(10, 0, 10);
  HOME.scene.add(light);

  // Keys ready to press.
  HOME.horizontalSpeed = 0;
  HOME.forwardSpeed = 0;
  $("document").keydown(HOME.keydown);
  $("document").keyup(HOME.keyup);
  $("document").mousemove(HOME.mousemove);

  HOME.animate();
}

