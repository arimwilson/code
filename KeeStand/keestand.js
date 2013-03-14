// Global love.
var username_ = "";
var password_ = "";
var salt_ = "";
var password_hash_ = "";
var version_ = 1;
var local_storage_ = false;
var stop_ = false;

function AssertException(message) {
  this.message = message;
}

AssertException.prototype.toString = function () {
  return "AssertException: " + this.message;
}

function assert(exp, message) {
  if (!exp) {
    throw new AssertException(message);
  }
}


function decrypt(data) {
  assert(password_, "password_ is null.");
  return sjcl.decrypt(password_, data);
}

function create_accordion(active) {
  $("#data > div > h3").click(function(event) {
    if (stop_) {
      event.stopImmediatePropagation();
      event.preventDefault();
      stop_ = false;
    }
  });
  $("#data").accordion({collapsible: true, active: active,
                        header: "> div > h3"})
            .sortable({axis: "y", handle: "h3",
                       stop: function() { stop_ = true; }});
}

function add_input(input) {
  var output = $(
      "<div><h3><a href='#'><span class='org'>" + input[0] + "</span></a>" +
      "</h3><div><textarea rows=1 cols=20>" + input[1] + "</textarea>" +
      "<textarea rows=1 cols=20>" + input[2] + "</textarea>" +
      "<textarea rows=1 cols=20>" + input[3] + "</textarea>" +
      "<input type='button' value='Delete'></div>");
  output.find(".org").click(function(event) {
    var srcElement = event.srcElement? $(event.srcElement): $(event.target);
    var name = prompt("New name?", srcElement.html());
    if (name !== null && name !== "")
      srcElement.html(name);
    event.stopImmediatePropagation();
  });
  output.find("input:button").click(function(event) {
    var sure = confirm("Delete this password?");
    if (sure) {
      $("#data").accordion("destroy");
      var srcElement = event.srcElement? event.srcElement: event.target;
      var div = $(srcElement.parentNode.parentNode);
      div.remove();
      create_accordion(false);
    }
  });
  return output;
}

function editor(data) {
  $("#data").accordion("destroy").empty();
  data = data.split("\n");
  for (i = 0; i < data.length - 1; ++i) {
    data[i] = data[i].split(",");
    for (j = 0; j < data[i].length; ++j) {
      data[i][j] = unescape(data[i][j]);
    }
    $("#data").append(add_input(data[i]));
  }
  create_accordion(false);
}

function password_hash(password, salt) {
  hash = sjcl.hash.sha256.hash(password + salt);
  // TODO(ariw): Is 1000 right here? I wish I could use bcrypt...
  for (i = 1; i < 1000; ++i)
    hash = sjcl.hash.sha256.hash(hash);
  return sjcl.codec.base64.fromBits(hash, false);
}

function salt_success(data) {
  salt_ = data;
  password_hash_ = password_hash(password_, salt_);
  $.post("script/login",
         { username: username_, password_hash: password_hash_, salt: salt_ },
         function(data) {
           $("#login").hide();
           $("#edit").show();
           // Determine whether or not to use local storage based on last
           // modification date.
           if (local_storage_) {
             last_modified_local = new Date(localStorage["last_modified"]);
           }
           if (data) {
             last_modified = new Date(1000 * data["last_modified"]);
           }
           if (data &&
               (!local_storage_ || last_modified >= last_modified_local)) {
             editor(decrypt(data["passwords"]));
             version_ = data["version"];
             save_local(data["passwords"], last_modified);
           } else if (local_storage_) {
             editor(decrypt(localStorage["passwords"]));
             version_ = parseInt(localStorage["version"])
             save(localStorage["passwords"]);
           }
         }, "json");
}

function salt_error(xhr, text_status, error_thrown) {
  if (local_storage_) {
    salt_ = localStorage["salt"];
    password_hash_ = localStorage["password_hash"];
    $("#login").hide();
    $("#edit").show();
    editor(decrypt(localStorage["passwords"]));
  }
}

function create_csv(input) {
  data = "";
  for (i = 0; i < input.length; i += 4) {
    data += escape(input[i].innerHTML) + "," +
            escape(input[i + 1].value) + "," +
            escape(input[i + 2].value) + "," +
            escape(input[i + 3].value) + "\n";
  }
  return data;
}

