#![allow(dead_code)]

use itertools::Itertools;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

type Sides = [i32; 3];

fn main() -> Result<(), Box<dyn Error>> {
    // Open input.txt for the test cases
    let input = match File::open("input.txt") {
        Ok(file) => file,
        Err(_) => {
            println!("Expected input file input.txt in current working directory.");
            return Ok(());
        }
    };

    // Read the file line-by-line
    let mut lines = BufReader::new(input).lines();

    // Get the test case count and initialize the case vector
    let test_case_count = lines.next()
        .expect("Expected # of test cases on line 1")?
        .parse::<usize>()
        .expect("Invalid test case count");
    let mut test_cases: Vec<[Sides; 4]> = Vec::with_capacity(test_case_count);

    // Parse the test cases
    for _ in 0..test_case_count {
        let mut triangles = [[0i32; 3]; 4];
        for i in 0..4 {
            let line = lines.next().expect("Expected three sides of a triangle")?;
            let mut split = line.split(' ');
            for j in 0..triangles[i].len() {
                triangles[i][j] = split.next()
                    .expect("Expected triangle side length")
                    .parse::<i32>()
                    .expect("Invalid side length encountered");
            }
        }
        test_cases.push(triangles);
    }

    // Run each case
    for case in test_cases.iter() {
        if brute_force_by_area(case) {
            println!("Yes");
        } else {
            println!("No");
        }
    }

    Ok(())
}

/*
Methodology: find a combination of sides from the four given triangles that yields a larger triangle
with the same area as the sum of the remaining triangles.

In order to check all cases, combinations of 3, 4, 5, and 6 sides must be taken.

Things to note:
 - A side should not be counted twice
 - A maximum of two sides can be counted from an individual triangle
*/
fn brute_force_by_area(case: &[Sides; 4]) -> bool {
    // Compute target area
    let area_sum = case.iter()
        .map(|[a, b, c]| area(*a, *b, *c))
        .sum::<f64>();

    // Which triangle to pull a side from
    let mut triangle_choice = [0usize; 6];
    // Which side to choose based on the triangle from the array above
    let mut side_choice;

    for side_count in 3..=6 {
        // Initialize choices
        for i in 0..side_count {
            triangle_choice[i] = i % 4;
        }
        side_choice = [0usize, 0, 0, 0, 1, 1];

        loop {
            // Gather the current selected sides based on the two choice arrays and brute force all
            // combinations and groupings of those
            let combination_found = brute_force_groupings(
                triangle_choice.iter()
                    .enumerate()
                    .map(|(side_index, &index)| case[index][side_choice[side_index]])
                    .collect(),
                area_sum
            );

            if combination_found {
                return true;
            }

            /*
            The following code incrementaly traverses the valid combinations according to the restrictions
            outlined above. This through a tiered system, where if one level fails then it is reset and the
            next incrementation level is used. 1) and 2) only execute if the side count is 5 or 6.

            1) Try to increment the chosen sides for the triangles which have two sides selected. This simply
               "counts" using custom digit sizes, skipping side indices which were previously selected.
            
            2) If 1) fails, take the next combination of the triangles which have two sides selected.

            3) If 2) fails, increment the chosen sides for the triangles which have on side selected. This
               is done in a similar manner to 1).
            
            4) If 3) fails, take the next combination of the triangles which only have one side selected.

            If all four tiers fail then the function exits.
            */

            let mut i = 4;

            // Steps 1) and 2)
            if side_count > 4 {
                // Step 1)
                while i < side_count {
                    // Increment chosen side
                    side_choice[i] += 1;

                    // If it is the same as the first chosen side, skip it
                    if side_choice[i] == side_choice[triangle_choice[i]] {
                        side_choice[i] += 1;
                    }

                    // Roll over to the next index if needed
                    if side_choice[i] == 3 {
                        side_choice[i] = if side_choice[triangle_choice[i]] == 0 { 1 } else { 0 };
                        i += 1;
                    } else {
                        break;
                    }
                }

                if i == side_count {
                    // Step 2)
                    if next_combination(&mut triangle_choice[4..side_count], 4) {
                        // Reset the side choices
                        for j in 4..side_count {
                            side_choice[j] = if side_choice[triangle_choice[j]] == 0 { 1 } else { 0 };
                        }

                        continue;
                    }
                } else {
                    continue;
                }
            }

            // Step 3)
            i = 0;
            let single_side_count = side_count.min(4);
            while i < single_side_count {
                // Increment chosen side
                side_choice[i] += 1;

                // Roll over into the next index if needed
                if side_choice[i] == 3 {
                    side_choice[i] = 0;
                    i += 1;
                } else {
                    break;
                }
            }

            // At this point, the first sides chosen could conflict with the sides chosen for the triangles
            // that provide two sides. Thus, the data regarding the doubly-chosen triangles needs to be
            // reset.
            if side_count > 4 {
                for j in 4..side_count {
                    triangle_choice[j] = j - 4;
                }

                for j in 4..side_count {
                    side_choice[j] = if side_choice[triangle_choice[j]] == 0 { 1 } else { 0 };
                }
            }

            // Step 4) and exit condition
            if i == single_side_count && !next_combination(&mut triangle_choice[..single_side_count], 4) {
                break;
            }
        }
    }

    false
}

