use std::collections::HashMap;

use nutype::nutype;
use sea_query::{Nullable, Value};
use serde::Serialize;

#[nutype(
    sanitize(trim, lowercase),
    validate(len_char_min = 3, len_char_max = 20),
    derive(
        Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Deref, Borrow, TryFrom,
        Serialize
    )
)]
pub struct MenuName(String);

// Implement From<MenuName> for sea_query::Value
impl From<MenuName> for Value {
    fn from(menu_name: MenuName) -> Self {
        Value::String(Some(Box::new(menu_name.into_inner())))
    }
}

// Implement Nullable for MenuName (needed for Option<MenuName>)
impl Nullable for MenuName {
    fn null() -> Value {
        Value::String(None)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, sqlx::FromRow)]
pub struct Menu {
    pub id: i64,
    pub name: String,
    pub parent_id: i64,
    pub parent_name: Option<String>,
    pub order_index: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct MenuTree {
    pub id: i64,
    pub name: String,
    pub selected: bool,
    pub partial_selected: bool,
    pub children: Vec<MenuTree>,
    pub is_authorized: bool,
}

impl MenuTree {
    pub fn new(
        id: i64,
        name: String,
        selected: bool,
        partial_selected: bool,
        children: Vec<MenuTree>,
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

pub fn children_menu_tree<'a>(
    menus: &'a [Menu],
    sid_map: &HashMap<i64, bool>,
    parent_id: i64,
) -> Vec<MenuTree> {
    // 构建父节点到子节点的索引
    let mut node_map: HashMap<i64, Vec<&'a Menu>> = HashMap::new();
    for menu in menus {
        node_map.entry(menu.parent_id).or_default().push(menu);
    }

    // 找到特定父ID的子节点并构建树
    node_map
        .get(&parent_id)
        .map(|children| build_menu_tree(&node_map, children, sid_map))
        .unwrap_or_default()
}

fn build_menu_tree<'a>(
    node_map: &HashMap<i64, Vec<&'a Menu>>,
    menus: &[&'a Menu],
    sid_map: &HashMap<i64, bool>,
) -> Vec<MenuTree> {
    // 预分配结果向量
    let mut trees = Vec::with_capacity(menus.len());

    for menu in menus {
        // 递归构建子节点
        let children = node_map
            .get(&menu.id)
            .map(|children| build_menu_tree(node_map, children, sid_map))
            .unwrap_or_default();

        // 初始化树节点
        let mut tree_node = MenuTree {
            id: menu.id,
            name: menu.name.clone(),
            selected: false,
            partial_selected: false,
            children,
            is_authorized: false,
        };

        // 计算节点的选中状态和授权状态
        calculate_selection_and_authorization_state(node_map, &mut tree_node, sid_map);

        trees.push(tree_node);
    }

    trees
}

fn calculate_selection_and_authorization_state(
    node_map: &HashMap<i64, Vec<&Menu>>,
    node: &mut MenuTree,
    sid_map: &HashMap<i64, bool>,
) {
    if !node_map.contains_key(&node.id) || node_map[&node.id].is_empty() {
        // 叶子节点：直接从sid_map获取选中状态和授权状态
        node.selected = sid_map.get(&node.id).copied().unwrap_or(false);
        node.is_authorized = sid_map.get(&node.id).copied().unwrap_or(false);
    } else {
        // 非叶子节点：计算选中状态和授权状态

        // 1. 如果所有子节点都被选中，则当前节点被选中
        node.selected = !node.children.iter().any(|child| !child.selected);

        // 2. 计算部分选中状态
        let has_selected = node.children.iter().any(|child| child.selected);
        let has_unselected = node.children.iter().any(|child| !child.selected);
        let has_partial = node.children.iter().any(|child| child.partial_selected);

        node.partial_selected = (has_selected && has_unselected) || has_partial;

        // 3. 计算授权状态：如果有任何子节点被授权，则当前节点也被授权
        node.is_authorized = node.children.iter().any(|child| child.is_authorized);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_children_menu_tree() {
        let menus = vec![
            Menu {
                id: 1,
                name: "111".to_string(),
                parent_id: -1,
                parent_name: None,
                order_index: 1,
            },
            Menu {
                id: 2,
                name: "222".to_string(),
                parent_id: 1,
                parent_name: Some("111".to_string()),
                order_index: 2,
            },
            Menu {
                id: 3,
                name: "333".to_string(),
                parent_id: 2,
                parent_name: Some("222".to_string()),
                order_index: 3,
            },
            Menu {
                id: 4,
                name: "444".to_string(),
                parent_id: 2,
                parent_name: Some("222".to_string()),
                order_index: 4,
            },
        ];
        let mut sid_map = HashMap::new();
        sid_map.insert(1, true);
        sid_map.insert(2, true);
        sid_map.insert(3, true);

        let parent_id = -1;
        let result = children_menu_tree(&menus, &sid_map, parent_id);
        println!("{:?}", result);
        assert_eq!(result.len(), 1);
    }
}
