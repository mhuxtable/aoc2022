use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    cell::RefCell,
    collections::{hash_map::DefaultHasher, HashMap, HashSet},
    hash::{Hash, Hasher},
};

fn parse(input: &str) -> HashMap<String, (u32, Vec<String>)> {
    let mut valves = HashMap::new();

    lazy_static! {
        static ref re: Regex = Regex::new(
            r"(?m)^Valve ([A-Z]+) has flow rate=(\d+); tunnels? leads? to valves? (.+)$"
        )
        .unwrap();
    }

    for caps in re.captures_iter(input) {
        let (name, flow_rate, tunnels) = (caps[1].to_string(), &caps[2], &caps[3].to_string());

        let flow_rate: u32 = flow_rate.parse().unwrap();
        let tunnels: Vec<String> = tunnels.split(", ").map(|x| x.to_string()).collect();

        valves.insert(name, (flow_rate, tunnels));
    }

    valves
}

fn floyd(graph: &HashMap<String, (u32, Vec<String>)>) -> HashMap<String, HashMap<String, u32>> {
    let keys: Vec<&String> = graph.keys().clone().sorted().collect();
    let id_of = |node: &String| keys.iter().position(|&x| x == node).unwrap();

    let mut dist = vec![vec![u32::MAX; keys.len()]; keys.len()];
    for &key in &keys {
        dist[id_of(key)][id_of(key)] = 0;
        if !key.ends_with("+") {
            let augmented_node = format!("{}+", key);
            dist[id_of(key)][id_of(&augmented_node)] = 0;
            dist[id_of(&augmented_node)][id_of(key)] = 0;
        }

        for connection in &graph[key].1 {
            dist[id_of(key)][id_of(&connection)] = 1;
        }
    }

    if keys.contains(&&"Q".to_string()) {
        // The cost of getting from Q to AA is nothing as it's a fake node that restarts us back at
        // AA, and changes player.
        dist[id_of(&"Q".to_string())][id_of(&"AA".to_string())] = 0;

        for x in 0..keys.len() {
            dist[id_of(keys[x])][id_of(&"Q".to_string())] = 0;
        }
    }

    for k in 0..keys.len() {
        for i in 0..keys.len() {
            for j in 0..keys.len() {
                let alt = dist[i][k].saturating_add(dist[k][j]);
                if dist[i][j] > alt {
                    dist[i][j] = alt;
                }
            }
        }
    }

    let mut costs: HashMap<String, HashMap<String, u32>> = HashMap::new();

    for (i, &key) in keys.iter().enumerate() {
        if key != "AA" && key != "Q" && graph[key].0 == 0 {
            // We don't care about connections to rooms with flow 0; they are useless
            continue;
        }

        let entry = costs.entry(key.clone()).or_default();

        for (j, &connection) in keys.iter().enumerate() {
            if graph[connection].0 == 0 && connection != "Q" {
                // don't care about connections to rooms with flow 0
                continue;
            } else if connection == key {
                // don't track the 0 cost connection from room to room, as we don't want to
                // re-visit the same room again
                continue;
            }

            entry.insert(connection.clone(), dist[i][j]);
        }
    }

    costs
}

fn graph_with_actuation_nodes(
    graph: &HashMap<String, (u32, Vec<String>)>,
) -> HashMap<String, (u32, Vec<String>)> {
    // for each node K+, we add an additional connection K+ which models the cost of staying in the
    // location to open the valve. K+ is connected to K and all original connections of K. Note
    // that we never need to set up reverse connections; i.e. for some connection J of K (J ≠ K),
    // we do not need to connect J to K+.

    let mut augmented_graph = HashMap::new();

    for node in graph.keys() {
        let actuate_node = format!("{}+", node);
        let (flow_rate, mut tunnels) = graph[node].clone();

        augmented_graph.insert(actuate_node.clone(), (flow_rate, tunnels.clone()));

        tunnels.push(actuate_node);
        augmented_graph.insert(node.clone(), (0, tunnels));
    }

    augmented_graph
}

#[derive(Clone)]
struct State<'a> {
    current_node: &'a str,
    mins_remaining: usize,
    open_valves: HashSet<String>,
    flow: u32,
    can_take_q: bool,
    steps: Vec<(String, u32, usize)>,
}

fn hash_valves(s: &HashSet<String>) -> u64 {
    let mut hash = DefaultHasher::new();

    0xFF.hash(&mut hash);

    for valve in s.iter().sorted() {
        valve.hash(&mut hash);
    }

    0xFF.hash(&mut hash);

    hash.finish()
}