/*
This function is equivalent to brute_force_by_area except it does not adhere to any choosing restrictions.
*/
fn brute_force_by_area_unchecked(case: &[Sides; 4]) -> bool {
    let area_sum = case.iter()
        .map(|[a, b, c]| area(*a, *b, *c))
        .sum::<f64>();
    let all_sides = case.iter()
        .flatten()
        .cloned()
        .collect::<Vec<_>>();
    
    let mut side_selections = [0usize; 6];
    for side_count in 3..=6 {
        for j in 0..side_count {
            side_selections[j] = j;
        }

        loop {
            let combination_found = brute_force_groupings(
                side_selections.iter()
                    .map(|&index| all_sides[index])
                    .collect(),
                area_sum
            );
            
            if combination_found {
                return true;
            }

            if !next_combination(&mut side_selections[..side_count], all_sides.len()) {
                break;
            }
        }
    }

    false
}

/*
This function takes a vector of sides and combines them in every valid configuration to form three distinct
sides. The resulting triangle's area is then checked agains the sum to see if it is a match.

The way sides are chosen is through permutations of groupings. A grouping specifies the number of individual
sides to combine to obtain each larger side. A valid grouping will sum to the total number of individual
sides. For instance, the grouping [2, 1, 1] will take two individual sides and add them to form the first
larger side, then take individual sides to form the last two larger sides. This would be a valid grouping
for a total number of four sides.

For each individual group, every combination of sides of that group size is tested.

Valid groupings are iterated by increasing the number of sides used to form the first, larger side while
simultaneously decreasing the number used for later sides.

This function only works for side counts between 3 and 6 inclusive.
*/
fn brute_force_groupings(sides: Vec<i32>, area_sum: f64) -> bool {
    let side_count = sides.len();

    // Initial side groupings
    let mut grouping: [usize; 3] = match side_count {
        3 => [1, 1, 1],
        4 => [2, 1, 1],
        5 => [2, 2, 1],
        6 => [2, 2, 2],
        _ => panic!("Side count must be between 3 and 6 inclusive.")
    };

    // The three sides
    let mut a;
    let mut b;
    let mut c;

    loop {
        // The three groupings, 
        let mut group_a = (0..grouping[0]).collect::<Vec<_>>();
        let mut group_b: Vec<usize>;
        let mut group_c: Vec<usize>;

        loop {
            group_b = (0..grouping[1]).collect::<Vec<_>>();
            loop {
                group_c = (0..grouping[2]).collect::<Vec<_>>();
                loop {
                    // Compute the side lengths
                    let mut sides_copy = sides.clone();
                    a = group_a.iter().map(|&index| sides_copy[index]).sum::<i32>();
                    group_a.iter().cloned().sorted().rev().for_each(|index| { sides_copy.remove(index); });
                    b = group_b.iter().map(|&index| sides_copy[index]).sum::<i32>();
                    group_b.iter().cloned().sorted().rev().for_each(|index| { sides_copy.remove(index); });
                    c = group_c.iter().map(|&index| sides_copy[index]).sum::<i32>();
                    group_c.iter().cloned().sorted().rev().for_each(|index| { sides_copy.remove(index); });

                    // Check to see if it is a valid combination
                    let area = area(a, b, c);
                    if (area - area_sum).abs() < 1e-10 {
                        return true;
                    }

                    // Take the next combination of group c
                    if !next_combination(&mut group_c, side_count - grouping[0] - grouping[1]) {
                        break;
                    }
                }

                // Take the next combination of group b
                if !next_combination(&mut group_b, side_count - grouping[0]) {
                    break;
                }
            }

            // Take the next combination of group a
            if !next_combination(&mut group_a, side_count) {
                break;
            }
        }

        // Compute the next grouping
        if grouping[2] > 1 {
            grouping[0] += 1;
            grouping[2] -= 1;
        } else if grouping[1] > 1 {
            grouping[0] += 1;
            grouping[1] -= 1;
        } else {
            break false;
        }
    }
}

/*
Takes an input of numbers between 0 (inclusive) and bound (exclusive) in strictly increasing order and computes
the next combination of non-duplicated numbers between 0 and bound.
*/
fn next_combination(set: &mut [usize], bound: usize) -> bool {
    let len = set.len();

    // Empty set, no combinations
    if len == 0 {
        return false;
    }

    // Increment the final index
    set[len - 1] += 1;

    // If we hit the bound, find the highest-indexed number we can increase, and then count up from
    // that number by one for the remaining numbers
    if set[len - 1] == bound {
        // If the set is length one then we just need to count up to bound
        if len == 1 {
            return false;
        }

        // Find the highest index we can increase by one without collision
        let mut offset = 2;
        loop {
            // We can increase the number without exceeding the bound of violating the invariant
            // that the set is strictly increasing
            if set[len - offset] < bound - offset {
                break;
            }

            // We could not find a number to increase, all combinations have been taken
            if offset == len {
                return false;
            }

            offset += 1;
        }

        // Set the remaining numbers to increase by one after the number we just incremented
        set[len - offset] += 1;
        for i in (1..offset).rev() {
            set[len - i] = set[len - i - 1] + 1;
        }
    }

    true
}

fn area(a: i32, b: i32, c: i32) -> f64 {
    if a + b <= c || a + c <= b || b + c <= a {
        return 0.0;
    }

    let a = a as f64;
    let b = b as f64;
    let c = c as f64;
    0.25 * f64::sqrt((a + b + c) * (a + b - c) * (a - b + c) * (-a + b + c))
}