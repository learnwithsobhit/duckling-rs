-- Copyright (c) 2016-present, Facebook, Inc. (original DucklingExpensive cases)
-- Timed analyze-only replay of exe/DucklingExpensive.hs (no parse-tree I/O).
--
-- Built and run by tools/oracle/bench_expensive.sh inside duckling-oracle-base.

{-# LANGUAGE OverloadedStrings #-}
{-# LANGUAGE ScopedTypeVariables #-}
{-# OPTIONS -fno-full-laziness #-}

module Main (main) where

import Control.Monad (void, replicateM_)
import Data.Text (Text)
import System.Environment (getArgs)
import qualified Data.HashSet as HashSet

import Duckling.Api (analyze)
import Duckling.Dimensions.Types
import Duckling.Locale (Lang(..), makeLocale)
import Duckling.Testing.Types (testContext, testOptions)

enText :: Text
enText = "Monday 3rd at 9.30am or 2.30pm, Saturday 8th at 10.30am, Tuesday 11th at 2pm, Wednesday 12th at 2.30pm, Friday 14th at 12.30pm xx"

esText :: Text
esText = "Horario es de Lunes a Viernes de 2 pm a 10 pm. S\195\161bado de 9 am a 7 pm"

main :: IO ()
main = do
  args <- getArgs
  let n :: Int = case args of
        (x:_) -> read x
        [] -> 20
      en = makeLocale EN Nothing
      es = makeLocale ES Nothing
      targets = HashSet.singleton (Seal Time)
      once = do
        void $ analyze enText (testContext {locale = en}) testOptions targets
        void $ analyze esText (testContext {locale = es}) testOptions targets
  -- Warm the rule tables the same way Criterion does.
  once
  replicateM_ n once
