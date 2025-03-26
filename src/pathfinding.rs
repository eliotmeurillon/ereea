use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use crate::environment::Map;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct Node {
    position: (usize, usize),
    f_score: i32,
    g_score: i32,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f_score.cmp(&self.f_score)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn find_path(
    map: &Map,
    start: (usize, usize),
    goal: (usize, usize),
) -> Option<Vec<(usize, usize)>> {
    let mut open_set = BinaryHeap::new();
    let mut came_from = HashMap::new();
    let mut g_scores = HashMap::new();
    let mut f_scores = HashMap::new();

    g_scores.insert(start, 0);
    f_scores.insert(start, manhattan_distance(start, goal));
    open_set.push(Node {
        position: start,
        f_score: f_scores[&start],
        g_score: 0,
    });

    while let Some(current) = open_set.pop() {
        if current.position == goal {
            return Some(reconstruct_path(came_from, current.position));
        }

        for neighbor in get_neighbors(map, current.position) {
            let tentative_g_score = g_scores[&current.position] + 1;

            if !g_scores.contains_key(&neighbor) || tentative_g_score < g_scores[&neighbor] {
                came_from.insert(neighbor, current.position);
                g_scores.insert(neighbor, tentative_g_score);
                let f_score = tentative_g_score + manhattan_distance(neighbor, goal);
                f_scores.insert(neighbor, f_score);

                open_set.push(Node {
                    position: neighbor,
                    f_score,
                    g_score: tentative_g_score,
                });
            }
        }
    }

    None
}

fn manhattan_distance(a: (usize, usize), b: (usize, usize)) -> i32 {
    (a.0.abs_diff(b.0) + a.1.abs_diff(b.1)) as i32
}

fn get_neighbors(map: &Map, pos: (usize, usize)) -> Vec<(usize, usize)> {
    let mut neighbors = Vec::new();
    let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];

    for (dx, dy) in directions.iter() {
        let new_x = pos.0 as isize + dx;
        let new_y = pos.1 as isize + dy;

        if new_x >= 0
            && new_x < map.config.width as isize
            && new_y >= 0
            && new_y < map.config.height as isize
        {
            let new_pos = (new_x as usize, new_y as usize);
            if map.is_walkable(new_pos.0, new_pos.1) {
                neighbors.push(new_pos);
            }
        }
    }

    neighbors
}

fn reconstruct_path(
    came_from: HashMap<(usize, usize), (usize, usize)>,
    mut current: (usize, usize),
) -> Vec<(usize, usize)> {
    let mut path = vec![current];
    while let Some(&prev) = came_from.get(&current) {
        path.push(prev);
        current = prev;
    }
    path.reverse();
    path
}
