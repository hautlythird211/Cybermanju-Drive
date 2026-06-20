use cybermanju_types::schema::FileNode;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BackendSelector {
    Local(String),
    GitHub(String),
    GitLab(String),
    Codeberg(String),
    Gitea(String),
    GoogleDrive(String),
    GooglePhotos(String),
    Telegram(String),
    Mega(String),
    Custom(String),
}

impl BackendSelector {
    pub fn backend_type(&self) -> &str {
        match self {
            BackendSelector::Local(_) => "local",
            BackendSelector::GitHub(_) => "github",
            BackendSelector::GitLab(_) => "gitlab",
            BackendSelector::Codeberg(_) => "codeberg",
            BackendSelector::Gitea(_) => "gitea",
            BackendSelector::GoogleDrive(_) => "gdrive",
            BackendSelector::GooglePhotos(_) => "gphotos",
            BackendSelector::Telegram(_) => "telegram",
            BackendSelector::Mega(_) => "mega",
            BackendSelector::Custom(_) => "custom",
        }
    }

    pub fn label(&self) -> &str {
        match self {
            BackendSelector::Local(l) => l,
            BackendSelector::GitHub(l) => l,
            BackendSelector::GitLab(l) => l,
            BackendSelector::Codeberg(l) => l,
            BackendSelector::Gitea(l) => l,
            BackendSelector::GoogleDrive(l) => l,
            BackendSelector::GooglePhotos(l) => l,
            BackendSelector::Telegram(l) => l,
            BackendSelector::Mega(l) => l,
            BackendSelector::Custom(l) => l,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolutionDistribution {
    pub resolution: String,
    pub backend: BackendSelector,
    pub path: String,
    pub estimated_cost: f64,
}

#[derive(Debug, Clone)]
pub struct CostModel {
    pub cost_per_gb: HashMap<String, f64>,
    pub bandwidth_cost_per_gb: HashMap<String, f64>,
    pub redundancy_factor: HashMap<String, f64>,
}

impl Default for CostModel {
    fn default() -> Self {
        let mut cost_per_gb = HashMap::new();
        cost_per_gb.insert("local".to_string(), 0.0);
        cost_per_gb.insert("github".to_string(), 0.0);
        cost_per_gb.insert("gitlab".to_string(), 0.0);
        cost_per_gb.insert("codeberg".to_string(), 0.0);
        cost_per_gb.insert("gitea".to_string(), 0.0);
        cost_per_gb.insert("gdrive".to_string(), 0.02);
        cost_per_gb.insert("gphotos".to_string(), 0.01);
        cost_per_gb.insert("telegram".to_string(), 0.0);
        cost_per_gb.insert("mega".to_string(), 0.005);
        cost_per_gb.insert("custom".to_string(), 0.01);

        let mut bandwidth_cost_per_gb = HashMap::new();
        bandwidth_cost_per_gb.insert("local".to_string(), 0.0);
        bandwidth_cost_per_gb.insert("github".to_string(), 0.0);
        bandwidth_cost_per_gb.insert("gitlab".to_string(), 0.0);
        bandwidth_cost_per_gb.insert("codeberg".to_string(), 0.0);
        bandwidth_cost_per_gb.insert("gitea".to_string(), 0.0);
        bandwidth_cost_per_gb.insert("gdrive".to_string(), 0.12);
        bandwidth_cost_per_gb.insert("gphotos".to_string(), 0.0);
        bandwidth_cost_per_gb.insert("telegram".to_string(), 0.0);
        bandwidth_cost_per_gb.insert("mega".to_string(), 0.0);
        bandwidth_cost_per_gb.insert("custom".to_string(), 0.05);

        let mut redundancy_factor = HashMap::new();
        redundancy_factor.insert("local".to_string(), 1.0);
        redundancy_factor.insert("github".to_string(), 1.0);
        redundancy_factor.insert("gitlab".to_string(), 1.0);
        redundancy_factor.insert("codeberg".to_string(), 1.0);
        redundancy_factor.insert("gitea".to_string(), 1.0);
        redundancy_factor.insert("gdrive".to_string(), 1.0);
        redundancy_factor.insert("gphotos".to_string(), 1.0);
        redundancy_factor.insert("telegram".to_string(), 1.0);
        redundancy_factor.insert("mega".to_string(), 1.0);
        redundancy_factor.insert("custom".to_string(), 1.0);

        Self {
            cost_per_gb,
            bandwidth_cost_per_gb,
            redundancy_factor,
        }
    }
}

impl CostModel {
    pub fn storage_cost(&self, backend: &BackendSelector, size_bytes: u64) -> f64 {
        let gb = size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        let cost = self.cost_per_gb.get(backend.backend_type()).unwrap_or(&0.01);
        let redundancy = self
            .redundancy_factor
            .get(backend.backend_type())
            .unwrap_or(&1.0);
        gb * cost * redundancy
    }

    pub fn transfer_cost(&self, backend: &BackendSelector, size_bytes: u64) -> f64 {
        let gb = size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        let cost = self
            .bandwidth_cost_per_gb
            .get(backend.backend_type())
            .unwrap_or(&0.05);
        gb * cost
    }

    pub fn total_cost(&self, backend: &BackendSelector, size_bytes: u64) -> f64 {
        self.storage_cost(backend, size_bytes) + self.transfer_cost(backend, size_bytes)
    }

    pub fn find_cheapest<'a>(
        &self,
        backends: &'a [BackendSelector],
        size_bytes: u64,
    ) -> Option<&'a BackendSelector> {
        backends
            .iter()
            .min_by(|a, b| {
                self.total_cost(a, size_bytes)
                    .partial_cmp(&self.total_cost(b, size_bytes))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Resolution {
    Original,
    FullHd,
    Hd,
    Sd,
    Thumbnail,
    Preview,
}

impl Resolution {
    pub fn label(&self) -> &str {
        match self {
            Resolution::Original => "original",
            Resolution::FullHd => "1080p",
            Resolution::Hd => "720p",
            Resolution::Sd => "480p",
            Resolution::Thumbnail => "thumbnail",
            Resolution::Preview => "preview",
        }
    }

    pub fn estimated_size_bytes(&self, original_size: u64) -> u64 {
        match self {
            Resolution::Original => original_size,
            Resolution::FullHd => (original_size as f64 * 0.5) as u64,
            Resolution::Hd => (original_size as f64 * 0.25) as u64,
            Resolution::Sd => (original_size as f64 * 0.1) as u64,
            Resolution::Thumbnail => (original_size as f64 * 0.01).max(5_000.0) as u64,
            Resolution::Preview => (original_size as f64 * 0.05).max(10_000.0) as u64,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DistributionPlanEntry {
    pub file_id: String,
    pub resolution: Resolution,
    pub backend: BackendSelector,
    pub path: String,
    pub estimated_cost: f64,
}

#[derive(Debug, Clone)]
pub struct DistributionPlan {
    pub entries: Vec<DistributionPlanEntry>,
    pub total_estimated_cost: f64,
}

pub struct DistributionPlanner {
    pub backends: Vec<BackendSelector>,
    pub cost_model: CostModel,
    pub preferred_original: Option<BackendSelector>,
    pub preferred_resolutions: HashMap<Resolution, BackendSelector>,
}

impl DistributionPlanner {
    pub fn new(backends: Vec<BackendSelector>) -> Self {
        Self {
            backends,
            cost_model: CostModel::default(),
            preferred_original: None,
            preferred_resolutions: HashMap::new(),
        }
    }

    pub fn with_cost_model(mut self, cost_model: CostModel) -> Self {
        self.cost_model = cost_model;
        self
    }

    pub fn with_preferred_original(mut self, backend: BackendSelector) -> Self {
        self.preferred_original = Some(backend);
        self
    }

    pub fn with_preferred_resolution(
        mut self,
        resolution: Resolution,
        backend: BackendSelector,
    ) -> Self {
        self.preferred_resolutions.insert(resolution, backend);
        self
    }

    pub fn plan_distribution(&self, file: &FileNode) -> DistributionPlan {
        let resolutions = vec![
            Resolution::Original,
            Resolution::FullHd,
            Resolution::Hd,
            Resolution::Sd,
            Resolution::Thumbnail,
            Resolution::Preview,
        ];

        let mut entries = Vec::new();
        let mut total_cost = 0.0;

        for resolution in &resolutions {
            let backend = self.select_backend(resolution, file.size_bytes);
            let est_size = resolution.estimated_size_bytes(file.size_bytes);
            let cost = self.cost_model.total_cost(&backend, est_size);

            let path = format!(
                "{}/{}/{}",
                file.id,
                resolution.label(),
                file.name
            );

            entries.push(DistributionPlanEntry {
                file_id: file.id.clone(),
                resolution: resolution.clone(),
                backend,
                path,
                estimated_cost: cost,
            });

            total_cost += cost;
        }

        DistributionPlan {
            entries,
            total_estimated_cost: total_cost,
        }
    }

    fn select_backend(&self, resolution: &Resolution, size_bytes: u64) -> BackendSelector {
        if let Some(backend) = self.preferred_resolutions.get(resolution) {
            return backend.clone();
        }

        if let Some(ref backend) = self.preferred_original {
            if *resolution == Resolution::Original {
                return backend.clone();
            }
        }

        if self.backends.is_empty() {
            return BackendSelector::Local("default".to_string());
        }

        self.cost_model
            .find_cheapest(&self.backends, size_bytes)
            .cloned()
            .unwrap_or_else(|| BackendSelector::Local("default".to_string()))
    }

    pub fn plan_batch(&self, files: &[FileNode]) -> Vec<DistributionPlan> {
        files.iter().map(|f| self.plan_distribution(f)).collect()
    }

    pub fn total_batch_cost(&self, plans: &[DistributionPlan]) -> f64 {
        plans.iter().map(|p| p.total_estimated_cost).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_type() {
        assert_eq!(
            BackendSelector::GitHub("gh".to_string()).backend_type(),
            "github"
        );
        assert_eq!(
            BackendSelector::Local("disk".to_string()).backend_type(),
            "local"
        );
    }

    #[test]
    fn test_cost_model() {
        let model = CostModel::default();
        let gh = BackendSelector::GitHub("gh".to_string());
        let gdrive = BackendSelector::GoogleDrive("gd".to_string());

        let size_1gb = 1024 * 1024 * 1024;
        assert_eq!(model.storage_cost(&gh, size_1gb), 0.0);
        assert!((model.storage_cost(&gdrive, size_1gb) - 0.02).abs() < 0.001);
    }

    #[test]
    fn test_resolution_sizes() {
        let orig = Resolution::Original.estimated_size_bytes(1_000_000);
        assert_eq!(orig, 1_000_000);

        let thumb = Resolution::Thumbnail.estimated_size_bytes(1_000_000);
        assert!(thumb < orig);
    }

    #[test]
    fn test_distribution_planner() {
        let backends = vec![
            BackendSelector::Local("disk".to_string()),
            BackendSelector::GitHub("gh".to_string()),
            BackendSelector::GoogleDrive("gd".to_string()),
        ];

        let planner = DistributionPlanner::new(backends);
        let file = FileNode {
            id: "test-1".to_string(),
            name: "photo.jpg".to_string(),
            file_type: "image".to_string(),
            parent_id: None,
            size_bytes: 5_000_000,
            mime_type: Some("image/jpeg".to_string()),
            hash_blake3: None,
            encrypted: false,
            encryption_algorithm: None,
            compression_layers: vec![],
            thumbnail_path: None,
            context_data: None,
            tags: vec![],
            collection_ids: vec![],
            face_group_ids: vec![],
            loose_group_ids: vec![],
            gps_lat: None,
            gps_lon: None,
            created_at: String::new(),
            modified_at: String::new(),
        };

        let plan = planner.plan_distribution(&file);
        assert_eq!(plan.entries.len(), 6);
        assert!(plan.total_estimated_cost >= 0.0);
    }
}
