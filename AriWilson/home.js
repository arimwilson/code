var HOME = {
}

HOME.keydown = function(event) {
  if (event.keyCode == 37) {  // left
    HOME.horizontalSpeed = -1;
    return false;
  } else if (event.keyCode == 38) {  // up
    HOME.forwardSpeed = 1;
    return false;
  } else if (event.keyCode == 39) {  // right
    HOME.horizontalSpeed = 1;
    return false;
  } else if (event.keyCode == 40) { // down
    HOME.forwardSpeed = -1;
    return false;
  }
}

HOME.keyup = function(event) {
  if (event.keyCode == 37 || event.keyCode == 39) {  // left/right
    HOME.horizontalSpeed = 0;
    return false;
  } else if (event.keyCode == 38 || event.keyCode == 40) {  // up/down
    HOME.forwardSpeed = 0;
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
  var timeDiff = 0;
  if (HOME.oldTime) {
    timeDiff = time - HOME.oldTime;
  }
  HOME.oldTime = time;

  // Adjust where we're looking based on the mouse.
  var look = HOME.look;
  /* TODO(ariw): look.setRotationFromAxis()?
  var rotation = new Matrix4();

  look.x = Math.cos(Math.atan2(look.x, look.y) +
                    HOME.horizontalRotate * 2 * Math.Pi / 360);
  look.y = Math.sin(Math.atan2(look.x, look.y) +
                    HOME.horizontalRotate * 2 * Math.Pi / 360);
  look.z = ;*/
  HOME.horizontalRotate = 0;
  HOME.verticalRotate = 0;
  look = HOME.look.clone();

  // Adjust where we are based on the keyboard.
  HOME.camera.translate(HOME.forwardSpeed, look);
  // TODO(ariw): Horizontal?

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
  HOME.camera.position.set(0, 30, 30);
  HOME.camera.lookAt(HOME.scene.position);
  HOME.look = new THREE.Vector3();
  HOME.look.sub(HOME.scene.position, HOME.camera.position);

  HOME.scene.add(HOME.camera);

  var cube = new THREE.Mesh(
    new THREE.CubeGeometry(5, 5, 5),
    new THREE.MeshLambertMaterial({ color: 0xFF0000 })
  );
  HOME.scene.add(cube);

  var light = new THREE.PointLight(0xFFFF00);
  light.position.set(10, 0, 10);
  HOME.scene.add(light);

  HOME.horizontalRotate = 0;
  HOME.verticalRotate = 0;
  HOME.horizontalSpeed = 0;
  HOME.forwardSpeed = 0;

  // Keys ready to press.
  $(document).keydown(HOME.keydown);
  $(document).keyup(HOME.keyup);
  $(document).mousemove(HOME.mousemove);

  HOME.animate();
}

