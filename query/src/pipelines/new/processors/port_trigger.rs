use std::cell::UnsafeCell;
use std::collections::VecDeque;
use std::sync::Arc;

use petgraph::prelude::EdgeIndex;
use petgraph::prelude::NodeIndex;
use petgraph::prelude::StableGraph;

pub struct UpdateList {
    inner: UnsafeCell<UpdateListMutable>,
}

unsafe impl Send for UpdateList {}

unsafe impl Sync for UpdateList {}

pub enum DirectedEdge {
    Source(EdgeIndex),
    Target(EdgeIndex),
}

impl DirectedEdge {
    pub fn get_source<N, E>(&self, graph: &StableGraph<N, E>) -> NodeIndex {
        match self {
            DirectedEdge::Source(edge_index) => graph.edge_endpoints(*edge_index).unwrap().0,
            DirectedEdge::Target(edge_index) => graph.edge_endpoints(*edge_index).unwrap().1,
        }
    }

    pub fn get_target<N, E>(&self, graph: &StableGraph<N, E>) -> NodeIndex {
        match self {
            DirectedEdge::Source(edge_index) => graph.edge_endpoints(*edge_index).unwrap().1,
            DirectedEdge::Target(edge_index) => graph.edge_endpoints(*edge_index).unwrap().0,
        }
    }
}

struct UpdateListMutable {
    updated_edges: Vec<DirectedEdge>,
    updated_triggers: Vec<UnsafeCell<UpdateTrigger>>,
}

impl UpdateList {
    pub fn create() -> Arc<UpdateList> {
        Arc::new(UpdateList {
            inner: UnsafeCell::new(UpdateListMutable {
                updated_edges: vec![],
                updated_triggers: vec![],
            }),
        })
    }

    /// Trigger node input or output edge. Executor will schedule this edge.
    ///
    /// # Safety
    ///
    /// Must be thread safe call. In other words, it needs to be called in single thread or in mutex guard.
    #[inline(always)]
    pub unsafe fn update_edge(&self, edge: DirectedEdge) {
        let inner = &mut *self.inner.get();
        inner.updated_edges.push(edge);
    }

    /// Enter the next scheduling cycle
    ///
    /// # Safety
    ///
    /// Must be thread safe call. In other words, it needs to be called in single thread or in mutex guard.
    pub unsafe fn trigger(&self, queue: &mut VecDeque<DirectedEdge>) {
        let inner = &mut *self.inner.get();

        for trigger in &inner.updated_triggers {
            UpdateTrigger::trigger_version(trigger.get());
        }

        while let Some(index) = inner.updated_edges.pop() {
            queue.push_front(index);
        }
    }

    /// Create schedule trigger for the port
    ///
    /// # Safety
    ///
    /// Must be thread safe call. In other words, it needs to be called in single thread or in mutex guard.
    pub unsafe fn create_trigger(self: &Arc<Self>, edge_index: EdgeIndex) -> *mut UpdateTrigger {
        let inner = &mut *self.inner.get();
        let update_trigger = UpdateTrigger::create(edge_index, self.clone());
        inner.updated_triggers.push(UnsafeCell::new(update_trigger));
        inner.updated_triggers.last().unwrap().get()
    }
}

pub struct UpdateTrigger {
    index: EdgeIndex,
    update_list: Arc<UpdateList>,
    version: usize,
    prev_version: usize,
}

unsafe impl Send for UpdateTrigger {}

impl UpdateTrigger {
    pub fn create(index: EdgeIndex, update_list: Arc<UpdateList>) -> UpdateTrigger {
        UpdateTrigger {
            index,
            update_list,
            version: 0,
            prev_version: 0,
        }
    }

    /// Enter the next scheduling cycle
    ///
    /// # Safety
    ///
    /// *mut UpdateTrigger must be a safe pointer
    pub unsafe fn trigger_version(self_: *mut UpdateTrigger) {
        (*self_).prev_version = (*self_).version;
    }

    /// Trigger node input edge. Executor will schedule this edge.
    ///
    /// # Safety
    ///
    /// *mut UpdateTrigger must be a safe pointer
    #[inline(always)]
    pub unsafe fn update_input(self_: &*mut UpdateTrigger) {
        if !self_.is_null() {
            let self_ = &mut **self_;
            if self_.version == self_.prev_version {
                self_.version += 1;
                self_
                    .update_list
                    .update_edge(DirectedEdge::Target(self_.index));
            }
        }
    }

    /// Trigger node output edge. Executor will schedule this edge.
    ///
    /// # Safety
    ///
    /// *mut UpdateTrigger must be a safe pointer
    #[inline(always)]
    pub unsafe fn update_output(self_: &*mut UpdateTrigger) {
        if !self_.is_null() {
            let self_ = &mut **self_;
            if self_.version == self_.prev_version {
                self_.version += 1;
                self_
                    .update_list
                    .update_edge(DirectedEdge::Source(self_.index));
            }
        }
    }
}
