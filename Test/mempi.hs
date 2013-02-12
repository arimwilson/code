-- Run with runhaskell mempi <guess for pi>

import System.Environment
import System.Exit
import Text.Printf

main = do
    args <- getArgs
    if (length args) /= 1
      then do putStrLn "Error: should have exactly one argument (guess for pi)."
              exitWith (ExitFailure 1)
      else do real_pi <- readFile "pi.txt"
              putStrLn (memorizeNumber (init real_pi) (head args) 1)

memorizeNumber :: String -> String -> Int -> String
memorizeNumber (x:xs) (y:ys) digits
    | x /= y =
        printf "Wrong on digit %d. You typed %c but it should have been %c."
        digits y x
    | otherwise = memorizeNumber xs ys (digits + 1)
memorizeNumber [] _ digits = "Correct for all known digits."
memorizeNumber ys [] digits =
    let (remaining, count) = nextDigits ys 5 in
        printf "Correct for %d digits. Next %d digits are %s." (digits - 1)
               count remaining

nextDigits :: String -> Int -> (String, Int)
nextDigits (x:xs) digits
    | digits == 0 = ([], 0)
    | otherwise = let (y, z) = (nextDigits xs (digits - 1)) in (x : y, z + 1)
nextDigits [] digits = ([], 0)
