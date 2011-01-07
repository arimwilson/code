package main

import ("flag"; "fmt"; "os";)

func compare(real_pi *os.File, guessed_pi string) {
  const NBUF = 512;
  var buf [NBUF]byte;
  pos := 0;
  for {
    switch nr, err := real_pi.Read(&buf); true {
      case err != nil || nr == 0:
        return;
      case nr > 0:
        for i := 0; i < nr && pos < len(guessed_pi); i++ {
          if string(&buf)[i] != guessed_pi[pos] {
            fmt.Printf("Wrong on digit %d. You typed %s but it should have "
                       "been %s.", pos + 1, string(guessed_pi[pos]),
                       string(buf[i]));
            return;
          }
          pos++;
        }
    }
    if (pos == len(guessed_pi)) {
        var next_digits string;
        if pos + 5 < len(buf) {
          next_digits = string(buf[pos:pos+5]);
        } else {
          next_digits = string(buf[pos:len(buf)]);
        }
        fmt.Printf("Correct for %d digits of pi. Next %d digits are %s.", pos,
                   len(next_digits), next_digits);
      return;
    }
  }
}


func main() {
  if flag.Parse(); flag.NArg() != 1 {
    fmt.Printf("Error: should have exactly one argument (guess for pi).");
    os.Exit(1)
  }
  pi_file, err := os.Open("pi.txt", os.O_RDONLY, 0);
  if err != nil {
    fmt.Printf("Error: could not open pi.txt.");
    os.Exit(1)
  }
  defer pi_file.Close();
  compare(pi_file, flag.Arg(0));
}

