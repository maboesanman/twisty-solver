use std::{borrow::Borrow, collections::{BTreeMap, HashMap}};

use bitvec::array;
use itertools::Itertools;

use crate::{
    Tables,
    kociemba::{
        coords::{CoordIdentityPerm, CornerOrientRawCoord, EdgeGroupOrientSymCoord},
        tables::prune_phase_1::{DenseSample, PrunePhase1Table},
    },
};

fn boxed_array<const N: usize, T: Copy + std::fmt::Debug>(item: T) -> Box<[T; N]> {
    vec![item; N].into_boxed_slice().try_into().unwrap()
}

const HISTOGRAM_COUNTS: [u64; 13] = [
    23234349, 28382355, 91063593, 402666785, 370708519, 382718993, 362771367, 343860449, 331420353,
    314668848, 140246523, 3107022, 6,
];

pub fn compute_column_permutations(tables: &Tables) -> (Box<[u16; 2187]>, Box<[u16; 64430]>) {
    let prune_phase_1_tables: &PrunePhase1Table<CoordIdentityPerm, CoordIdentityPerm, DenseSample> =
        tables.as_ref();

    let mut co_working = boxed_array(0);
    let mut ego_working = boxed_array(0);

    for i in 0u16..2187 {
        co_working[i as usize] = i;
    }
    for i in 0u16..64430 {
        ego_working[i as usize] = i;
    }

    let mut counts: BTreeMap<u8, u32> = BTreeMap::new();

    let (co_min_sort, ego_min_sort) = {
        let mut co_pre_sort = boxed_array::<2187, _>(u8::MAX);
        let mut ego_pre_sort = boxed_array::<64430, _>(u8::MAX);

        for i in 0..64430 {
            let ego = EdgeGroupOrientSymCoord::<CoordIdentityPerm>::new(i);

            for j in 0..2187 {
                let co = CornerOrientRawCoord::<CoordIdentityPerm>::new(j);

                let dist = prune_phase_1_tables.get_value(ego, co);

                co_pre_sort[j as usize] = co_pre_sort[j as usize].min(dist);
                ego_pre_sort[i as usize] = ego_pre_sort[i as usize].min(dist);
                *counts.entry(dist).or_default() += 1;
            }
        }

        (co_pre_sort, ego_pre_sort)
    };

    co_working.sort_by_key(|i| co_min_sort[*i as usize]);
    ego_working.sort_by_key(|i| ego_min_sort[*i as usize]);

    let mut co_groups = co_working
        .chunk_by_mut(|a, b| co_min_sort[*a as usize] == co_min_sort[*b as usize]).fuse();

    let mut ego_groups = ego_working
        .chunk_by_mut(|a, b| ego_min_sort[*a as usize] == ego_min_sort[*b as usize]).fuse();

    let counts: [u32; 13] = counts.values().copied().collect_array().unwrap();
    // let entry_count_sum = counts.iter().sum::<u32>() as f64;
    let access_count_sum = HISTOGRAM_COUNTS.iter().sum::<u64>() as f64;
    let probabilities = counts
        .iter()
        .copied()
        .zip(HISTOGRAM_COUNTS.iter().copied())
        .map(|(a, b)| {
            let a = a as f64;
            let b = b as f64;

            b / access_count_sum / a
        })
        .collect_array::<13>()
        .unwrap();

    let mut co_sorted: &[u16] = co_groups.next().unwrap();
    let mut ego_sorted: &[u16] = ego_groups.next().unwrap();

    loop {
        let mut did_something = false;
        let mut new_co = co_groups.next();
        let mut new_ego = ego_groups.next();

        if let Some(new_co) = new_co.as_mut() {
            did_something = true;
            sort_partial(new_co, co_sorted, ego_sorted, false, tables, &probabilities);
        }

        if let Some(new_ego) = new_ego.as_mut() {
            did_something = true;
            sort_partial(new_ego, ego_sorted, co_sorted, true, tables, &probabilities);
        }

        if let Some(new_co) = new_co {
            let new_co: &[u16] = new_co;
            assert_eq!(co_sorted.as_ptr().wrapping_add(co_sorted.len()), new_co.as_ptr());
            co_sorted = unsafe {
                std::slice::from_raw_parts(co_sorted.as_ptr(), co_sorted.len() + new_co.len())
            };
        }

        if let Some(new_ego) = new_ego {
            let new_ego: &[u16] = new_ego;
            assert_eq!(ego_sorted.as_ptr().wrapping_add(ego_sorted.len()), new_ego.as_ptr());
            ego_sorted = unsafe {
                std::slice::from_raw_parts(ego_sorted.as_ptr(), ego_sorted.len() + new_ego.len())
            };
        }

        if !did_something {
            break;
        }
    }

    // println!("{probabilities:?}");

    (co_working, ego_working)
}

