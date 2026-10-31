//! Shared JSON shapes for dimensions that resolve to a value or an (open) interval
//! (AmountOfMoney, Distance, Quantity, Temperature, Volume).

use crate::json::Json;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    Above,
    Under,
}

/// `SimpleValue`: the single value's object plus `"type": "value"`.
pub fn simple(single: Json) -> Json {
    match single {
        Json::Obj(mut m) => {
            m.insert("type".to_string(), Json::str("value"));
            Json::Obj(m)
        }
        _ => Json::Obj(Default::default()),
    }
}

/// `IntervalValue (from, to)`.
pub fn between(from: Json, to: Json) -> Json {
    Json::obj([("type", Json::str("interval")), ("from", from), ("to", to)])
}

/// `OpenIntervalValue (v, Above | Under)`.
pub fn open(direction: Direction, v: Json) -> Json {
    match direction {
        Direction::Above => Json::obj([("type", Json::str("interval")), ("from", v)]),
        Direction::Under => Json::obj([("type", Json::str("interval")), ("to", v)]),
    }
}
