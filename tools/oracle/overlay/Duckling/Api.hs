-- Copyright (c) 2016-present, Facebook, Inc.
-- All rights reserved.
--
-- This source code is licensed under the BSD-style license found in the
-- LICENSE file in the root directory of this source tree.


{-# LANGUAGE NoRebindableSyntax #-}
{-# LANGUAGE RecordWildCards #-}
{-# LANGUAGE OverloadedStrings #-}

module Duckling.Api
  ( analyze
  , formatToken
  , parse
  , supportedDimensions
  ) where

import Data.Aeson (Value, object, toJSON, encode, (.=))
import qualified Data.ByteString.Lazy.Char8 as LBS8
import Data.List (sort)
import System.Environment (lookupEnv)
import System.IO
import Control.Concurrent.MVar
import qualified Data.ByteString.Lazy as LBS
import System.IO.Unsafe (unsafePerformIO)
import qualified Data.Time as Time
import qualified Data.Time.Calendar as Cal
import qualified Data.Time.LocalTime.TimeZone.Series as Series
import Data.HashMap.Strict (HashMap)
import Data.HashSet (HashSet)
import Data.Text (Text)
import Prelude
import qualified Data.HashMap.Strict as HashMap
import qualified Data.HashSet as HashSet
import qualified Data.Text as Text

import Duckling.Dimensions.Types
import Duckling.Dimensions
import Duckling.Engine
import Duckling.Locale
import Duckling.Ranking.Classifiers
import Duckling.Ranking.Rank
import Duckling.Resolve
import Duckling.Rules
import Duckling.Types

-- | Parses `input` and returns a curated list of entities found.
parse
  :: Text
  -> Context
  -> Options
  -> [Seal Dimension]
  -> [Entity]
parse input ctx options =
  map (formatToken input)
  . analyze input ctx options
  . HashSet.fromList

supportedDimensions :: HashMap Lang [Seal Dimension]
supportedDimensions =
  HashMap.fromList [ (l, allDimensions l) | l <- [minBound..maxBound] ]

-- | Returns a curated list of resolved tokens found
-- When `targets` is non-empty, returns only tokens of such dimensions.
analyze
  :: Text
  -> Context
  -> Options
  -> HashSet (Seal Dimension)
  -> [ResolvedToken]
analyze input context options targets =
  dumpAnalyze input context options targets (analyzeReal input context options targets)

{-# NOINLINE dumpHandle #-}
dumpHandle :: Maybe (MVar Handle)
dumpHandle = unsafePerformIO $ do
  mpath <- lookupEnv "DUCKLING_DUMP"
  case mpath of
    Nothing -> return Nothing
    Just path -> do
      h <- openFile path AppendMode
      hSetBuffering h (BlockBuffering Nothing)
      hSetEncoding h utf8
      Just <$> newMVar h

{-# NOINLINE dumpAnalyze #-}
dumpAnalyze
  :: Text -> Context -> Options -> HashSet (Seal Dimension)
  -> [ResolvedToken] -> [ResolvedToken]
dumpAnalyze input context options targets result = case dumpHandle of
  Nothing -> result
  Just mv ->
    let rec = encodeRecord input context options targets result
    in LBS8.length rec `seq` unsafePerformIO (do
         withMVar mv $ \h -> do
           LBS.hPut h rec
           LBS.hPut h "\n"
           hFlush h
         return result)

utcJson :: Time.UTCTime -> Value
utcJson (Time.UTCTime day dt) = object
  [ "mjd" .= Cal.toModifiedJulianDay day
  , "psecs" .= (truncate (toRational dt * 1000000000000) :: Integer)
  ]

zoneJson :: Time.TimeZone -> Value
zoneJson (Time.TimeZone mins summer name) = object
  [ "offset_min" .= mins, "summer" .= summer, "name" .= name ]

encodeRecord
  :: Text -> Context -> Options -> HashSet (Seal Dimension) -> [ResolvedToken]
  -> LBS8.ByteString
encodeRecord input Context{referenceTime = DucklingTime (Series.ZoneSeriesTime refUtc series), locale = Locale lang mreg} Options{..} targets result =
  encode $ object
    [ "lang" .= show lang
    , "region" .= fmap show mreg
    , "ref_utc" .= utcJson refUtc
    , "tz_default" .= zoneJson (Series.tzsTimeZone series)
    , "tz_transitions" .=
        [ object [ "at" .= utcJson t, "zone" .= zoneJson z ]
        | (t, z) <- Series.tzsTransitions series ]
    , "latent" .= withLatent
    , "targets" .= sort [ toName d | Seal d <- HashSet.toList targets ]
    , "input" .= input
    , "entities" .= map (toJSON . formatToken input) result
    ]

analyzeReal
  :: Text
  -> Context
  -> Options
  -> HashSet (Seal Dimension)
  -> [ResolvedToken]
analyzeReal input context@Context{..} options targets =
  rank (classifiers locale) targets
  $ filter isRelevantDimension
  $ parseAndResolve (rulesFor locale targets) input context options
  where
    isRelevantDimension Resolved{node = Node{token = (Token d _)}} =
      HashSet.null targets || HashSet.member (Seal d) targets

-- | Converts the resolved token to the API format
formatToken :: Text -> ResolvedToken -> Entity
formatToken sentence
  (Resolved (Range start end) node@Node{token = Token dim _} value latent)
  = Entity (toName dim) body value start end latent node
  where
    body = Text.drop start $ Text.take end sentence
