use std::collections::{hash_map::Entry as HmEntry, HashMap};
use std::hash::Hash;

use anyhow::Error;

use crate::compiler::{compile, CodeBlock, CompileError, CompileOptions, ExternalBus, OneoffTable};

struct CacheEntry<T> {
    version: u64,
    code: CodeBlock<T>,
}

pub struct Executor<I, T> {
    oneoffs: OneoffTable,
    bus: ExternalBus<T>,
    options: CompileOptions,
    cache: HashMap<I, CacheEntry<T>>,
}

impl<I, T> Executor<I, T>
where
    I: Eq + Hash,
{
    pub fn new(bus: ExternalBus<T>, options: CompileOptions) -> Result<Self, Error> {
        Ok(Executor {
            oneoffs: OneoffTable::generate(&bus, &options)?,
            bus,
            options,
            cache: HashMap::new(),
        })
    }

    pub fn compile(
        &mut self,
        id: I,
        version: u64,
        base_addr: u16,
        data: &[u8],
    ) -> Result<&CodeBlock<T>, Error> {
        let bus = self.bus;
        let oneoffs = &self.oneoffs;
        let options = self.options;
        let create_entry = || -> Result<CacheEntry<T>, CompileError> {
            let code = compile(base_addr, data, bus, &oneoffs, &options)?;
            Ok(CacheEntry { version, code })
        };
        let entry = self.cache.entry(id);
        match entry {
            HmEntry::Occupied(e) => {
                let e = e.into_mut();
                if e.version != version {
                    *e = create_entry()?;
                }
                Ok(&e.code)
            }
            HmEntry::Vacant(v) => {
                let e = v.insert(create_entry()?);
                Ok(&e.code)
            }
        }
    }
}
