/// Représente la station de base sur la planète
#[derive(Debug)]
pub struct Station {
    pub energy_storage: u32,
    pub minerals_storage: u32,
    pub scientific_data_count: u32,
}

impl Station {
    pub fn new() -> Self {
        Self {
            energy_storage: 0,
            minerals_storage: 0,
            scientific_data_count: 0,
        }
    }

    /// Ajoute de l'énergie au stockage
    pub fn add_energy(&mut self, amount: u32) {
        self.energy_storage += amount;
    }

    /// Ajoute des minéraux au stockage
    pub fn add_minerals(&mut self, amount: u32) {
        self.minerals_storage += amount;
    }

    /// Ajoute des données scientifiques
    pub fn add_scientific_data(&mut self, amount: u32) {
        self.scientific_data_count += amount;
    }
}
