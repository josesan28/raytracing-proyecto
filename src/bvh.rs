use crate::bounding_box::Aabb;
use crate::material::{Intersect, RayIntersect};
use crate::math::Vec3;

const LEAF_SIZE: usize = 4;

struct Node {
    bounds: Aabb,
    left: usize,
    right: usize,
    start: usize,
    count: usize,
}

pub struct Bvh {
    nodes: Vec<Node>,
    object_indices: Vec<usize>,
}

impl Bvh {
    pub fn build(objects: &[Box<dyn RayIntersect>]) -> Self {
        let mut object_indices: Vec<usize> = (0..objects.len()).collect();
        let mut nodes = Vec::new();
        if !objects.is_empty() {
            build_node(objects, &mut object_indices, &mut nodes, 0);
        }
        Self {
            nodes,
            object_indices,
        }
    }

    pub fn closest_intersection(
        &self,
        objects: &[Box<dyn RayIntersect>],
        origin: &Vec3,
        direction: &Vec3,
        max_distance: f32,
    ) -> Intersect {
        if self.nodes.is_empty() {
            return Intersect::empty();
        }

        let mut closest = Intersect::empty();
        let mut closest_distance = max_distance;
        let mut closest_index = usize::MAX;
        let mut stack = [0_usize; 64];
        let mut stack_size = 1;

        while stack_size > 0 {
            stack_size -= 1;
            let node_index = stack[stack_size];
            let node = &self.nodes[node_index];
            if !node.bounds.is_hit(origin, direction, closest_distance) {
                continue;
            }

            if node.count > 0 {
                for &object_index in &self.object_indices[node.start..node.start + node.count] {
                    let candidate = objects[object_index].ray_intersect(origin, direction);
                    let is_closer = candidate.distance < closest_distance
                        || (candidate.distance == closest_distance && object_index < closest_index);
                    if candidate.is_intersecting && is_closer {
                        closest_distance = candidate.distance;
                        closest_index = object_index;
                        closest = candidate;
                    }
                }
            } else {
                stack[stack_size] = node.left;
                stack_size += 1;
                stack[stack_size] = node.right;
                stack_size += 1;
            }
        }

        closest
    }
}

fn build_node(
    objects: &[Box<dyn RayIntersect>],
    indices: &mut [usize],
    nodes: &mut Vec<Node>,
    start: usize,
) -> usize {
    let mut bounds = objects[indices[0]].bounding_box();
    for &index in &indices[1..] {
        bounds = bounds.union(objects[index].bounding_box());
    }

    let node_index = nodes.len();
    nodes.push(Node {
        bounds,
        left: 0,
        right: 0,
        start,
        count: indices.len(),
    });

    if indices.len() <= LEAF_SIZE {
        return node_index;
    }

    let extent = bounds.extent();
    let axis = if extent.x >= extent.y && extent.x >= extent.z {
        0
    } else if extent.y >= extent.z {
        1
    } else {
        2
    };
    indices.sort_by(|left, right| {
        let left_center = objects[*left].bounding_box().centroid();
        let right_center = objects[*right].bounding_box().centroid();
        component(left_center, axis).total_cmp(&component(right_center, axis))
    });

    let middle = indices.len() / 2;
    let (left_indices, right_indices) = indices.split_at_mut(middle);
    let left = build_node(objects, left_indices, nodes, start);
    let right = build_node(objects, right_indices, nodes, start + middle);
    nodes[node_index].left = left;
    nodes[node_index].right = right;
    nodes[node_index].count = 0;
    node_index
}

fn component(vector: Vec3, axis: usize) -> f32 {
    match axis {
        0 => vector.x,
        1 => vector.y,
        _ => vector.z,
    }
}
