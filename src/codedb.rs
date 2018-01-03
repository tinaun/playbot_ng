use std::path::Path;
use std::fs::{OpenOptions,File};
use std::io::{Seek,SeekFrom};
use std::collections::HashMap;
use failure::{Error, ResultExt};
use std::io::Write;
use json;
use chrono::prelude::*;
use std::borrow::Cow;

pub struct CodeDB {
    file: File,
    cache: HashMap<String, Value<'static>>,
}

#[derive(Serialize,Deserialize,Clone)]
pub struct Entry<'a> {
    key: Cow<'a, str>,
    value: Value<'a>,
    modified_by: Cow<'a, str>,
    date: DateTime<Utc>,
}

impl<'a> Entry<'a> {
    fn new<K, M>(key: K, value: Value<'a>, modified_by: M) -> Self
    where
        K: Into<Cow<'a, str>>,
        M: Into<Cow<'a, str>>,
    {
        Self {
            key: key.into(),
            value,
            modified_by: modified_by.into(),
            date: Utc::now(),
        }
    }
}

#[derive(Serialize,Deserialize,Clone)]
pub enum Value<'a> {
    Function(Cow<'a, str>),
    Locked,
    Deleted,
}

impl CodeDB {
    pub fn open_or_create<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let path = path.as_ref();
        let mut file = OpenOptions::new().read(true).create(true).append(true).open(path)
            .with_context(|_| format!("Failed to open code file {}", path.display()))?;

        file.seek(SeekFrom::Current(0))?;
        
        let mut cache = HashMap::new();
        let stream = json::Deserializer::from_reader(&mut file).into_iter();
        
        for entry in stream {
            let entry: Entry = entry?;
            cache.insert(entry.key.into_owned(), entry.value);
        }

        file.seek(SeekFrom::End(0))?;

        Ok(Self {
            file,
            cache,
        })
    }

    pub fn lookup_fn(&self, key: &str) -> Option<&str> {
        self.cache.get(key).and_then(|value| match *value {
            Value::Function(ref fun) => Some(fun.as_ref()),
            _ => None
        })
    }

    pub fn insert_fn<K, V>(&mut self, key: K, fun: V, modified_by: &str) -> Result<(), Error>
    where
        K: Into<String>,
        V: Into<String>,
    {
        let key = key.into();
        let fun = fun.into();
        let old = self.cache.entry(key.clone()).or_insert(Value::Deleted);

        match *old {
            Value::Deleted | Value::Function(..) => {
                let value = Value::Function(fun.into());

                // Write data in a single write call
                // See https://doc.rust-lang.org/nightly/std/fs/struct.OpenOptions.html#method.append
                let entry = Entry::new(key, value.clone(), modified_by);
                let serialized = json::to_vec(&entry)?;
                self.file.write_all(&serialized)?;
                self.file.flush()?;

                *old = value;

                Ok(())
            },
            Value::Locked => Err(format_err!("Entry is locked.")),
        }
    }
}
