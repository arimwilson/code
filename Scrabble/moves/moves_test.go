package moves_test

import ("testing";
        "../moves")

func TestMoveHash(t *testing.T) {
  a := moves.Move{Word: "test", Start: moves.Location{1, 1},
                  Direction: moves.ACROSS}
  b := a.Copy()

  if a.Hash() != b.Hash() { t.Fail() }
  a.Word = "test2"
  if a.Hash() == b.Hash() { t.Fail() }
  a.Word = b.Word
  a.Direction = moves.DOWN
  if a.Hash() == b.Hash() { t.Fail() }
  a.Direction = b.Direction
  a.Start.X = 2
  if a.Hash() == b.Hash() { t.Fail() }
}
