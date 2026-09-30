use std::collections::BTreeMap;

use super::geometry::{
    Edge, EdgeKind, Lane, MapLayout, NODE_HEIGHT, NODE_WIDTH, PIN_HEIGHT, PIN_WIDTH, Placed, Rect,
    VOLUME_HEIGHT, VOLUME_WIDTH,
};
use super::{MapService, talks_to};

/// The space around the map, and the height of the column labels above it.
const MARGIN: f32 = 40.;
const LABEL: f32 = 30.;
/// The gap between the pins and the lanes, and between the lanes and the volumes.
const COLUMN_GAP: f32 = 110.;
/// The padding inside a lane, and the height of its label.
const LANE_PAD: f32 = 30.;
const LANE_HEAD: f32 = 40.;
const LANE_GAP: f32 = 32.;
/// The gaps between the two node columns of a lane, and between its rows.
const NODE_GAP_X: f32 = 80.;
const NODE_GAP_Y: f32 = 48.;
/// The smallest gap between two pins or two volumes.
const STACK_GAP: f32 = 10.;
/// How far a same-column edge bends out to the right.
const BULGE: f32 = 50.;

/// Lays out `services` in columns: host ports, then a lane per network, then volumes.
/// The result depends only on names and numbers, not on the input order.
pub fn layout(services: &[MapService]) -> MapLayout {
    let mut sorted: Vec<&MapService> = services.iter().collect();
    sorted.sort_by(|a, b| (&a.name, &a.id).cmp(&(&b.name, &b.id)));
    let mut lanes: BTreeMap<&str, Vec<&MapService>> = BTreeMap::new();
    for service in &sorted {
        let network = service.networks.iter().min().map_or("", String::as_str);
        lanes.entry(network).or_default().push(service);
    }
    // The lane of services without a network goes last.
    let mut order: Vec<(&str, Vec<&MapService>)> = lanes.into_iter().collect();
    order.sort_by_key(|(network, _)| network.is_empty());

    let lane_x = MARGIN + PIN_WIDTH + COLUMN_GAP;
    let lane_w = 2. * LANE_PAD + 2. * NODE_WIDTH + NODE_GAP_X;
    let mut map = MapLayout {
        pins_label: (MARGIN, MARGIN),
        ..MapLayout::default()
    };
    let mut y = MARGIN + LABEL;
    for (network, members) in order {
        let columns = split_columns(&members);
        let rows = columns[0].len().max(columns[1].len());
        let h = LANE_HEAD + rows as f32 * (NODE_HEIGHT + NODE_GAP_Y) - NODE_GAP_Y + LANE_PAD;
        for (col, services) in columns.iter().enumerate() {
            let x = lane_x + LANE_PAD + col as f32 * (NODE_WIDTH + NODE_GAP_X);
            for (row, service) in services.iter().enumerate() {
                let rect = Rect {
                    x,
                    y: y + LANE_HEAD + row as f32 * (NODE_HEIGHT + NODE_GAP_Y),
                    w: NODE_WIDTH,
                    h: NODE_HEIGHT,
                };
                map.nodes.push(Placed {
                    key: service.id.clone(),
                    rect,
                });
            }
        }
        map.lanes.push(Lane {
            network: network.to_string(),
            rect: Rect {
                x: lane_x,
                y,
                w: lane_w,
                h,
            },
        });
        y += h + LANE_GAP;
    }

    place_pins(&mut map, &sorted);
    place_volumes(&mut map, &sorted, lane_x + lane_w + COLUMN_GAP);
    talk_edges(&mut map, &sorted);

    let right = map
        .volumes
        .iter()
        .map(|v| v.rect.right())
        .fold(lane_x + lane_w, f32::max);
    let bottom = map
        .lanes
        .iter()
        .map(|l| l.rect.bottom())
        .chain(map.pins.iter().map(|(_, _, r)| r.bottom()))
        .chain(map.volumes.iter().map(|v| v.rect.bottom()))
        .fold(MARGIN + LABEL, f32::max);
    map.width = right + MARGIN;
    map.height = bottom + MARGIN;
    map
}

/// The two node columns of a lane. Services that publish ports go in the first, so
/// port edges never cross a node; with none, the services fill both in turn.
fn split_columns<'a>(members: &[&'a MapService]) -> [Vec<&'a MapService>; 2] {
    let (publishing, quiet): (Vec<&MapService>, Vec<&MapService>) =
        members.iter().partition(|s| !s.ports.is_empty());
    if publishing.is_empty() {
        let first = quiet.iter().step_by(2).copied().collect();
        let second = quiet.iter().skip(1).step_by(2).copied().collect();
        [first, second]
    } else {
        [publishing, quiet]
    }
}

