-- Dumps a TimeZoneSeries as read by timezone-olson, to validate the Rust TZif reader.
import Data.Time.LocalTime
import Data.Time.LocalTime.TimeZone.Olson
import Data.Time.LocalTime.TimeZone.Series
import Data.Time.Clock
import Data.Time.Calendar.Julian ()
import Data.Time.Calendar
import System.Environment
tz (TimeZone m s n) = show m ++ "," ++ show s ++ "," ++ n
main = do
  [f] <- getArgs
  s <- getTimeZoneSeriesFromOlsonFile f
  putStrLn ("default " ++ tz (tzsTimeZone s))
  putStrLn ("n " ++ show (length (tzsTransitions s)))
  mapM_ (\(UTCTime d t, z) -> putStrLn (show (toModifiedJulianDay d) ++ " " ++ show (diffTimeToPicoseconds t) ++ " " ++ tz z)) (tzsTransitions s)
