use std::{
    fmt::{self, Formatter},
    hash::{BuildHasher, Hash},
    marker::PhantomData,
};

use serde::{
    de::{MapAccess, SeqAccess, Visitor},
    ser::{SerializeMap, SerializeSeq},
    Deserialize, Deserializer, Serialize, Serializer,
};

use crate::{LinkedHashMap, LinkedHashSet};

// LinkedHashMap impls

impl<K, V, S> Serialize for LinkedHashMap<K, V, S>
where
    K: Serialize + Eq + Hash,
    V: Serialize,
    S: BuildHasher,
{
    #[inline]
    fn serialize<T: Serializer>(&self, serializer: T) -> Result<T::Ok, T::Error> {
        let mut map_serializer = serializer.serialize_map(Some(self.len()))?;
        for (k, v) in self {
            map_serializer.serialize_key(k)?;
            map_serializer.serialize_value(v)?;
        }
        map_serializer.end()
    }
}

impl<'de, K, V, S> Deserialize<'de> for LinkedHashMap<K, V, S>
where
    K: Deserialize<'de> + Eq + Hash,
    V: Deserialize<'de>,
    S: BuildHasher + Default,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Debug)]
        pub struct LinkedHashMapVisitor<K, V, S> {
            marker: PhantomData<LinkedHashMap<K, V, S>>,
        }

        impl<K, V, S> LinkedHashMapVisitor<K, V, S> {
            fn new() -> Self {
                LinkedHashMapVisitor {
                    marker: PhantomData,
                }
            }
        }

        impl<K, V, S> Default for LinkedHashMapVisitor<K, V, S> {
            fn default() -> Self {
                Self::new()
            }
        }

        impl<'de, K, V, S> Visitor<'de> for LinkedHashMapVisitor<K, V, S>
        where
            K: Deserialize<'de> + Eq + Hash,
            V: Deserialize<'de>,
            S: BuildHasher + Default,
        {
            type Value = LinkedHashMap<K, V, S>;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                write!(formatter, "a map")
            }

            #[inline]
            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
                let mut values = LinkedHashMap::with_capacity_and_hasher(
                    map.size_hint().unwrap_or(0),
                    S::default(),
                );

                while let Some((k, v)) = map.next_entry()? {
                    values.insert(k, v);
                }

                Ok(values)
            }
        }

        deserializer.deserialize_map(LinkedHashMapVisitor::default())
    }
}

// LinkedHashSet impls

impl<T, S> Serialize for LinkedHashSet<T, S>
where
    T: Serialize + Eq + Hash,
    S: BuildHasher,
{
    #[inline]
    fn serialize<U: Serializer>(&self, serializer: U) -> Result<U::Ok, U::Error> {
        let mut seq_serializer = serializer.serialize_seq(Some(self.len()))?;
        for v in self {
            seq_serializer.serialize_element(v)?;
        }
        seq_serializer.end()
    }
}

#[derive(Debug)]
pub struct LinkedHashSetVisitor<T> {
    marker: PhantomData<LinkedHashSet<T>>,
}

impl<T> LinkedHashSetVisitor<T> {
    fn new() -> Self {
        LinkedHashSetVisitor {
            marker: PhantomData,
        }
    }
}

impl<T> Default for LinkedHashSetVisitor<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'de, T> Visitor<'de> for LinkedHashSetVisitor<T>
where
    T: Deserialize<'de> + Eq + Hash,
{
    type Value = LinkedHashSet<T>;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "a sequence")
    }

    #[inline]
    fn visit_seq<S: SeqAccess<'de>>(self, mut seq: S) -> Result<Self::Value, S::Error> {
        let mut values = LinkedHashSet::with_capacity(seq.size_hint().unwrap_or(0));

        while let Some(v) = seq.next_element()? {
            values.insert(v);
        }

        Ok(values)
    }
}

impl<'de, T> Deserialize<'de> for LinkedHashSet<T>
where
    T: Deserialize<'de> + Eq + Hash,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_seq(LinkedHashSetVisitor::default())
    }
}
