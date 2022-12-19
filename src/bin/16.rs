use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::{HashMap, HashSet};

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
        if key != "AA" && graph[key].0 == 0 {
            // We don't care about connections to rooms with flow 0; they are useless
            continue;
        }

        let entry = costs.entry(key.clone()).or_default();

        for (j, &connection) in keys.iter().enumerate() {
            if graph[connection].0 == 0 {
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
    current_nodes: Vec<&'a str>,
    mins_remaining: Vec<usize>,
    open_valves: HashSet<String>,
    flow: u32,
}

fn brute_force(
    mut state: &mut State,
    visits_per_round: usize,
    flow_rates: &HashMap<String, u32>,
    costs: &HashMap<String, HashMap<String, u32>>,
) -> (u32, Vec<String>) {
    for (i, current_node) in state.current_nodes.iter().enumerate() {
        if current_node.ends_with("+") && !state.open_valves.contains(&current_node.to_string()) {
            state.open_valves.insert(current_node.to_string());
            state.flow +=
                state.mins_remaining[i] as u32 * flow_rates[current_node.trim_end_matches('+')];
        }
    }

    assert!(visits_per_round >= 1 && visits_per_round <= 2);
    assert!(state.current_nodes.len() == visits_per_round);

    let filter_next_nodes = |i: usize, (neighbour, cost): (&String, &u32)| {
        flow_rates[neighbour.trim_end_matches('+')] > 0
            && state.mins_remaining[i]
                .checked_sub(*cost as usize)
                .is_some()
            && !state.open_valves.contains(neighbour)
    };

    let next_node_candidates: Vec<Vec<(&String, &u32)>> = state
        .current_nodes
        .iter()
        .enumerate()
        .map(|(i, current_node)| {
            costs[*current_node]
                .iter()
                .filter(|(neighbour, cost)| filter_next_nodes(i, (&neighbour, cost)))
                .collect()
        })
        .collect();

    let mut next_choices =
        Vec::with_capacity(next_node_candidates.iter().map(|x| x.len()).product());

    if visits_per_round == 1 {
        next_choices.extend(
            next_node_candidates[0]
                .iter()
                .map(|&x| vec![x])
                .collect::<Vec<Vec<(&String, &u32)>>>(),
        );
    } else {
        for &p1 in &next_node_candidates[0] {
            for &p2 in &next_node_candidates[1] {
                next_choices.push(vec![p1, p2]);
            }
        }
    }

    let result = next_choices
        .iter()
        .map(|next_nodes| {
            let (next_nodes, mins): (Vec<&String>, Vec<&u32>) =
                next_nodes.into_iter().map(|item| *item).unzip();

            let mut state = State {
                current_nodes: next_nodes.iter().map(|s| s.as_str()).collect(),
                mins_remaining: state
                    .mins_remaining
                    .iter()
                    .enumerate()
                    .map(|(i, rem)| rem - *mins[i] as usize)
                    .collect(),
                open_valves: state.open_valves.clone(),
                flow: state.flow,
            };

            let result = brute_force(&mut state, visits_per_round, flow_rates, costs);

            result
        })
        .sorted_by_key(|(flow, _)| *flow)
        .last();

    let default = (
        state.flow,
        state
            .open_valves
            .iter()
            .map(|x| x.clone())
            .collect::<Vec<String>>(),
    );

    result.unwrap_or(default)
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
        current_nodes: vec!["AA"],
        mins_remaining: vec![30],
        open_valves: HashSet::new(),
        flow: 0,
    };

    let (flow, valves) = brute_force(&mut state, 1, &flow_rates, &costs);
    dbg!(&valves);

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

    let mut state = State {
        current_nodes: vec!["AA", "AA"],
        mins_remaining: vec![26, 26],
        open_valves: HashSet::new(),
        flow: 0,
    };

    let (flow, valves) = brute_force(&mut state, 2, &flow_rates, &costs);
    dbg!(&valves);

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
