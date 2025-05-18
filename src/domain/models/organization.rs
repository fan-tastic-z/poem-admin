use std::collections::HashMap;

use nutype::nutype;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum OrganizationLimitType {
    Root,                       // 跟组织
    FirstLevel,                 // 一级组织
    SubOrganization,            // 子组织
    SubOrganizationIncludeSelf, // 子组织包含自己
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, sqlx::FromRow)]
pub struct Organization {
    pub id: i64,
    pub name: String,
    pub parent_id: i64,
    pub parent_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CreateOrganizationRequest {
    pub name: OrganizationName,
    pub parent_id: i64,
    pub parent_name: Option<OrganizationName>,
}

impl CreateOrganizationRequest {
    pub fn new(
        name: OrganizationName,
        parent_id: i64,
        parent_name: Option<OrganizationName>,
    ) -> Self {
        Self {
            name,
            parent_id,
            parent_name,
        }
    }
}

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_min = 3, len_char_max = 20),
    derive(
        Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Deref, Borrow, TryFrom,
        Serialize
    )
)]
pub struct OrganizationName(String);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct OrganizationTree {
    pub id: i64,
    pub name: String,
    pub selected: bool,
    pub partial_selected: bool,
    pub children: Vec<OrganizationTree>,
    pub is_authorized: bool,
}

impl OrganizationTree {
    pub fn new(
        id: i64,
        name: String,
        selected: bool,
        partial_selected: bool,
        children: Vec<OrganizationTree>,
        is_authorized: bool,
    ) -> Self {
        Self {
            id,
            name,
            selected,
            partial_selected,
            children,
            is_authorized,
        }
    }
}

pub fn all_tree(organizations: &[Organization]) -> Vec<OrganizationTree> {
    let capacity = organizations.len();
    let mut node_map: HashMap<i64, Vec<&Organization>> = HashMap::with_capacity(capacity);
    let mut parent_id_map: HashMap<i64, i64> = HashMap::with_capacity(capacity);
    let mut self_map: HashMap<i64, &Organization> = HashMap::with_capacity(capacity);

    organizations.iter().for_each(|node| {
        node_map
            .entry(node.parent_id)
            .or_insert_with(Vec::new)
            .push(node);

        parent_id_map.insert(node.id, node.parent_id);
        self_map.insert(node.id, node);
    });

    let binding = Vec::new();
    let roots = node_map.get(&-1).unwrap_or(&binding);
    tree_dfs_organization(&node_map, roots, -1)
}

pub fn first_level_tree(organizations: &[Organization], id: i64, mid_id: i64) -> OrganizationTree {
    let capacity = organizations.len();
    let mut node_map: HashMap<i64, Vec<&Organization>> = HashMap::with_capacity(capacity);
    let mut parent_id_map: HashMap<i64, i64> = HashMap::with_capacity(capacity);
    let mut self_map: HashMap<i64, &Organization> = HashMap::with_capacity(capacity);

    organizations.iter().for_each(|node| {
        node_map
            .entry(node.parent_id)
            .or_insert_with(Vec::new)
            .push(node);

        parent_id_map.insert(node.id, node.parent_id);
        self_map.insert(node.id, node);
    });

    if id == -1 {
        return OrganizationTree {
            id: -1,
            name: "根组织".to_string(),
            selected: false,
            partial_selected: false,
            children: tree_dfs_organization(
                &node_map,
                node_map.get(&id).unwrap_or(&Vec::new()),
                mid_id,
            ),
            is_authorized: false,
        };
    }

    // 获取第一级ID
    let first_level_id = get_first_level_id(&parent_id_map, id);

    let is_authorized = mid_id == -1 || mid_id == first_level_id;
    let mid_id_for_dfs = if is_authorized { -1 } else { mid_id };

    let first_level_node = self_map.get(&first_level_id);

    OrganizationTree {
        id: first_level_node.map_or(0, |n| n.id),
        name: first_level_node.map_or_else(String::new, |n| n.name.clone()),
        selected: false,
        partial_selected: false,
        children: tree_dfs_organization(
            &node_map,
            node_map.get(&first_level_id).unwrap_or(&Vec::new()),
            mid_id_for_dfs,
        ),
        is_authorized,
    }
}

fn get_first_level_id(parent_id_map: &HashMap<i64, i64>, id: i64) -> i64 {
    parent_id_map
        .get(&id)
        .copied()
        .map(|parent_id| {
            if parent_id == -1 {
                id
            } else {
                get_first_level_id(parent_id_map, parent_id)
            }
        })
        .unwrap_or(-1)
}

pub fn children_organization_tree(
    organizations: &[Organization],
    parent_id: i64,
) -> Vec<OrganizationTree> {
    let mut m = HashMap::new();
    for organization in organizations {
        m.entry(organization.parent_id)
            .or_insert_with(Vec::new)
            .push(organization);
    }
    return tree_dfs_organization(&m, m.get(&parent_id).unwrap_or(&Vec::new()), -1);
}

fn tree_dfs_organization(
    node_map: &HashMap<i64, Vec<&Organization>>,
    nodes: &[&Organization],
    mid_id: i64,
) -> Vec<OrganizationTree> {
    nodes
        .iter()
        .map(|&node| {
            let is_authorized = node.id == mid_id || mid_id == -1;
            let children_mid_id = if is_authorized { -1 } else { mid_id };

            let children = node_map.get(&node.id).map_or_else(Vec::new, |children| {
                tree_dfs_organization(node_map, children, children_mid_id)
            });

            OrganizationTree {
                id: node.id,
                name: node.name.clone(),
                selected: false,
                partial_selected: false,
                children,
                is_authorized,
            }
        })
        .collect()
}
