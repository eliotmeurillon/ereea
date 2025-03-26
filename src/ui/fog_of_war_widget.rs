use ratatui::{
    buffer::Buffer,
    layout::{Rect, Corner},
    style::Color,
    widgets::Widget,
};
use crate::environment::map::{CellVisibility, Map};  // On suppose que vous avez un `Map`, etc.

/// Exemple d’un widget qui gère un "fog of war" avec un cache de couleurs et un compteur de frames.
/// L’idée est inspirée de l’exemple `Colors_RGB` : on prépare à l’avance un "cache" (self.colors),
/// puis on l’utilise dans `render(...)` pour dessiner. On incrémente aussi un frame_count si on veut
/// animer ou faire varier les couleurs d’une frame à l’autre.
pub struct FogOfWarWidget<'a> {
    /// Référence vers la map (on suppose qu’elle contient `visibility`, `width`, `height`, etc.)
    pub map: &'a Map,

    /// Un cache de couleurs, dimensionné à (height x width).
    /// Optionnel : on peut le remplir en fonction de la visibility, ou pour l’animation.
    pub colors: Vec<Vec<Color>>,

    /// Compteur de frames pour l’animation (optionnel).
    pub frame_count: usize,
}

impl<'a> FogOfWarWidget<'a> {
    /// Construit un nouveau FogOfWarWidget, en initialisant (éventuellement) le cache.
    pub fn new(map: &'a Map) -> Self {
        let h = map.config.height;
        let w = map.config.width;
        // Initialiser le cache de couleurs à noir, par exemple
        let mut colors = vec![vec![Color::Black; w]; h];

        // On pourrait, si on veut, déjà remplir `colors` selon la visibilité
        // (ici on laisse par défaut noir, on le fera dans render).
        Self {
            map,
            colors,
            frame_count: 0,
        }
    }

    /// Met à jour le cache de couleurs en fonction de la visibilité et du frame_count.
    /// Par exemple, on peut faire un dégradé pour `Explored`, un noir total pour `Hidden`, etc.
    /// Appelé avant ou pendant le render.
    fn update_colors(&mut self) {
        // On récupère la taille
        let h = self.map.config.height;
        let w = self.map.config.width;

        for y in 0..h {
            for x in 0..w {
                match self.map.visibility[y][x] {
                    CellVisibility::Hidden => {
                        // Noircir => plus la frame_count est élevée, plus c’est sombre, etc.
                        self.colors[y][x] = Color::Rgb(0, 0, 0);
                    }
                    CellVisibility::Explored => {
                        // Un gris variable en fonction de frame_count,
                        // juste pour montrer qu’on pourrait animer.
                        let fade = ((self.frame_count % 100) as u8) + 50; 
                        // clamp fade in [50..150] par ex
                        let c = if fade < 150 { fade } else { 150u8 };
                        self.colors[y][x] = Color::Rgb(c, c, c);
                    }
                    CellVisibility::Visible => {
                        // Visible => on donne un gris clair ou un fade plus lumineux
                        let fade = ((self.frame_count % 100) as u8) + 150;
                        let c = if fade < 255 { fade } else { 255u8 };
                        self.colors[y][x] = Color::Rgb(c, c, c);
                    }
                }
            }
        }
    }
}

/// L’implémentation du trait Widget pour FogOfWarWidget.
/// On s’inspire de `ColorsWidget` du code `Colors_RGB` : on va écrire
/// directement dans le `Buffer`.
impl<'a> Widget for &mut FogOfWarWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // 1) On peut mettre à jour le cache de couleurs ici (ou en amont).
        self.update_colors();

        // 2) On récupère la taille de la zone de dessin
        let width = area.width as usize;
        let height = area.height as usize;

        // 3) Récupération des dims de la carte
        let map_w = self.map.config.width;
        let map_h = self.map.config.height;

        // Pour éviter de dessiner hors de la zone, calculons les min
        let w_min = map_w.min(width);
        let h_min = map_h.min(height);

        // 4) On parcourt l’aire visible pour écrire dans le buffer
        //    Note : On utilise un offset (area.left, area.top)
        for y in 0..h_min {
            for x in 0..w_min {
                let cell_color = self.colors[y][x];

                // On prend le symbole qu’on veut (ex: ' ' ou '█')
                // Si on veut "pixel", on peut faire un demi-bloc et gérer 2 lignes à la fois.
                // Ici on va juste dessiner un bloc plein '█'
                let ch = '█';

                // Index dans le buffer
                let buf_x = area.left() + x as u16; 
                let buf_y = area.top() + y as u16;

                // On met à jour la cellule du buffer
                let cell = buf.get_mut(buf_x, buf_y);
                cell.set_char(ch);
                cell.set_fg(cell_color);
                cell.set_bg(cell_color); // On peut mettre la même color en fg/bg => bloc coloré
            }
        }

        // 5) Optionnel : Incrémenter le compteur de frames pour animer
        self.frame_count += 1;
    }
}
