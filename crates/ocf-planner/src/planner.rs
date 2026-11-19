use crate::candidate::generate_candidates;
use crate::cost::{CostModel, HardConstraints};
use crate::plan::CompressionPlan;
use crate::profile::DatasetProfile;

pub struct Planner {
    pub cost_model: CostModel,
}

impl Planner {
    pub fn new() -> Self {
        Self { cost_model: CostModel { weights: Default::default(), constraints: HardConstraints::default() } }
    }

    pub fn with_constraints(constraints: HardConstraints) -> Self {
        Self { cost_model: CostModel { weights: Default::default(), constraints } }
    }

    /// Select best plan among candidates satisfying hard constraints.
    pub fn plan(&self, profile: &DatasetProfile) -> Option<CompressionPlan> {
        let candidates = generate_candidates(profile);
        let mut feasible: Vec<CompressionPlan> = candidates.into_iter()
            .filter(|p| self.cost_model.feasible(p))
            .collect();
        feasible.sort_by(|a, b| {
            self.cost_model.cost(a).partial_cmp(&self.cost_model.cost(b)).unwrap_or(std::cmp::Ordering::Equal)
        });
        feasible.into_iter().next()
    }

    pub fn plan_all_ranked(&self, profile: &DatasetProfile) -> Vec<(CompressionPlan, f64)> {
        let candidates = generate_candidates(profile);
        let mut v: Vec<(CompressionPlan, f64)> = candidates.into_iter()
            .filter(|p| self.cost_model.feasible(p))
            .map(|p| { let c = self.cost_model.cost(&p); (p, c) })
            .collect();
        v.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        v
    }
}
