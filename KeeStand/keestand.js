// Global love.
var username_ = "";
var password_ = "";
var salt_ = "";
var password_hash_ = "";
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

// Version 2- passwords used an older SJCL library. We silently convert them on
// save.
function decrypt(data, version) {
  assert(password_, "password_ is null.");
  if (version <= 2) {
    return old_sjcl.decrypt(password_, data);
  } else if (version == 3) {
    return sjcl.decrypt(password_, data);
  }
}

// Version 1 serialized form:
// encrypt(escape(organization1) + "," + escape(username1) + "," + ... + "\n" +
// ...)
// Version 2+ serialized form:
// encrypt(arrayToCsv([[organization1, username1, ...], ...]))
//
// We silently convert version 1 passwords to version 2+ passwords when saving
// them.
function deserialize(data, version) {
  if (version == 1) {
    data = data.split("\n").slice(0, -1);
    for (i = 0; i < data.length; ++i) {
      data[i] = data[i].split(",");
      for (j = 0; j < data[i].length; ++j) {
        data[i][j] = unescape(data[i][j]);
      }
    }
    return data;
  } else if (version >= 2) {
    return CSV.csvToArray(data);
  }
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
  for (i = 0; i < data.length; ++i) {
    $("#data").append(add_input(data[i]));
  }
  create_accordion(false);
}

function password_hash(password, salt) {
  var hash = sjcl.hash.sha256.hash(password + salt);
  // TODO(ariw): Is 1000 right here? I wish I could use bcrypt...
  for (i = 1; i < 1000; ++i)
    hash = sjcl.hash.sha256.hash(hash);
  return sjcl.codec.base64.fromBits(hash, false);
}

function salt_success(data) {
  salt_ = data;
  password_hash_ = password_hash(password_, salt_);
  $.ajax({type: "POST", url: "script/login",
          data: { username: username_, password_hash: password_hash_,
                 salt: salt_ },
          success: login_successful, error: login_error, dataType: "json"});
}

function salt_error() {
  if (local_storage_) {
    salt_ = localStorage["salt"];
    password_hash_ = password_hash(password_, salt_);
    var actual_password_hash = localStorage["password_hash"];
    if (password_hash_ != actual_password_hash) {
      login_error();
      return;
    }
    $("#login").hide();
    $("#edit").show();
    var version = parseInt(localStorage["version"])
    editor(deserialize(decrypt(localStorage["passwords"], version), version));
    save(localStorage["passwords"]);
  }
}

function login_successful(data) {
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
    var version = data["version"];
    editor(deserialize(decrypt(data["passwords"], version), version));
    save_local(data["passwords"], last_modified);
  } else if (local_storage_) {
    var version = parseInt(localStorage["version"])
    editor(deserialize(decrypt(localStorage["passwords"], version), version));
    save(localStorage["passwords"]);
  }
}

function login_error() {
  $("#login_error").text("Password incorrect or username already taken.");
}

function encrypt(data) {
  assert(password_, "password_ is null.");
  return sjcl.encrypt(password_, data);
}

function serialize(input) {
  data = [];
  for (i = 0; i < input.length; i += 4) {
    data.push([input[i].innerHTML, input[i + 1].value, input[i + 2].value,
               input[i + 3].value]);
  }
  return CSV.arrayToCsv(data);
}

function save_local(passwords, last_modified) {
  assert(username_ && salt_ && password_hash_,
         "username_ or salt_ or password_hash_ is null.");
  localStorage["username"] = username_;
  localStorage["salt"] = salt_;
  localStorage["password_hash"] = password_hash_;
  localStorage["passwords"] = passwords;
  localStorage["last_modified"] = last_modified.toUTCString();
  localStorage["version"] = 2;
}

function save(passwords) {
  // TODO(ariw): Notify when successful.
  $.post("script/save", { passwords: passwords, version: 3 });
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
  // Display last used username and focus on input boxes, as a convenience.
  if (localStorage.getItem("username")) {
    $("#username").val(localStorage["username"]);
    $("#password").focus();
  } else {
    $("#username").focus();
  }

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
    passwords = encrypt(serialize($("#data").find(".org,textarea")));
    save_local(passwords, new Date());
    save(passwords);
  });

  $(".options_button").click(function() {
    $("#edit").hide();
    $("#gen_pw").hide();
    $("#import_csv").hide();
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

  $("#import_csv_file").change(function(event) {
    file = event.target.files[0];
    reader = new FileReader();
    reader.onload = function(event) {
      data = event.target.result;
      if (!data) {
        alert("CSV is empty.");
        return;
      }
      $("#import_csv").hide();
      $("#edit").show();
      editor(deserialize(data, 2));
    }
    reader.readAsText(file);
  });

  $("#export_csv_file").click(function() {
    passwords = serialize($("#data").find(".org,textarea"));
    location.href = "data:text/csv;base64," +
                    btoa(unescape(encodeURIComponent(passwords)));
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
