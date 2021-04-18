use crate::entry::entry_point::OperationResult;
use std::path::{Path, PathBuf};
use std::fs::create_dir_all;
use serde::{Deserialize, Serialize};
use schemars::{JsonSchema};
use crate::index::index::{Index, PayloadIndex};
use crate::types::{SearchParams, Filter, PointOffsetType, Distance, Indexes};
use crate::vector_storage::vector_storage::{ScoredPointOffset, VectorStorage};
use std::sync::Arc;
use atomic_refcell::AtomicRefCell;
use crate::payload_storage::payload_storage::ConditionChecker;
use std::cmp::max;
use itertools::Itertools;
use std::ops::Deref;
use crate::index::hnsw_index::point_scorer::FilteredScorer;
use rand::{thread_rng, Rng};
use rand::prelude::ThreadRng;
use rand::distributions::Uniform;
use crate::index::hnsw_index::config::HnswConfig;


struct HNSWIndex {
    condition_checker: Arc<AtomicRefCell<dyn ConditionChecker>>,
    vector_storage: Arc<AtomicRefCell<dyn VectorStorage>>,
    payload_index: Arc<AtomicRefCell<dyn PayloadIndex>>,
    config: HnswConfig,
    path: PathBuf,
    distance: Distance,
    thread_rng: ThreadRng,
}


impl HNSWIndex {
    fn get_random_layer(&mut self, reverse_size: f64) -> usize {
        let distribution = Uniform::new(0.0, 1.0);
        let sample: f64 = self.thread_rng.sample(distribution);
        let picked_level = sample.ln() * reverse_size;
        return picked_level.round() as usize;
    }

    fn open(
        path: &Path,
        distance: Distance,
        condition_checker: Arc<AtomicRefCell<dyn ConditionChecker>>,
        vector_storage: Arc<AtomicRefCell<dyn VectorStorage>>,
        payload_index: Arc<AtomicRefCell<dyn PayloadIndex>>,
        index_config: Option<Indexes>,
    ) -> OperationResult<Self> {
        create_dir_all(path)?;
        let mut rng = thread_rng();

        let config_path = HnswConfig::get_config_path(path);
        let config = if config_path.exists() {
            HnswConfig::load(&config_path)?
        } else {
            let (m, ef_construct) = match index_config {
                None => match Indexes::default_hnsw() {
                    Indexes::Hnsw { m, ef_construct } => (m, ef_construct),
                    _ => panic!("Mismatch index config"),
                },
                Some(indx) => match indx {
                    Indexes::Hnsw { m, ef_construct } => (m, ef_construct),
                    _ => panic!("Mismatch index config"),
                }
            };
            HnswConfig::new(m, ef_construct)
        };

        Ok(HNSWIndex {
            condition_checker,
            vector_storage,
            payload_index,
            config,
            path: path.to_owned(),
            distance,
            thread_rng: rng,
        })
    }

    fn build_and_save(&mut self) -> OperationResult<()> {
        unimplemented!()
    }

    fn search_with_condition(&self, top: usize, ef: usize, points_scorer: &FilteredScorer) -> Vec<ScoredPointOffset> {
        unimplemented!()
    }

    fn link_point(&mut self, point_id: PointOffsetType, ef: usize, points_scorer: &FilteredScorer) {
        let layer_id = self.get_random_layer(self.config.level_factor);
        unimplemented!()
    }
}


impl Index for HNSWIndex {
    fn search(&self, vector: &Vec<f32>, filter: Option<&Filter>, top: usize, params: Option<&SearchParams>) -> Vec<ScoredPointOffset> {
        let req_ef = match params {
            None => self.config.ef,
            Some(request_params) => match request_params {
                SearchParams::Hnsw { ef } => *ef
            }
        };

        // ef should always be bigger that required top
        let ef = max(req_ef, top);

        let vector_storage = self.vector_storage.borrow();
        let raw_scorer = vector_storage.raw_scorer(vector, &self.distance);
        let condition_checker = self.condition_checker.borrow();

        let points_scorer = FilteredScorer {
            raw_scorer: raw_scorer.as_ref(),
            condition_checker: condition_checker.deref(),
            filter,
        };

        self.search_with_condition(top, ef, &points_scorer)
    }

    fn build_index(&mut self) -> OperationResult<()> {
        unimplemented!()
    }
}