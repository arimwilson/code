var HOME = {
}

HOME.keydown = function(event) {
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

HOME.keyup = function(event) {
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

HOME.mousemove = function(event) {
  if (!HOME.pageX || !HOME.pageY) {
    HOME.pageX = event.pageX;
    HOME.pageY = event.pageY;
    return;
  }
  // TODO(ariw): Figure out real formula here.
  HOME.horizontalRotate += 10 * (event.pageX - HOME.pageX) / HOME.width;
  HOME.verticalRotate += 10 * (event.pageY - HOME.pageY) / HOME.height;
}

HOME.animate = function() {
  // See how much time has passed since the last frame.
  var time = Date.now();
  if (!HOME.oldTime) {
    HOME.oldTime = time;
    return;
  }
  var timeDiff = time - HOME.oldTime;
  HOME.oldTime = time;

  // Adjust where we're looking based on the mouse.
  var look = HOME.camera.look;

  look.x = Math.cos(Math.atan2(look.x, look.y) +
                    HOME.horizontalRotate * 2 * Math.Pi / 360);
  look.y = Math.sin(Math.atan2(look.x, look.y) +
                    HOME.horizontalRotate * 2 * Math.Pi / 360);
  look.z = ;
  HOME.camera.lookAt(look);
  HOME.horizontalRotate = 0;
  HOME.verticalRotate = 0;

  // Adjust where we are based on the keyboard.
  var position = HOME.camera.position;
  // TODO(ariw): Update HOME.camera.position.

  // Render the scene.
  HOME.renderer.render(HOME.scene, HOME.camera);
  window.webkitRequestAnimationFrame(HOME.animate);
}

HOME.start = function() {
  HOME.renderer = new THREE.WebGLRenderer();
  HOME.width = document.body.clientWidth;
  HOME.height = document.body.clientHeight;
  HOME.renderer.setSize(HOME.width, HOME.height);
  document.body.appendChild(HOME.renderer.domElement);

  // Set up the initial scene.
  HOME.scene = new THREE.Scene();

  HOME.camera = new THREE.PerspectiveCamera(
    35,  // Field of view
    HOME.width / HOME.height,  // Aspect ratio
    0.1,  // Near plane
    10000  // Far plane
  );
  HOME.camera.position.set(0, 0, 30);
  HOME.camera.look = HOME.scene.position.clone().normalize();

  HOME.scene.add(HOME.camera);

  var cube = new THREE.Mesh(
    new THREE.CubeGeometry(5, 5, 5),
    new THREE.MeshLambertMaterial({ color: 0xFF0000 })
  );
  HOME.scene.add(cube);

  var light = new THREE.PointLight(0xFFFF00);
  light.position.set(10, 0, 10);
  HOME.scene.add(light);

  HOME.animate();

  // Keys ready to press.
  HOME.horizontalSpeed = 0;
  HOME.forwardSpeed = 0;
  $("document").keydown(HOME.keydown);
  $("document").keyup(HOME.keyup);
  $("document").mousemove(HOME.mousemove);
}

