use std::collections::{BTreeMap, HashMap, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct NodeId(usize);

#[derive(Debug, Default)]
struct Graph {
    names: Vec<String>,
    edges: HashMap<NodeId, Vec<(NodeId, u32)>>,
}

impl Graph {
    fn add_node(&mut self, name: &str) -> NodeId {
        self.names.push(name.to_string());
        NodeId(self.names.len() - 1)
    }

    fn add_edge(&mut self, a: NodeId, b: NodeId, w: u32) {
        self.edges.entry(a).or_default().push((b, w));
        self.edges.entry(b).or_default().push((a, w));
    }

    fn neighbors(&self, n: NodeId) -> impl Iterator<Item = &(NodeId, u32)> + '_ {
        self.edges.get(&n).into_iter().flatten()
    }

    fn name(&self, n: NodeId) -> &str {
        &self.names[n.0]
    }

    fn bfs_order(&self, start: NodeId) -> Vec<NodeId> {
        let mut seen = vec![false; self.names.len()];
        let mut order = Vec::new();
        let mut queue = VecDeque::from([start]);
        seen[start.0] = true;
        while let Some(n) = queue.pop_front() {
            order.push(n);
            let mut next: Vec<NodeId> = self.neighbors(n).map(|(m, _)| *m).filter(|m| !seen[m.0]).collect();
            next.sort();
            for m in next {
                if !seen[m.0] {
                    seen[m.0] = true;
                    queue.push_back(m);
                }
            }
        }
        order
    }

    fn dijkstra(&self, start: NodeId) -> BTreeMap<NodeId, u32> {
        let mut dist: BTreeMap<NodeId, u32> = BTreeMap::new();
        let mut pending: Vec<(u32, NodeId)> = vec![(0, start)];
        dist.insert(start, 0);
        while let Some(pos) = pending.iter().enumerate().min_by_key(|(_, (d, n))| (*d, *n)).map(|(i, _)| i) {
            let (d, n) = pending.swap_remove(pos);
            if d > dist[&n] {
                continue;
            }
            for &(m, w) in self.neighbors(n) {
                let nd = d + w;
                let better = dist.get(&m).is_none_or(|&cur| nd < cur);
                if better {
                    dist.insert(m, nd);
                    pending.push((nd, m));
                }
            }
        }
        dist
    }

    fn components(&self) -> Vec<Vec<NodeId>> {
        let mut seen = vec![false; self.names.len()];
        let mut out = Vec::new();
        for i in 0..self.names.len() {
            if seen[i] {
                continue;
            }
            let comp = self.bfs_order(NodeId(i));
            for n in &comp {
                seen[n.0] = true;
            }
            out.push(comp);
        }
        out
    }
}

fn topo_sort(deps: &[(&str, &str)]) -> Result<Vec<String>, String> {
    let mut indegree: BTreeMap<&str, usize> = BTreeMap::new();
    let mut adj: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for &(a, b) in deps {
        indegree.entry(a).or_insert(0);
        *indegree.entry(b).or_insert(0) += 1;
        adj.entry(a).or_default().push(b);
    }
    let mut ready: VecDeque<&str> = indegree.iter().filter(|(_, d)| **d == 0).map(|(n, _)| *n).collect();
    let mut order = Vec::new();
    while let Some(n) = ready.pop_front() {
        order.push(n.to_string());
        if let Some(next) = adj.get(n) {
            for &m in next {
                let d = indegree.get_mut(m).unwrap();
                *d -= 1;
                if *d == 0 {
                    ready.push_back(m);
                }
            }
        }
    }
    if order.len() == indegree.len() { Ok(order) } else { Err(format!("cycle among {} nodes", indegree.len() - order.len())) }
}

fn main() {
    let mut g = Graph::default();
    let ids: Vec<NodeId> = ["a", "b", "c", "d", "e", "f"].iter().map(|n| g.add_node(n)).collect();
    for &(a, b, w) in &[(0, 1, 7), (0, 2, 9), (0, 5, 14), (1, 2, 10), (1, 3, 15), (2, 3, 11), (2, 5, 2), (3, 4, 6), (4, 5, 9)] {
        g.add_edge(ids[a], ids[b], w);
    }
    let lonely = g.add_node("z");
    println!("{:?}", g.bfs_order(ids[0]).iter().map(|n| g.name(*n)).collect::<Vec<_>>());
    let dist = g.dijkstra(ids[0]);
    println!("{}", dist.iter().map(|(n, d)| format!("{}={d}", g.name(*n))).collect::<Vec<_>>().join(" "));
    println!("{:?} {:?}", dist.get(&lonely), g.neighbors(lonely).count());
    println!("{:?}", g.components().iter().map(|c| c.len()).collect::<Vec<_>>());
    println!("{:?}", topo_sort(&[("shirt", "tie"), ("tie", "jacket"), ("pants", "shoes"), ("pants", "belt"), ("belt", "jacket"), ("socks", "shoes")]));
    println!("{:?}", topo_sort(&[("a", "b"), ("b", "c"), ("c", "a"), ("x", "a")]));
    println!("{:?} {}", NodeId(3) < NodeId(10), g.edges.values().map(|v| v.len()).sum::<usize>());
}
