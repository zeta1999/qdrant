use std::collections::HashMap;
use std::hash::Hash;
use std::{mem, iter};

use serde::{Deserialize, Serialize};

use crate::index::field_index::{CardinalityEstimation, PrimaryCondition};
use crate::index::field_index::field_index::{FieldIndex, PayloadFieldIndex, PayloadFieldIndexBuilder};
use crate::types::{IntPayloadType, PayloadType, PointOffsetType, FieldCondition};

#[derive(Serialize, Deserialize)]
pub struct PersistedMapIndex<N: Hash + Eq + Clone> {
    map: HashMap<N, Vec<PointOffsetType>>,
}

impl<N: Hash + Eq + Clone> PersistedMapIndex<N> {
    pub fn new() -> PersistedMapIndex<N> {
        PersistedMapIndex {
            map: Default::default(),
        }
    }

    pub fn match_cardinality(&self, value: &N) -> CardinalityEstimation {
        let values_count = match self.map.get(value) {
            None => 0,
            Some(points) => points.len()
        };

        CardinalityEstimation {
            primary_clauses: vec![],
            min: values_count,
            exp: values_count,
            max: values_count,
        }
    }

    fn add_many(&mut self, idx: PointOffsetType, values: &Vec<N>)
    {
        for value in values {
            let vec = match self.map.get_mut(&value) {
                None => {
                    let new_vec = vec![];
                    self.map.insert(value.clone(), new_vec);
                    self.map.get_mut(&value).unwrap()
                }
                Some(vec) => vec
            };
            vec.push(idx);
        }
    }

    fn get_iterator(&self, value: &N) -> Box<dyn Iterator<Item=PointOffsetType> + '_> {
        self.map
            .get(value)
            .map(|ids| Box::new(ids.iter().cloned()) as Box<dyn Iterator<Item=PointOffsetType>>)
            .unwrap_or(Box::new(iter::empty::<PointOffsetType>()))
    }
}

impl PayloadFieldIndex for PersistedMapIndex<String> {
    fn filter(&self, condition: &FieldCondition) -> Option<Box<dyn Iterator<Item=PointOffsetType> + '_>> {
        condition.r#match.as_ref().and_then(|match_condition|
            match_condition.keyword.as_ref().map(|keyword| self.get_iterator(keyword))
        )
    }

    fn estimate_cardinality(&self, condition: &FieldCondition) -> Option<CardinalityEstimation> {
        condition.r#match.as_ref().and_then(|match_condition|
            match_condition.keyword
                .as_ref()
                .map(|keyword| {
                    let mut estimation = self.match_cardinality(keyword);
                    estimation.primary_clauses.push(PrimaryCondition::Condition(condition.clone()));
                    estimation
                })
        )
    }
}

impl PayloadFieldIndex for PersistedMapIndex<IntPayloadType> {
    fn filter(&self, condition: &FieldCondition) -> Option<Box<dyn Iterator<Item=PointOffsetType> + '_>> {
        condition.r#match.as_ref().and_then(|match_condition|
            match_condition.integer.as_ref().map(|int| self.get_iterator(int))
        )
    }

    fn estimate_cardinality(&self, condition: &FieldCondition) -> Option<CardinalityEstimation> {
        condition.r#match.as_ref().and_then(|match_condition|
            match_condition.integer
                .as_ref()
                .map(|number| {
                    let mut estimation = self.match_cardinality(number);
                    estimation.primary_clauses.push(PrimaryCondition::Condition(condition.clone()));
                    estimation
                }))
    }
}

impl PayloadFieldIndexBuilder for PersistedMapIndex<String> {
    fn add(&mut self, id: usize, value: &PayloadType) {
        match value {
            PayloadType::Keyword(keywords) => self.add_many(id, keywords),
            _ => panic!("Unexpected payload type: {:?}", value)
        }
    }

    fn build(&mut self) -> FieldIndex {
        let data = mem::replace(&mut self.map, Default::default());

        FieldIndex::KeywordIndex(PersistedMapIndex {
            map: data,
        })
    }
}

impl PayloadFieldIndexBuilder for PersistedMapIndex<IntPayloadType> {
    fn add(&mut self, id: usize, value: &PayloadType) {
        match value {
            PayloadType::Integer(numbers) => self.add_many(id, numbers),
            _ => panic!("Unexpected payload type: {:?}", value)
        }
    }

    fn build(&mut self) -> FieldIndex {
        let data = mem::replace(&mut self.map, Default::default());

        FieldIndex::IntMapIndex(PersistedMapIndex {
            map: data,
        })
    }
}
