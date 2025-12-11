// Spatial navigation algorithms for calculating cursor movement

use super::types::{Rect, WASDKey};

/// Calculate the best next element to navigate to based on direction
pub fn find_nearest_in_direction(
    current_bounds: &Rect,
    candidates: &[(String, Rect)],
    direction: WASDKey,
) -> Option<String> {
    if candidates.is_empty() {
        return None;
    }

    let (dx, dy) = direction.direction_vector();
    let (current_x, current_y) = current_bounds.center();

    // Filter candidates that are in the desired direction
    let valid_candidates: Vec<_> = candidates
        .iter()
        .filter(|(_, bounds)| {
            let (target_x, target_y) = bounds.center();
            is_in_direction(current_x, current_y, target_x, target_y, dx, dy)
        })
        .collect();

    if valid_candidates.is_empty() {
        return None;
    }

    // Find the closest candidate using weighted distance
    valid_candidates
        .iter()
        .min_by(|(_, bounds_a), (_, bounds_b)| {
            let dist_a = calculate_directional_distance(
                current_x, current_y, 
                bounds_a.center().0, bounds_a.center().1,
                dx, dy
            );
            let dist_b = calculate_directional_distance(
                current_x, current_y,
                bounds_b.center().0, bounds_b.center().1,
                dx, dy
            );
            dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(id, _)| id.clone())
}

/// Check if target point is in the direction from current point
fn is_in_direction(
    current_x: f64,
    current_y: f64,
    target_x: f64,
    target_y: f64,
    direction_x: f64,
    direction_y: f64,
) -> bool {
    let to_target_x = target_x - current_x;
    let to_target_y = target_y - current_y;

    // Dot product with direction vector should be positive
    let dot_product = to_target_x * direction_x + to_target_y * direction_y;
    
    // Accept any forward movement (threshold > 0.0)
    // Using > 1.0 would exclude valid targets less than 1 pixel away
    dot_product > 0.0
}

/// Calculate distance with directional weighting
/// Prioritizes elements directly in line with the direction
fn calculate_directional_distance(
    current_x: f64,
    current_y: f64,
    target_x: f64,
    target_y: f64,
    direction_x: f64,
    _direction_y: f64,
) -> f64 {
    let to_target_x = target_x - current_x;
    let to_target_y = target_y - current_y;

    // Euclidean distance
    let direct_distance = (to_target_x.powi(2) + to_target_y.powi(2)).sqrt();

    // Calculate perpendicular distance from the directional axis
    // This penalizes elements that are off to the side
    let perpendicular_distance = if direction_x != 0.0 {
        // Horizontal movement - penalize vertical offset
        to_target_y.abs()
    } else {
        // Vertical movement - penalize horizontal offset
        to_target_x.abs()
    };

    // Weighted combination: prioritize aligned elements
    direct_distance + perpendicular_distance * 2.0
}

/// Navigate in grid layout
pub fn navigate_grid(
    current_index: usize,
    total_elements: usize,
    columns: usize,
    direction: WASDKey,
) -> Option<usize> {
    if total_elements == 0 {
        return None;
    }

    let rows = (total_elements + columns - 1) / columns;
    let current_row = current_index / columns;
    let current_col = current_index % columns;

    let new_index = match direction {
        WASDKey::W => {
            // Move up
            if current_row > 0 {
                Some(current_index - columns)
            } else {
                None
            }
        }
        WASDKey::S => {
            // Move down
            if current_row < rows - 1 {
                let candidate = current_index + columns;
                if candidate < total_elements {
                    Some(candidate)
                } else {
                    None
                }
            } else {
                None
            }
        }
        WASDKey::A => {
            // Move left
            if current_col > 0 {
                Some(current_index - 1)
            } else {
                None
            }
        }
        WASDKey::D => {
            // Move right
            if current_col < columns - 1 && current_index + 1 < total_elements {
                Some(current_index + 1)
            } else {
                None
            }
        }
    };

    new_index
}

/// Navigate in list layout
pub fn navigate_list(
    current_index: usize,
    total_elements: usize,
    is_vertical: bool,
    direction: WASDKey,
) -> Option<usize> {
    if total_elements == 0 {
        return None;
    }

    match (is_vertical, direction) {
        // Vertical list: W/S for navigation
        (true, WASDKey::W) => {
            if current_index > 0 {
                Some(current_index - 1)
            } else {
                None
            }
        }
        (true, WASDKey::S) => {
            if current_index < total_elements - 1 {
                Some(current_index + 1)
            } else {
                None
            }
        }
        // Horizontal list: A/D for navigation
        (false, WASDKey::A) => {
            if current_index > 0 {
                Some(current_index - 1)
            } else {
                None
            }
        }
        (false, WASDKey::D) => {
            if current_index < total_elements - 1 {
                Some(current_index + 1)
            } else {
                None
            }
        }
        // Other directions don't navigate in list mode
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_navigation() {
        // 3x3 grid (9 elements)
        let columns = 3;
        let total = 9;

        // From center (index 4), test all directions
        assert_eq!(navigate_grid(4, total, columns, WASDKey::W), Some(1)); // Up
        assert_eq!(navigate_grid(4, total, columns, WASDKey::S), Some(7)); // Down
        assert_eq!(navigate_grid(4, total, columns, WASDKey::A), Some(3)); // Left
        assert_eq!(navigate_grid(4, total, columns, WASDKey::D), Some(5)); // Right

        // From top-left (index 0), can't go up or left
        assert_eq!(navigate_grid(0, total, columns, WASDKey::W), None);
        assert_eq!(navigate_grid(0, total, columns, WASDKey::A), None);
    }

    #[test]
    fn test_vertical_list_navigation() {
        let total = 5;

        // Middle element
        assert_eq!(navigate_list(2, total, true, WASDKey::W), Some(1));
        assert_eq!(navigate_list(2, total, true, WASDKey::S), Some(3));
        
        // A/D don't work in vertical list
        assert_eq!(navigate_list(2, total, true, WASDKey::A), None);
        assert_eq!(navigate_list(2, total, true, WASDKey::D), None);

        // Boundaries
        assert_eq!(navigate_list(0, total, true, WASDKey::W), None);
        assert_eq!(navigate_list(4, total, true, WASDKey::S), None);
    }

    #[test]
    fn test_directional_filtering() {
        // Moving right (direction +1, 0)
        assert!(is_in_direction(0.0, 0.0, 5.0, 0.0, 1.0, 0.0)); // Directly right
        assert!(is_in_direction(0.0, 0.0, 5.0, 1.0, 1.0, 0.0)); // Slightly up-right
        assert!(!is_in_direction(0.0, 0.0, -5.0, 0.0, 1.0, 0.0)); // Left (wrong direction)
    }
}

