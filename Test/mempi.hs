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
    | x == y = memorizeNumber xs ys (digits+1)
    | otherwise =
        printf "Wrong on digit %d. You typed %c but it should have been %c."
               digits y x
memorizeNumber ys [] digits =
    let next_digits = firstN ys 5 in
        printf "Correct for %d digits. Next %d digits are %s." (digits-1)
               (length next_digits) next_digits
memorizeNumber [] _ digits = "Correct for all known digits."

-- Get first n elements from a list.
firstN :: [a] -> Int -> [a]
firstN (x:xs) n
    | n > 0 = (x : (firstN xs (n-1)))
    | otherwise = []
firstN [] n = []
