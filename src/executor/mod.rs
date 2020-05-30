use std::collections::{hash_map::Entry as HmEntry, HashMap};
use std::fmt::Debug;
use std::fs::File;
use std::hash::Hash;
use std::io::{BufWriter, Write};

use anyhow::Error;
use log::*;

use crate::{
    compiler::{compile, CodeBlock, CompileOptions, ExternalBus, OneoffTable},
    Args,
};

struct CacheEntry<T> {
    version: u64,
    code: CodeBlock<T>,
}

pub struct ExecutorOptions {
    pub compile_options: CompileOptions,
    pub disassembly_logfile: Option<String>,
}

pub struct Executor<I, T> {
    oneoffs: OneoffTable,
    bus: ExternalBus<T>,
    compile_options: CompileOptions,
    cache: HashMap<I, CacheEntry<T>>,
    logfile: Option<BufWriter<File>>,
}

impl<I, T> Executor<I, T>
where
    I: Copy + Eq + Hash + Debug,
{
    pub fn new(bus: ExternalBus<T>, options: ExecutorOptions) -> Result<Self, Error> {
        let logfile = options
            .disassembly_logfile
            .map(|path| -> Result<BufWriter<File>, Error> {
                Ok(BufWriter::new(File::create(path)?))
            })
            .transpose()?;
        Ok(Executor {
            oneoffs: OneoffTable::generate(&bus, &options.compile_options)?,
            bus,
            compile_options: options.compile_options,
            cache: HashMap::new(),
            logfile,
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
        let options = self.compile_options;
        let logfile = &mut self.logfile;
        let mut create_entry = || -> Result<CacheEntry<T>, Error> {
            let code = compile(base_addr, data, bus, &oneoffs, &options)?;
            let x86_disasm = code.disassemble()?;
            logfile
                .as_mut()
                .map(|f| -> Result<(), Error> {
                    writeln!(
                        f,
                        "Compiled block {:?} at {:#06x?}, version {}",
                        id, base_addr, version
                    )?;
                    for (idx, inst) in code.instructions().iter().enumerate() {
                        let pc = base_addr.wrapping_add(idx as u16);
                        match inst {
                            Ok(i) => writeln!(
                                f,
                                "{:04x}: {:<50}, cycles {:2}/{:8}, encoding: {:02x?}",
                                pc,
                                format!("{:?}", i.cmd),
                                i.cycles,
                                format!("{:?}", i.alt_cycles),
                                i.encoding
                            ),
                            Err(bytes) => writeln!(f, "{:04x}: Incomplete {:02x?}", pc, bytes),
                        }?;
                    }
                    for inst in x86_disasm.iter() {
                        writeln!(f, "{}", inst)?
                    }
                    Ok(f.flush()?)
                })
                .unwrap_or(Ok(()))?;
            Ok(CacheEntry { version, code })
        };
        let entry = self.cache.entry(id);
        match entry {
            HmEntry::Occupied(e) => {
                let e = e.into_mut();
                if e.version != version {
                    debug!(
                        "Recompiling block {:?} at {:#06x?}, version {}",
                        id, base_addr, version
                    );
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

impl ExecutorOptions {
    pub fn new(args: &Args) -> Self {
        ExecutorOptions {
            compile_options: CompileOptions::new(args),
            disassembly_logfile: args.disassembly_logfile.clone(),
        }
    }
}