/// One pin per published port, beside its service, stacked when they would overlap.
fn place_pins(map: &mut MapLayout, services: &[&MapService]) {
    let mut wanted: Vec<(f32, u16, String)> = Vec::new();
    for service in services {
        let Some(node) = map.node(&service.id) else {
            continue;
        };
        let mut ports = service.ports.clone();
        ports.sort_unstable();
        ports.dedup();
        let stack = ports.len() as f32 * (PIN_HEIGHT + STACK_GAP) - STACK_GAP;
        for (i, port) in ports.into_iter().enumerate() {
            let top = node.mid_y() - stack / 2. + i as f32 * (PIN_HEIGHT + STACK_GAP);
            wanted.push((top, port, service.id.clone()));
        }
    }
    wanted.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    let tops = spread(wanted.iter().map(|(top, ..)| *top).collect(), PIN_HEIGHT);
    for ((_, port, id), top) in wanted.into_iter().zip(tops) {
        let rect = Rect {
            x: MARGIN,
            y: top,
            w: PIN_WIDTH,
            h: PIN_HEIGHT,
        };
        let node = map.node(&id).expect("a pin's service is placed");
        let from = (rect.right(), rect.mid_y());
        let to_y = from.1.clamp(node.y + 16., node.bottom() - 16.);
        map.edges.push(curve(EdgeKind::Port, from, (node.x, to_y)));
        map.pins.push((port, id, rect));
    }
}

/// One node per mounted volume, level with the middle of its users.
fn place_volumes(map: &mut MapLayout, services: &[&MapService], x: f32) {
    let mut users: BTreeMap<&str, Vec<Rect>> = BTreeMap::new();
    for service in services {
        let Some(node) = map.node(&service.id) else {
            continue;
        };
        for volume in &service.volumes {
            users.entry(volume).or_default().push(node);
        }
    }
    if users.is_empty() {
        return;
    }
    map.volumes_label = Some((x, MARGIN));
    let mut wanted: Vec<(f32, &str, Vec<Rect>)> = users
        .into_iter()
        .map(|(name, nodes)| {
            let mid = nodes.iter().map(Rect::mid_y).sum::<f32>() / nodes.len() as f32;
            (mid - VOLUME_HEIGHT / 2., name, nodes)
        })
        .collect();
    wanted.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(b.1)));
    let tops = spread(wanted.iter().map(|(top, ..)| *top).collect(), VOLUME_HEIGHT);
    for ((_, name, nodes), top) in wanted.into_iter().zip(tops) {
        let rect = Rect {
            x,
            y: top,
            w: VOLUME_WIDTH,
            h: VOLUME_HEIGHT,
        };
        for node in nodes {
            let from = (node.right(), node.mid_y());
            map.edges
                .push(curve(EdgeKind::Mount, from, (rect.x, rect.mid_y())));
        }
        map.volumes.push(Placed {
            key: name.to_string(),
            rect,
        });
    }
}

/// An edge per pair of services where one names the other, drawn once per pair.
fn talk_edges(map: &mut MapLayout, services: &[&MapService]) {
    let all: Vec<MapService> = services.iter().map(|s| (*s).clone()).collect();
    let mut drawn: Vec<(&str, &str)> = Vec::new();
    for service in &all {
        for target in talks_to(service, &all) {
            let pair = if service.id.as_str() < target {
                (service.id.as_str(), target)
            } else {
                (target, service.id.as_str())
            };
            if drawn.contains(&pair) {
                continue;
            }
            drawn.push(pair);
            let (Some(a), Some(b)) = (map.node(&service.id), map.node(target)) else {
                continue;
            };
            map.edges.push(link(a, b));
        }
    }
}

/// A talks-to edge: side to side between columns, or bent out to the right within one.
fn link(a: Rect, b: Rect) -> Edge {
    if (a.x - b.x).abs() < 1. {
        let (from, to) = ((a.right(), a.mid_y()), (b.right(), b.mid_y()));
        return Edge {
            kind: EdgeKind::TalksTo,
            from,
            c1: (from.0 + BULGE, from.1),
            c2: (to.0 + BULGE, to.1),
            to,
        };
    }
    let (left, right) = if a.x < b.x { (a, b) } else { (b, a) };
    curve(
        EdgeKind::TalksTo,
        (left.right(), left.mid_y()),
        (right.x, right.mid_y()),
    )
}

/// A left-to-right S curve.
fn curve(kind: EdgeKind, from: (f32, f32), to: (f32, f32)) -> Edge {
    let mid = (from.0 + to.0) / 2.;
    Edge {
        kind,
        from,
        c1: (mid, from.1),
        c2: (mid, to.1),
        to,
    }
}

/// Moves each top down as little as needed so items of height `h` do not overlap.
/// `tops` must be sorted.
fn spread(tops: Vec<f32>, h: f32) -> Vec<f32> {
    let mut floor = MARGIN + LABEL;
    tops.into_iter()
        .map(|top| {
            let top = top.max(floor);
            floor = top + h + STACK_GAP;
            top
        })
        .collect()
}

#[cfg(test)]
mod tests;
