#[derive(Debug)]
pub struct CleanStats {
    pub total: usize,
    pub too_short: usize,
    pub too_long: usize,
    pub kept: usize,
    pub low_tier: usize,    // Ajout pour suivre les ressources de palier bas
    pub medium_tier: usize, // Ajout pour suivre les ressources de palier moyen
    pub high_tier: usize,   // Ajout pour suivre les ressources de palier élevé
}

impl CleanStats {
    pub fn new() -> Self {
        Self {
            total: 0,
            too_short: 0,
            too_long: 0,
            kept: 0,
            low_tier: 0,
            medium_tier: 0,
            high_tier: 0,
        }
    }

    pub fn add_kept(&mut self, tier: &str) {
        self.total += 1;
        self.kept += 1;
        match tier {
            "low" => self.low_tier += 1,
            "medium" => self.medium_tier += 1,
            "high" => self.high_tier += 1,
            _ => (),
        }
    }

    pub fn add_too_short(&mut self) {
        self.total += 1;
        self.too_short += 1;
    }

    pub fn add_too_long(&mut self) {
        self.total += 1;
        self.too_long += 1;
    }
}

impl std::fmt::Display for CleanStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Total: {}, Kept: {}, Dropped: {} (too short: {}, too long: {}), Tiers - Low: {}, Medium: {}, High: {}",
            self.total,
            self.kept,
            self.too_short + self.too_long,
            self.too_short,
            self.too_long,
            self.low_tier,
            self.medium_tier,
            self.high_tier
        )
    }
}

impl Default for CleanStats {
    fn default() -> Self {
        Self::new()
    }
}