function encrypt(data) {
  assert(password_, "password_ is null.");
  return sjcl.encrypt(password_, data);
}

function save_local(passwords, last_modified) {
  assert(username_ && salt_ && password_hash_,
         "username_ or salt_ or password_hash_ is null.");
  localStorage["username"] = username_;
  localStorage["salt"] = salt_;
  localStorage["password_hash"] = password_hash_;
  localStorage["passwords"] = passwords;
  localStorage["last_modified"] = last_modified.toUTCString();
  localStorage["version"] = version_;
}

function save(passwords) {
  // TODO(ariw): Notify when successful.
  $.post("script/save", { passwords: passwords, version: version_ });
}

function get_random_num(lower, upper) {
  return (Math.floor(Math.random() * (upper - lower)) + lower);
}

var char_set = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVW" +
               "XYZ`~!@#$%^&*()-_=+[{]}\\|;:'\",<.>/? ";
function get_random_char() {
  return char_set.charAt(get_random_num(0, char_set.length));
}

function generate_password(length) {
  password = "";
  for (i = 0; i < length; ++i) {
    password += get_random_char();
  }
  return password;
}

$(document).ready(function() {
  // Display last used username, as a convenience.
  if (localStorage.getItem("username"))
    $("#username").val(localStorage["username"]);

  // TODO(ariw): Validation on password length/content.
  $("#login_button").click(function() {
    username_ = $("#username").val();
    password_ = $("#password").val();
    local_storage_ =
        localStorage.getItem("username") &&
        localStorage["username"] === username_ &&
        localStorage.getItem("salt") &&
        localStorage.getItem("password_hash") &&
        localStorage.getItem("last_modified") &&
        localStorage.getItem("version") &&
        localStorage.getItem("passwords") &&
        password_hash(password_, localStorage["salt"]) === localStorage[
            "password_hash"];
    $.ajax({type: "POST", url: "script/salt", data: { username: username_ },
            timeout: 3000, success: salt_success, error: salt_error});
    return false;
  });

  $("#add_button").click(function() {
    $("#data").accordion("destroy").append(
        add_input(["Organization", "Username", "Password", "Notes"]));
    create_accordion($("#data > div:last > h3"));
  });

  $("#save_button").click(function() {
    passwords = encrypt(create_csv($("#data").find(".org,textarea")));
    save_local(passwords, new Date());
    save(passwords);
  });

  $(".options_button").click(function() {
    $("#edit").hide();
    $("#gen_pw").hide();
    $("#import_csv").hide();
    $("#export_csv").hide();
    $("#options").show();
  });

  $("#edit_button").click(function() {
    $("#options").hide();
    $("#edit").show();
  });

  $("#gen_pw_button").click(function() {
    $("#gen_pw_data").remove();
    $("#gen_pw").prepend("<div id='gen_pw_data'>" + generate_password(10) +
                         "</div>");
    $("#options").hide();
    $("#gen_pw").show();
  });

  $("#import_csv_button").click(function() {
    $("#options").hide();
    $("#import_csv").show();
  });

  $("#import_button").click(function() {
    data = $("#import_csv > form > textarea").val();
    if (!data) {
      alert("CSV is empty.");
      return;
    }
    $("#import_csv").hide();
    $("#edit").show();
    editor(data);
  });

  $("#export_csv_button").click(function() {
    input = $("#data").find(".org,textarea");
    $("#export_csv_data").remove();
    $("#export_csv").prepend("<div id='export_csv_data'>" +
                             create_csv(input).replace(/\n/g, "<br>") +
                             "</div>");
    $("#options").hide();
    $("#export_csv").show();
  });

  $("#delete_account_button").click(function() {
    var sure = confirm("Are you sure you want to delete your account?");
    if (sure) {
      var really_sure = confirm(
          "Are you really sure you want to delete your account? This action " +
          "cannot be undone!");
      if (really_sure) {
        localStorage.clear();
        // TODO(ariw): Alert if unable to delete for some reason.
        $.ajax({type: "POST", url: "script/deleteaccount",
                success: function() { location.reload(true); }});
      }
    }
  });
});
