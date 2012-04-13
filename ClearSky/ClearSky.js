// Set up a collection to contain idea information. On the server,
// it is backed by a MongoDB collection named "ideas."

Ideas = new Meteor.Collection("ideas");

if (Meteor.is_client) {
  Template.leaderboard.ideas = function () {
    return Ideas.find({}, {sort: {score: -1, idea: 1}});
  };

  Template.leaderboard.selected_idea = function () {
    var idea = Ideas.findOne(Session.get("selected_idea"));
    return idea && idea.idea;
  };

  Template.idea.selected = function () {
    return Session.equals("selected_idea", this._id) ? "selected" : '';
  };

  Template.leaderboard.events = {
    'click input.inc': function () {
      Ideas.update(Session.get("selected_idea"), {$inc: {score: 1}});
    }
  };

  Template.idea.events = {
    'click': function () {
      Session.set("selected_idea", this._id);
    }
  };
}

// On server startup, create some ideas if the database is empty.
if (Meteor.is_server) {
  Meteor.startup(function () {
    if (Ideas.find().count() === 0) {
      var ideas = ["Create ClearSky app"];
      for (var i = 0; i < ideas.length; i++)
        Ideas.insert({idea: ideas[i], score: 0});
    }
  });
}
