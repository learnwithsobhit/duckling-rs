{-# LANGUAGE RecordWildCards #-}
{-# LANGUAGE OverloadedStrings #-}
-- Dumps every Naive Bayes classifier table used by Duckling.Ranking.Classifiers to JSON.
-- Output: <dir>/<LANG>_<REGION|XX>.json
-- Non-finite doubles are encoded as the strings "-inf" / "inf".
module Main (main) where

import Data.Aeson
import qualified Data.Aeson.Key as K
import qualified Data.Aeson.KeyMap as KM
import qualified Data.ByteString.Lazy as LB
import qualified Data.HashMap.Strict as HM
import Data.List (sortOn)
import qualified Data.Text as T
import System.Environment (getArgs)

import Duckling.Locale
import Duckling.Ranking.Classifiers (classifiers)
import Duckling.Ranking.Types

dbl :: Double -> Value
dbl x
  | isInfinite x = String (if x > 0 then "inf" else "-inf")
  | isNaN x = String "nan"
  | otherwise = toJSON x

classData :: ClassData -> Value
classData ClassData{..} = object
  [ "prior" .= dbl prior
  , "unseen" .= dbl unseen
  , "n" .= n
  , "likelihoods" .= Object (KM.fromList
      [ (K.fromText f, dbl v) | (f, v) <- sortOn fst (HM.toList likelihoods) ])
  ]

classifier :: Classifier -> Value
classifier (Classifier ok ko) = object [ "ok" .= classData ok, "ko" .= classData ko ]

dump :: FilePath -> Locale -> IO ()
dump dir loc = do
  let cs = classifiers loc
      obj = Object $ KM.fromList
        [ (K.fromText r, classifier c) | (r, c) <- sortOn fst (HM.toList cs) ]
  LB.writeFile (dir ++ "/" ++ show loc ++ ".json") (encode obj)
  putStrLn $ show loc ++ ": " ++ show (HM.size cs) ++ " rules"

main :: IO ()
main = do
  [dir] <- getArgs
  let locs = [ Locale l Nothing | l <- [minBound .. maxBound] ]
          ++ [ Locale EN (Just GB), Locale EN (Just US) ]
  mapM_ (dump dir) locs