const TILE_SIZE: usize = 16;

fn sort_partial(
    to_sort: &mut [u16],
    sorted_adjacent: &[u16],
    sorted_opposite: &[u16],
    transpose: bool,
    tables: &Tables,
    probabilities: &[f64],
) {
    if sorted_adjacent.len() + to_sort.len() < TILE_SIZE {
        return;
    }

    let a = sorted_adjacent.len() / TILE_SIZE;
    let b = sorted_adjacent.len() % TILE_SIZE;

    let prefix = &sorted_adjacent[a * TILE_SIZE..];

    let mut vectors: HashMap<u16, Vec<f64>> = HashMap::new();

    let prune_phase_1_tables: &PrunePhase1Table<CoordIdentityPerm, CoordIdentityPerm, DenseSample> =
        tables.as_ref();

    let mut total = prefix.len() + to_sort.len();
    let extra = match total % TILE_SIZE {
        0 => 0,
        x => TILE_SIZE - x
    };

    // total += extra;

    let cluster_count = total / TILE_SIZE;

    let mut array_data = Vec::new();
    let mut index_lookup = Vec::new();

    let mut sum_of_magnitudes = 0.0;
    
    for i in prefix.iter().chain(to_sort.iter()).copied() {
        let row_vec = sorted_opposite
            .iter()
            .map(|&j| {
                if transpose {
                    let ego = EdgeGroupOrientSymCoord::<CoordIdentityPerm>::new(i);
                    let co = CornerOrientRawCoord::<CoordIdentityPerm>::new(j);
                    prune_phase_1_tables.get_value(ego, co)
                } else {
                    let ego = EdgeGroupOrientSymCoord::<CoordIdentityPerm>::new(j);
                    let co = CornerOrientRawCoord::<CoordIdentityPerm>::new(i);
                    prune_phase_1_tables.get_value(ego, co)
                }
            })
            .map(|d| probabilities[d as usize])
            .chunks(TILE_SIZE)
            .into_iter()
            .map(|x| x.sum::<f64>() as f32)
            .collect_vec();

        sum_of_magnitudes += row_vec.iter().sum::<f32>().sqrt();

        array_data.extend(row_vec);
        index_lookup.push(i);
    }

    let avg_magnitude = sum_of_magnitudes / (total as f32);
    let correct = 1.0 / avg_magnitude;

    // array_data.iter_mut().for_each(|x| *x *= correct);

    let row_size = (sorted_opposite.len() + TILE_SIZE - 1) / TILE_SIZE;

    // array_data.extend((0..row_size * extra).into_iter().map(|_| 0.0));
    // index_lookup.extend((0..extra).into_iter().map(|_| 0));

    let data = ndarray::Array2::from_shape_vec((total, row_size), array_data).unwrap();

    let kmeans = kentro::KMeans::new(2)
        .with_euclidean(true)
        .with_verbose(true)
        .with_balanced(true)
        .with_iterations(10)
        .train(data.view(), None).unwrap()
        .into_iter().map(|v| v.len()).collect_vec();

    println!("{kmeans:?}");

    // for (i, v) in vectors {
    //     let magnitude = v.into_iter().map(|x| x * x).sum::<f64>().sqrt();

    //     println!("vector {}{i:05} magnitude {magnitude}", if transpose { "a" } else { "b" });
    // }

}

struct Cluster {
    items: [(u16, Vec<f32>); TILE_SIZE],
}

