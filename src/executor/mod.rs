use std::collections::HashMap;

use futures::channel::oneshot;

use crate::compiler::CodeBlock;
use crate::devices::PageId;

enum CacheEntry<T> {
    Wip(oneshot::Receiver<CodeBlock<T>>),
    Ready { version: u64, code: CodeBlock<T> },
}

struct CacheEntry<T> {
    version: u64,
    code: CodeBlock<T>,
}

pub struct Executor<T> {
    cache: HashMap<PageId, CacheEntry<T>>,
}

impl<T> Executor<T> {
    pub fn new() -> Self {
        Executor {
            cache: HashMap::new(),
        }
    }

    async fn compile_task

    pub fn compile(&mut self, page: &dyn Page) -> &CodeBlock<T> {

    }
}
