/**
 * @author mrdoob / http://mrdoob.com/
 */

THREE.PointerLockControls = function ( camera ) {

  var scope = this;

  var pitchObject = new THREE.Object3D();
  pitchObject.add( camera );

  var yawObject = new THREE.Object3D();
  yawObject.position.y = 10;
  yawObject.add( pitchObject );

  var moveForward = false;
  var moveBackward = false;
  var moveLeft = false;
  var moveRight = false;
  var flyUp = false;
  var flyDown = false;

  var velocity = new THREE.Vector3();

  var PI_2 = Math.PI / 2;

  var onMouseMove = function ( event ) {

    if ( scope.enabled === false ) return;

    var movementX = event.movementX || event.mozMovementX ||
                    event.webkitMovementX || 0;
    var movementY = event.movementY || event.mozMovementY ||
                    event.webkitMovementY || 0;

    yawObject.rotation.y -= movementX * 0.002;
    pitchObject.rotation.x -= movementY * 0.002;

    pitchObject.rotation.x = Math.max( - PI_2, Math.min(
        PI_2, pitchObject.rotation.x ) );

  };

  var onKeyDown = function ( event ) {

    switch ( event.keyCode ) {

      case 38: // up
      case 87: // w
        moveForward = true;
        break;

      case 37: // left
      case 65: // a
        moveLeft = true;
        break;

      case 40: // down
      case 83: // s
        moveBackward = true;
        break;

      case 39: // right
      case 68: // d
        moveRight = true;
        break;

      case 32: // space
        flyUp = true;
        break;
      case 16: // shift
        flyDown = true;
        break;

    }

  };

  var onKeyUp = function ( event ) {

    switch( event.keyCode ) {

      case 38: // up
      case 87: // w
        moveForward = false;
        break;

      case 37: // left
      case 65: // a
        moveLeft = false;
        break;

      case 40: // down
      case 83: // a
        moveBackward = false;
        break;

      case 39: // right
      case 68: // d
        moveRight = false;
        break;

      case 32: // space
        flyUp = false;
        break;
      case 16: // shift
        flyDown = false;
        break;
    }

  };

  document.addEventListener( 'mousemove', onMouseMove, false );
  document.addEventListener( 'keydown', onKeyDown, false );
  document.addEventListener( 'keyup', onKeyUp, false );

  this.enabled = false;

  this.getObject = function () {

    return yawObject;

  };

  this.update = function ( delta ) {

    if ( scope.enabled === false ) return;

    delta *= 0.1;

    velocity.x += ( - velocity.x ) * 0.08 * delta;
    velocity.z += ( - velocity.z ) * 0.08 * delta;
    velocity.y += ( - velocity.y ) * 0.08 * delta;

    if ( moveForward ) velocity.z -= 0.12 * delta;
    if ( moveBackward ) velocity.z += 0.12 * delta;

    if ( moveLeft ) velocity.x -= 0.12 * delta;
    if ( moveRight ) velocity.x += 0.12 * delta;

    if ( flyUp ) velocity.y += 0.12 * delta;
    if ( flyDown ) velocity.y -= 0.12 * delta;

    yawObject.translateX( velocity.x );
    yawObject.translateY( velocity.y );
    yawObject.translateZ( velocity.z );

    if ( yawObject.position.y < 10 ) {

      velocity.y = 0;
      yawObject.position.y = 10;

    }

  };

};