impl Cluster {
    fn optimal_exchange(&mut self, other: &mut Self) {
        let mut total_sum = sum(self.items.iter().chain(other.items.iter()).map(|i| i.1.as_slice()), None.into_iter());
        total_sum.iter_mut().for_each(|x| *x /= (TILE_SIZE * 2) as f32);
        let centroid = total_sum;
        let mut working = const {
            let mut init = [0usize; TILE_SIZE * 2];
            let mut i = 0;
            while i < TILE_SIZE * 2 {
                init[i] = i;
                i += 1;
            }

            init
        };
        let mut u = normalize(&self.items[0].1);

        for _ in 0..3 {
            // power iteration step: u = normalize(sum_v [ dot(v - centroid, u) * (v - centroid) ])
            let deviations: Vec<Vec<f32>> = self.items.iter().chain(other.items.iter())
                .map(|(_, v)| v.iter().zip(centroid.iter()).map(|(x, c)| x - c).collect())
                .collect();
            let weighted: Vec<Vec<f32>> = deviations.iter()
                .map(|d| scale(d, dot(d, &u)))
                .collect();
            u = sum(weighted.iter().map(|v| v.as_slice()), [].iter().map(|_: &Vec<f32>| [].as_slice()));
            normalize_in_place(&mut u);
        }

        for _ in 0..10 {
            let u = {
                let s = &mut working[TILE_SIZE..].iter().map(|x| self.items[*x - TILE_SIZE].1.as_slice());
                let o = &mut working[..TILE_SIZE].iter().map(|x| self.items[*x].1.as_slice());
                sum(s, o)
            };
            let init = working.clone();
            working.sort_by_cached_key(|&w| {
                let point = if w > TILE_SIZE {
                    self.items[w % TILE_SIZE].1.as_slice()
                } else {
                    other.items[w % TILE_SIZE].1.as_slice()
                };
                dot(point, &u);
            });
            if working == init {
                break
            }
        }
    }
}

fn power_iter_single<'a>(u: &'a mut [f32], w: &'a [f32], vecs: impl IntoIterator<Item = &'a [f32]>) {
    let mut result = Vec::new();
    for v in vecs {
        let s: f32 = v.iter().zip(w.iter()).zip(u.iter()).map(|((vi, wi), ui)| (vi - wi) * ui).sum();
        if result.is_empty() {
            result = v.iter().zip(w.iter()).map(|(vi, wi)| (vi - wi) * s).collect();
        } else {
            for (r, x) in result.iter_mut().zip(v.iter()) {
                *r += x * s;
            }
        }
    }
    u.copy_from_slice(&mut result);
}

fn dot(vector: &[f32], base: &[f32]) -> f32 {
    vector.iter().zip(base.iter()).map(|(a, b)| a * b).sum()
}

fn normalize(vector: &[f32]) -> Vec<f32> {
    let norm = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
    scale(vector, 1.0 / norm)
}

fn normalize_in_place(vector: &mut [f32]) {
    let norm = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
    scale_in_place(vector, 1.0 / norm);
}

fn scale(vector: &[f32], s: f32) -> Vec<f32> {
    vector.iter().map(|x| x * s).collect()
}

fn scale_in_place(vector: &mut [f32], s: f32) {
    vector.iter_mut().for_each(|x| *x *= s);
}

fn sum<'a>(pos: impl Iterator<Item = impl Borrow<[f32]>>, neg: impl Iterator<Item = Borrow<[f32]>>) -> Vec<f32> {
    let mut result = Vec::new();
    for (v, sign) in pos.map(|v| (v, 1.0f32)).chain(neg.map(|v| (v, -1.0f32))) {
        if result.is_empty() {
            result = v.iter().map(|x| x * sign).collect();
        } else {
            for (r, x) in result.iter_mut().zip(v.iter()) {
                *r += x * sign;
            }
        }
    }
    result
}

#[test]
fn do_the_thing() -> anyhow::Result<()> {
    let tables = Tables::new("tables")?;

    let (co, ego) = compute_column_permutations(&tables);

    // println!("co: {co:?}");
    // println!("ego: {ego:?}");



    Ok(())
}