fn brute_force<'a>(
    state: &mut State,
    flow_rates: &HashMap<String, u32>,
    costs: &HashMap<String, HashMap<String, u32>>,
    best_paths: &RefCell<&'a mut HashMap<(String, usize, u64), (u32, Vec<String>)>>,
    best_q_paths: &RefCell<&'a mut HashMap<(String, usize, u64), (u32, Vec<String>)>>,
) -> (u32, Vec<String>) {
    // Vec<(String, u32, usize)>) {
    let memo_key = &(
        state.current_node.to_string(),
        state.mins_remaining,
        hash_valves(&state.open_valves),
    );

    let cache = if state.can_take_q {
        best_paths
    } else {
        best_q_paths
    };

    {
        let cached = cache.borrow();
        let cached = cached.get(memo_key);

        if cached.is_some() {
            let (flow, valves) = cached.unwrap();
            // println!(
            //     "hit cache {} {} {:?} = {} {:?}",
            //     state.current_node, state.mins_remaining, &state.open_valves, flow, &valves
            // );

            let mut open_valves = state.open_valves.clone();

            for valve in valves {
                open_valves.insert(valve.to_string());
            }

            return (
                state.flow + flow,
                state
                    .open_valves
                    .iter()
                    .map(|x| x.clone())
                    .collect::<Vec<String>>(),
            );
        }
    }

    let mut new_flow = 0;

    if state.current_node.ends_with("+")
        && !state.open_valves.contains(&state.current_node.to_string())
    {
        state.open_valves.insert(state.current_node.to_string());
        new_flow +=
            state.mins_remaining as u32 * flow_rates[state.current_node.trim_end_matches('+')];
    }

    state.steps.push((
        state.current_node.to_string(),
        state.flow,
        state.mins_remaining,
    ));

    let filter_next_nodes = |(neighbour, cost): (&String, &u32)| {
        (flow_rates[neighbour.trim_end_matches('+')] > 0)
            && state.mins_remaining.checked_sub(*cost as usize).is_some()
            && !state.open_valves.contains(neighbour)
    };

    let mut next_node_candidates: Vec<(String, &u32)> = costs[state.current_node]
        .iter()
        .filter(|(neighbour, cost)| filter_next_nodes((&neighbour, cost)))
        .map(|(neighbour, cost)| (neighbour.clone(), cost))
        .collect();

    // introduce a node "Q" that resets the timer
    // https://www.reddit.com/r/adventofcode/comments/znr2eh/comment/j0jlrrs/?utm_source=reddit&utm_medium=web2x&context=3
    if state.can_take_q {
        next_node_candidates.push(("Q".to_string(), &0));
    }

    let result = next_node_candidates
        .iter()
        .map(|(next_node, &cost)| {
            let mut state = State {
                current_node: if *next_node == "Q" {
                    "AA"
                } else {
                    next_node.as_str()
                },
                mins_remaining: if *next_node == "Q" {
                    26
                } else {
                    state.mins_remaining - cost as usize
                },
                open_valves: state.open_valves.clone(),
                flow: state.flow + new_flow,
                can_take_q: state.can_take_q && *next_node != "Q",
                steps: state.steps.clone(),
            };

            let result = brute_force(&mut state, flow_rates, costs, best_paths, best_q_paths);

            result
        })
        .sorted_by_key(|(flow, _)| *flow)
        .last();

    let default = (
        state.flow + new_flow,
        state
            .open_valves
            .iter()
            .map(|x| x.clone())
            .collect::<Vec<String>>(),
    );

    let result = result.unwrap_or(default);
    cache
        .borrow_mut()
        .insert(memo_key.clone(), (result.0 - state.flow, result.1.clone()));

    result
}

pub fn part_one(input: &str) -> Option<u32> {
    let valves = parse(input);
    let graph_with_actuation_nodes = graph_with_actuation_nodes(&valves);

    let costs = floyd(&graph_with_actuation_nodes);
    let flow_rates = valves
        .iter()
        .map(|(k, (flow_rate, _))| (k, *flow_rate))
        .fold(HashMap::new(), |mut acc, (key, flow_rate)| {
            acc.insert(key.clone(), flow_rate);
            acc
        });

    let mut state = State {
        current_node: "AA",
        mins_remaining: 30,
        open_valves: HashSet::new(),
        flow: 0,
        can_take_q: false,
        steps: vec![],
    };

    let mut memo = HashMap::new();
    let mut memoq = HashMap::new();

    let (flow, valves) = brute_force(
        &mut state,
        &flow_rates,
        &costs,
        &RefCell::new(&mut memo),
        &RefCell::new(&mut memoq),
    );
    dbg!(&valves);
    // dbg!(&steps);

    Some(flow)
}

pub fn part_two(input: &str) -> Option<u32> {
    let valves = parse(input);

    let graph_with_actuation_nodes = graph_with_actuation_nodes(&valves);

    let costs = floyd(&graph_with_actuation_nodes);
    let flow_rates = valves
        .iter()
        .map(|(k, (flow_rate, _))| (k, *flow_rate))
        .fold(HashMap::new(), |mut acc, (key, flow_rate)| {
            acc.insert(key.clone(), flow_rate);
            acc
        });

    dbg!(&costs);

    let mut state = State {
        current_node: "AA",
        mins_remaining: 26,
        open_valves: HashSet::new(),
        flow: 0,
        can_take_q: true,
        steps: vec![],
    };

    let mut memo = HashMap::new();
    let mut memoq = HashMap::new();

    let (flow, _valves) = brute_force(
        &mut state,
        &flow_rates,
        &costs,
        &RefCell::new(&mut memo),
        &RefCell::new(&mut memoq),
    );
    // dbg!(&steps);

    Some(flow)
}

fn main() {
    let input = &advent_of_code::read_file("inputs", 16);
    advent_of_code::solve!(1, part_one, input);
    advent_of_code::solve!(2, part_two, input);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let input = advent_of_code::read_file("examples", 16);
        assert_eq!(part_one(&input), Some(1651));
    }

    #[test]
    fn test_part_two() {
        let input = advent_of_code::read_file("examples", 16);
        assert_eq!(part_two(&input), Some(1707));
    }
}
